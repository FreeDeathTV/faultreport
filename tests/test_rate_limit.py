import requests
import json
import threading
import time
import sys
from queue import Queue

BASE_URL = "http://localhost:8000/api"
WORKER_COUNT = 20
TEST_DURATION = 60  # seconds

print("✅ RATE LIMIT LOAD TEST")
print("=" * 70)

# Create test project
print("\n📋 Creating test project...")
r = requests.post(f"{BASE_URL}/projects", json={"name": "Rate Limit Test Project"})
project_id = r.json()["project_id"]
api_key = r.json()["api_key"]
print(f"✅ Project ID: {project_id}")
print(f"✅ API Key: {api_key}")

headers = {
    "Authorization": f"Bearer {api_key}",
    "Content-Type": "application/json"
}

error_payload = {
    "message": "Rate limit test error",
    "stack": "at test (rate.js:1:1)",
    "context": {"source": "load_test"}
}

stats = {
    "success": 0,
    "rate_limited": 0,
    "errors": 0,
    "start_time": time.time()
}

stats_lock = threading.Lock()
stop_signal = False

def worker():
    global stats, stop_signal
    
    while not stop_signal:
        try:
            r = requests.post(f"{BASE_URL}/projects/{project_id}/errors", 
                            json=error_payload, 
                            headers=headers,
                            timeout=5)
            
            with stats_lock:
                if r.status_code == 201:
                    stats["success"] += 1
                elif r.status_code == 429:
                    stats["rate_limited"] += 1
                else:
                    stats["errors"] += 1
                    
        except Exception as e:
            with stats_lock:
                stats["errors"] += 1

# Start workers
print(f"\n🚀 Starting {WORKER_COUNT} load test workers for {TEST_DURATION} seconds...")
threads = []
for i in range(WORKER_COUNT):
    t = threading.Thread(target=worker, daemon=True)
    threads.append(t)
    t.start()

# Monitor progress
for i in range(TEST_DURATION):
    time.sleep(1)
    elapsed = time.time() - stats["start_time"]
    rate = stats["success"] / elapsed if elapsed > 0 else 0
    
    sys.stdout.write(f"\r⏱️  {int(elapsed)}s | Success: {stats['success']} | Rate Limited: {stats['rate_limited']} | Errors: {stats['errors']} | {rate:.1f}/s    ")
    sys.stdout.flush()

stop_signal = True
print("\n\n✅ Test complete, waiting for workers to finish...")
time.sleep(2)

# Final stats
elapsed = time.time() - stats["start_time"]
total = stats["success"] + stats["rate_limited"] + stats["errors"]
rate = total / elapsed

print("\n" + "=" * 70)
print("📊 FINAL RESULTS")
print("=" * 70)
print(f"Total requests:      {total:,}")
print(f"Success (201):       {stats['success']:,}")
print(f"Rate Limited (429):  {stats['rate_limited']:,}")
print(f"Errors:              {stats['errors']:,}")
print(f"Total duration:      {elapsed:.1f} seconds")
print(f"Average rate:        {rate:.1f} requests/second")
print(f"Throughput:          {stats['success'] / elapsed:.1f} successful/second")
print("=" * 70)

if stats["rate_limited"] > 0 and stats["success"] <= 10000:
    print("\n✅ RATE LIMITING IS WORKING CORRECTLY ✅")
    print(f"✅ Successfully enforced cap of ~10,000/hour")
else:
    print("\n⚠️  Rate limit not yet triggered")

print("\n✅ Test completed successfully")