import requests
import json
import time

BASE_URL = "http://localhost:8000/api"

print("✅ Running Full Integration Test Suite")
print("=" * 70)

# Test 1: Create Project
print("\n📋 Test 1: POST /api/projects")
print("-" * 50)

try:
    r = requests.post(f"{BASE_URL}/projects", json={"name": "Integration Test Project"})
    print(f"Status Code: {r.status_code}")
    print(f"Response: {json.dumps(r.json(), indent=2)}")
    
    if r.status_code == 201:
        project_id = r.json()["project_id"]
        api_key = r.json()["api_key"]
        print(f"✅ SUCCESS: Got Project ID: {project_id}")
        print(f"✅ SUCCESS: Got API Key: {api_key}")
        print(f"⏱️  Waiting 200ms for database commit...")
        time.sleep(0.2)
        test1_pass = True
    else:
        print(f"❌ FAILED: Expected 201 Created")
        test1_pass = False
except Exception as e:
    print(f"❌ ERROR: {e}")
    test1_pass = False

if not test1_pass:
    print("\n❌ Aborting tests - project creation failed")
    exit(1)

# Test 2: Submit Error
print("\n📋 Test 2: Submit Error")
print("-" * 50)

error_payload = {
    "message": "Test error message",
    "stack": "at Object.<anonymous> (test.js:12:34)",
    "context": {
        "url": "https://example.com/test",
        "browser": "Chrome",
        "version": "120.0.0.0",
        "os": "Windows 11"
    }
}

headers = {
    "Authorization": f"Bearer {api_key}",
    "Content-Type": "application/json"
}

try:
    r = requests.post(f"{BASE_URL}/projects/{project_id}/errors", json=error_payload, headers=headers)
    print(f"Status Code: {r.status_code}")
    print(f"Response: {json.dumps(r.json(), indent=2)}")
    
    if r.status_code == 201:
        error_id = r.json()["id"]
        error_hash = r.json()["hash"]
        print(f"✅ SUCCESS: Error created with ID: {error_id}")
        test2_pass = True
    else:
        print(f"❌ FAILED: Expected 201 Created")
        test2_pass = False
except Exception as e:
    print(f"❌ ERROR: {e}")
    test2_pass = False

# Test 3: Deduplication
print("\n📋 Test 3: Deduplication (same error again)")
print("-" * 50)

try:
    r = requests.post(f"{BASE_URL}/projects/{project_id}/errors", json=error_payload, headers=headers)
    print(f"Status Code: {r.status_code}")
    print(f"Response: {json.dumps(r.json(), indent=2)}")
    
    if r.status_code == 201 and r.json()["was_duplicate"] == True and r.json()["count"] == 2:
        print(f"✅ SUCCESS: Deduplication working correctly, count = {r.json()['count']}")
        test3_pass = True
    else:
        print(f"❌ FAILED: Deduplication not working")
        test3_pass = False
except Exception as e:
    print(f"❌ ERROR: {e}")
    test3_pass = False

# Test 4: List Errors
print("\n📋 Test 4: List Errors endpoint")
print("-" * 50)

try:
    r = requests.get(f"{BASE_URL}/projects/{project_id}/errors", headers=headers)
    print(f"Status Code: {r.status_code}")
    data = r.json()
    print(f"Found {len(data['errors'])} errors in project")
    
    if r.status_code == 200 and len(data["errors"]) >= 1:
        print(f"✅ SUCCESS: Error listing working")
        test4_pass = True
    else:
        print(f"❌ FAILED: Error listing failed")
        test4_pass = False
except Exception as e:
    print(f"❌ ERROR: {e}")
    test4_pass = False

# Summary
print("\n" + "=" * 70)
print("✅ TEST SUMMARY")
print("=" * 70)
print(f"Test 1 - Create Project:  {'✅ PASS' if test1_pass else '❌ FAIL'}")
print(f"Test 2 - Submit Error:    {'✅ PASS' if test2_pass else '❌ FAIL'}")
print(f"Test 3 - Deduplication:   {'✅ PASS' if test3_pass else '❌ FAIL'}")
print(f"Test 4 - List Errors:     {'✅ PASS' if test4_pass else '❌ FAIL'}")
print("=" * 70)

passed = sum([test1_pass, test2_pass, test3_pass, test4_pass])
print(f"\n✅ {passed}/4 tests passed")

if passed == 4:
    print("\n🎉 ALL TESTS PASSED! Backend is fully functional.")
else:
    print("\n⚠️  Some tests failed - check above for details.")