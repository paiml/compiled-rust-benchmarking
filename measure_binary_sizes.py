#!/usr/bin/env python3
"""Measure binary sizes for all benchmarks and profiles"""

import json
import os
import subprocess
from pathlib import Path

def get_binary_size(benchmark, profile):
    """Get the size of a binary in bytes"""
    # Try multiple possible binary locations
    possible_paths = [
        f"target/{profile}/{benchmark}",
        f"target/release/{benchmark}",
        f"target/debug/{benchmark}",
    ]

    for path in possible_paths:
        if os.path.exists(path):
            return os.path.getsize(path)

    # Build if not found
    try:
        subprocess.run(
            ["cargo", "build", "-p", benchmark, "--profile", profile],
            capture_output=True,
            check=True
        )

        # Check again after building
        for path in possible_paths:
            if os.path.exists(path):
                return os.path.getsize(path)
    except subprocess.CalledProcessError:
        pass

    return None

def main():
    print("Measuring binary sizes for pathfinder profiles...\n")

    # Load pathfinder results to get all benchmark/profile combinations
    with open("pathfinder_results.json", "r") as f:
        results_data = json.load(f)

    binary_sizes = {}

    for result in results_data["results"]:
        benchmark = result["job"]["benchmark"]
        profile = result["job"]["config_id"]
        job_id = result["job"]["job_id"]

        if job_id in binary_sizes:
            continue  # Already measured

        print(f"Measuring {benchmark} × {profile}...")
        size = get_binary_size(benchmark, profile)

        if size is not None:
            binary_sizes[job_id] = {
                "benchmark": benchmark,
                "profile": profile,
                "size_bytes": size,
                "size_kb": size / 1024,
                "size_mb": size / (1024 * 1024)
            }
            print(f"  ✓ {size_kb:.1f} KB")
        else:
            print(f"  ✗ Failed to measure")

    # Save results
    output_file = "binary_sizes.json"
    with open(output_file, "w") as f:
        json.dump(binary_sizes, f, indent=2)

    print(f"\n✅ Measured {len(binary_sizes)} binaries")
    print(f"   Saved to {output_file}")

    # Quick summary
    if binary_sizes:
        sizes_kb = [data["size_kb"] for data in binary_sizes.values()]
        print(f"\nSize range: {min(sizes_kb):.1f} KB - {max(sizes_kb):.1f} KB")
        print(f"Average: {sum(sizes_kb) / len(sizes_kb):.1f} KB")

if __name__ == "__main__":
    main()
