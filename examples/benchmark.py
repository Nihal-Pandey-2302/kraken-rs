import json
import time

def main():
    # Setup: Load data and prepare as individual message strings
    with open("examples/benchmark_data.json", "r") as f:
        raw_data = json.load(f)
    
    msgs = [json.dumps(x) for x in raw_data]

    print(f"🐍 Benchmarking Python (Raw JSON) with {len(msgs)} messages...")

    start = time.time()
    count = 0
    for msg in msgs:
        # The core workload: Parse string -> Dict
        # This is the "fastest case" for Python (no object mapping overhead)
        data = json.loads(msg)
        count += 1
    
    end = time.time()
    duration = end - start

    print(f"✅ Processed {count} messages in {duration:.4f}s")
    print(f"🐌 Throughput: {count / duration:.2f} msgs/sec")

if __name__ == "__main__":
    main()
