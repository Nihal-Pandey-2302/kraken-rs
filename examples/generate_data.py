import json

data = []
# Trade sample
trade = [
    0,
    [
        ["55000.00000", "0.01000000", "1699999999.1234", "b", "m", ""]
    ],
    "trade",
    "XBT/USD"
]
# Book sample
book = [
    0,
    {"a": [["55001.00000", "1.00000000", "1699999999.1234"]], "c": "1234"},
    "book-10",
    "XBT/USD"
]

# Generate 10,000 messages
for _ in range(5000):
    data.append(trade)
    data.append(book)

with open("examples/benchmark_data.json", "w") as f:
    json.dump(data, f)

print("Generated examples/benchmark_data.json with 10,000 messages.")
