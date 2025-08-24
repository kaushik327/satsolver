#!/usr/bin/env python3
# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "matplotlib",
# ]
# ///

"""
This script reads a JSONL file containing SAT solver benchmark results and generates
two graphs:
1. Average duration vs L/N ratio
2. SAT prob vs L/N ratio

Usage: uv run viz.py <jsonl_file>
"""

import json
import argparse
from collections import defaultdict
from typing import Dict, List, Tuple
import matplotlib.pyplot as plt


def group_by_ratio(records: List[Dict]) -> Dict[float, List[Dict]]:
    """Group records by L/N ratio."""
    ratio_groups = defaultdict(list)

    for record in records:
        assert record["n"] != 0, f"{record} is an invalid record"
        ratio_groups[record["l"] / record["n"]].append(record)

    return dict(ratio_groups)


def calculate_metrics(
    ratio_groups: Dict[float, List[Dict]],
) -> Tuple[List[float], List[float], List[float]]:
    """Calculate average duration and SAT prob for each l/n ratio."""
    ratios = []
    avg_durations = []
    sat_probs = []

    for ratio, records in ratio_groups.items():
        # Calculate average duration
        avg_duration = sum(r["duration"] for r in records) / len(records)

        # Calculate SAT prob
        sat_prob = sum(1 for r in records if r["sat"]) / len(records)

        ratios.append(ratio)
        avg_durations.append(avg_duration)
        sat_probs.append(sat_prob)

    return ratios, avg_durations, sat_probs


def create_plots(
    ratios: List[float], avg_durations: List[float], sat_probs: List[float]
):
    """Create and display the two plots."""
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))

    # Plot 1: Average Duration vs L/N ratio
    ax1.plot(ratios, avg_durations, "b-o", linewidth=2, markersize=4)
    ax1.set_xlabel("L/N ratio")
    ax1.set_ylabel("Average Duration (ms)")
    ax1.set_title("Average 3SAT Solving Duration vs Clause-to-Variable Ratio")
    ax1.grid(True, alpha=0.3)

    # Plot 2: SAT prob vs L/N ratio
    ax2.plot(ratios, sat_probs, "r-o", linewidth=2, markersize=4)
    ax2.set_xlabel("L/N ratio")
    ax2.set_ylabel("Probability of Satisfiable Formulas")
    ax2.set_title("3SAT Probability of Satisfiability vs Clause-to-Variable Ratio")
    ax2.set_ylim(0, 1)
    ax2.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.show()


def main():
    parser = argparse.ArgumentParser(
        description="Visualize SAT solver performance data"
    )
    parser.add_argument(
        "jsonl_file", help="Path to JSONL file containing benchmark results"
    )

    args = parser.parse_args()

    # Parse data
    print(f"Reading data from {args.jsonl_file}...")
    with open(args.jsonl_file, "r") as f:
        records = [json.loads(line) for line in f if not line.isspace()]
    print(f"Loaded {len(records)} records")

    # Group by L/N ratio
    ratio_groups = group_by_ratio(records)
    print(f"Found {len(ratio_groups)} unique L/N ratios")

    # Calculate metrics
    ratios, avg_durations, sat_probs = calculate_metrics(ratio_groups)

    # Display summary statistics
    print(f"\nRatio range: {min(ratios):.2f} to {max(ratios):.2f}")
    print(f"Duration range: {min(avg_durations):.2f} to {max(avg_durations):.2f} ms")
    print(f"SAT prob range: {min(sat_probs):.2f} to {max(sat_probs):.2f}")

    # Create plots
    create_plots(ratios, avg_durations, sat_probs)


if __name__ == "__main__":
    main()
