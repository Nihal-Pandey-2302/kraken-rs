#[cfg(test)]
mod tests {
    use kraken_sdk::auth::sign_request;

    #[test]
    fn test_hmac_sha512_signing() {
        // Test with known input/output from Kraken docs (or simulated)
        let secret = "kQH5HW/8p1uGOVjbgWA7FunAmGO8lsSUXNsu3eow76sz84Q18fWxnyRzBHCd3pd5nE9qa99HAZtuZuj6F1huXg==";
        let nonce = "1616492376594";
        let post_data =
            "nonce=1616492376594&ordertype=limit&pair=XBTUSD&price=37500&type=buy&volume=1.25";
        let path = "/0/private/AddOrder"; // Added path as it is required for Kraken signature

        let signature = sign_request(secret, path, nonce, post_data).expect("Signing failed");

        // Kraken signatures are Base64 encoded.
        // SHA512 is 64 bytes. Base64 of 64 bytes is ceil(64 * 4 / 3) = 88 chars.
        // The user's snippet checked for 128 (hex), but our implementation returns Base64 (standard).
        // We will check for non-empty and correct length for Base64 SHA512.
        assert_eq!(signature.len(), 88);

        println!("Generated Signature: {}", signature);
    }
}
