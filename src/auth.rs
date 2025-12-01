use base64::{engine::general_purpose, Engine as _};
use eyre::Result;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha512 = Hmac<Sha512>;

pub struct Authenticator {
    api_key: String,
    api_secret: String,
    client: Client,
}

#[derive(Deserialize)]
struct TokenResponse {
    error: Vec<String>,
    result: Option<TokenResult>,
}

#[derive(Deserialize)]
struct TokenResult {
    token: String,
}

impl Authenticator {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
            client: Client::new(),
        }
    }

    pub async fn get_ws_token(&self) -> Result<String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_string();

        let path = "/0/private/GetWebSocketsToken";
        let url = format!("https://api.kraken.com{}", path);
        let post_data = format!("nonce={}", nonce);

        // 1. SHA256(nonce + POST data)
        let mut sha256 = Sha256::new();
        sha256.update(nonce.as_bytes());
        sha256.update(post_data.as_bytes());
        let sha256_digest = sha256.finalize();

        // 2. HMAC-SHA512(path + sha256_digest, secret)
        let secret_bytes = general_purpose::STANDARD.decode(&self.api_secret)?;
        let mut mac = HmacSha512::new_from_slice(&secret_bytes)?;
        mac.update(path.as_bytes());
        mac.update(&sha256_digest);
        let sig_bytes = mac.finalize().into_bytes();
        let signature = general_purpose::STANDARD.encode(sig_bytes);

        // 3. Send Request
        let resp = self
            .client
            .post(&url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .body(post_data)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        if !resp.error.is_empty() {
            return Err(eyre::eyre!("Kraken API Error: {:?}", resp.error));
        }

        Ok(resp.result.unwrap().token)
    }
}
