#!/usr/bin/env python3
"""
Example NSG job script.
This is the entry point that NSG will execute.
"""
import sys
import platform
import json
from datetime import datetime

print("=" * 80)
print("NSG Test Job - Example Python Script")
print("=" * 80)
print()

print(f"Python version: {sys.version}")
print(f"Platform: {platform.platform()}")
print(f"Architecture: {platform.machine()}")
print(f"Hostname: {platform.node()}")
print(f"Current time: {datetime.now().isoformat()}")
print()

# Read parameters if params.json exists
params = {}
try:
    with open("params.json", "r") as f:
        params = json.load(f)
    print("Parameters loaded from params.json:")
    print(json.dumps(params, indent=2))
except FileNotFoundError:
    print("No params.json found - using defaults")
except Exception as e:
    print(f"Error loading params.json: {e}")

print()
print("-" * 80)
print("Job Processing")
print("-" * 80)
print()

# Simulate some work
import time
print("Processing data...")
for i in range(5):
    print(f"  Step {i+1}/5...")
    time.sleep(1)

# Write output
output_data = {
    "status": "completed",
    "timestamp": datetime.now().isoformat(),
    "platform": platform.platform(),
    "python_version": sys.version,
    "parameters": params,
    "results": {
        "message": "Test job completed successfully",
        "steps_completed": 5
    }
}

with open("test_output.json", "w") as f:
    json.dump(output_data, f, indent=2)

print()
print("=" * 80)
print("Job Completed Successfully!")
print("=" * 80)
print()
print("Output written to: test_output.json")
print()

sys.exit(0)
