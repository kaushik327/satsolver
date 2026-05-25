#!/usr/bin/env python3
"""
Compare CDCL heuristics at the 3-SAT phase transition (l/n ≈ 4.27).

Sweeps each heuristic axis independently (polarity, restart, deletion)
and prints results as each configuration finishes.

Usage:
    uv run python scripts/heuristics.py [--n N] [--reps N] [--save FILE]
    uv run python scripts/heuristics.py --load FILE

    --n N...      variable count(s) to test  (default: 100)
                  pass multiple values for a scaling view, e.g. --n 50 75 100
    --reps N      solves per (config, n) cell  (default: 30)
    --save FILE   persist raw results to FILE for later --load
    --load FILE   skip sweep; re-display saved results
    --no-build    skip cargo build --release
"""

import argparse
import json
import statistics
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).parent.parent
BIN = REPO / "target" / "release" / "random"

PHASE_RATIO = 4.27  # l/n at the 3-SAT phase transition

# Each section sweeps one axis while holding the others at the default.
# Default config: polarity=phase-saving, restart=luby/100, deletion=lbd/6.
SECTIONS = [
    {
        "title": "polarity",
        "fixed": "restart=luby, deletion=lbd",
        "configs": [
            (
                "always-false",
                [
                    "--polarity",
                    "always-false",
                    "--restart",
                    "luby",
                    "--deletion",
                    "lbd",
                ],
            ),
            (
                "always-true",
                ["--polarity", "always-true", "--restart", "luby", "--deletion", "lbd"],
            ),
            (
                "phase-saving",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "luby",
                    "--deletion",
                    "lbd",
                ],
            ),
        ],
    },
    {
        "title": "restart",
        "fixed": "polarity=phase-saving, deletion=lbd",
        "configs": [
            (
                "none",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "none",
                    "--deletion",
                    "lbd",
                ],
            ),
            (
                "luby",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "luby",
                    "--deletion",
                    "lbd",
                ],
            ),
            (
                "geometric",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "geometric",
                    "--deletion",
                    "lbd",
                ],
            ),
        ],
    },
    {
        "title": "deletion",
        "fixed": "polarity=phase-saving, restart=luby",
        "configs": [
            (
                "none",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "luby",
                    "--deletion",
                    "none",
                ],
            ),
            (
                "lbd",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "luby",
                    "--deletion",
                    "lbd",
                ],
            ),
            (
                "activity",
                [
                    "--polarity",
                    "phase-saving",
                    "--restart",
                    "luby",
                    "--deletion",
                    "activity",
                ],
            ),
        ],
    },
    {
        "title": "baseline",
        "fixed": "all heuristics disabled",
        "configs": [
            (
                "all-off",
                [
                    "--polarity",
                    "always-false",
                    "--restart",
                    "none",
                    "--deletion",
                    "none",
                ],
            ),
        ],
    },
]


def build():
    print("Building release binary...", file=sys.stderr)
    subprocess.run(["cargo", "build", "--release", "--quiet"], cwd=REPO, check=True)


def run_config(args, n, reps):
    l = round(PHASE_RATIO * n)
    cmd = [str(BIN), "-n", str(n), "-l", str(l), "-r", str(reps), *args]
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    durations = []
    for line in result.stdout.splitlines():
        try:
            durations.append(int(json.loads(line)["duration_ms"]))
        except (json.JSONDecodeError, KeyError):
            continue
    return durations


def fmt_cell(samples):
    med = int(statistics.median(samples))
    p90 = int(sorted(samples)[int(len(samples) * 0.9)])
    mx = max(samples)
    return med, p90, mx


def section_header(section, ns, reps):
    title, fixed = section["title"], section["fixed"]
    label_w = max(len(lb) for lb, _ in section["configs"])
    single_n = len(ns) == 1

    if single_n:
        print(f"\n{title}  ({fixed})  n={ns[0]}, {reps} reps")
        print(f"  {'':>{label_w}}   median     p90     max")
    else:
        col_w = 14
        print(f"\n{title}  ({fixed})  {reps} reps")
        print(f"  {'':>{label_w}}" + "".join(f"  {'n='+str(n):^{col_w}}" for n in ns))
        print(f"  {'':>{label_w}}" + "".join(f"  {'med    p90':^{col_w}}" for _ in ns))

    return label_w


def print_row(label, label_w, samples_by_n, ns):
    single_n = len(ns) == 1
    if single_n:
        med, p90, mx = fmt_cell(samples_by_n[ns[0]])
        print(f"  {label:>{label_w}}   {med:>5}ms  {p90:>5}ms  {mx:>5}ms")
    else:
        col_w = 14
        parts = [f"  {label:>{label_w}}"]
        for n in ns:
            med, p90, _ = fmt_cell(samples_by_n[n])
            parts.append(f"  {med:>4}ms {p90:>4}ms  ")
        print("".join(parts))


def sweep_and_display(ns, reps, save_fh=None):
    for section in SECTIONS:
        label_w = section_header(section, ns, reps)
        for label, args in section["configs"]:
            samples_by_n = {}
            for n in ns:
                durations = run_config(args, n, reps)
                samples_by_n[n] = durations
                if save_fh:
                    for d in durations:
                        save_fh.write(
                            json.dumps(
                                {
                                    "section": section["title"],
                                    "label": label,
                                    "n": n,
                                    "duration_ms": d,
                                }
                            )
                            + "\n"
                        )
                    save_fh.flush()
            print_row(label, label_w, samples_by_n, ns)
            sys.stdout.flush()


def load_and_display(path):
    data = defaultdict(lambda: defaultdict(list))
    ns_seen = set()
    for line in path.read_text().splitlines():
        try:
            d = json.loads(line)
            data[(d["section"], d["label"])][int(d["n"])].append(int(d["duration_ms"]))
            ns_seen.add(int(d["n"]))
        except (json.JSONDecodeError, KeyError):
            continue

    ns = sorted(ns_seen)
    reps = min(len(v) for by_n in data.values() for v in by_n.values())

    for section in SECTIONS:
        label_w = section_header(section, ns, reps)
        for label, _ in section["configs"]:
            samples_by_n = data.get((section["title"], label), {})
            if samples_by_n:
                print_row(label, label_w, samples_by_n, ns)


def main():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--n", type=int, nargs="+", default=[100], metavar="N", dest="ns"
    )
    parser.add_argument("--reps", type=int, default=30, metavar="N")
    parser.add_argument("--save", type=Path, default=None, metavar="FILE")
    parser.add_argument("--load", type=Path, default=None, metavar="FILE")
    parser.add_argument("--no-build", action="store_true")
    args = parser.parse_args()

    if args.load:
        load_and_display(args.load)
        return

    if not args.no_build:
        build()

    if not BIN.exists():
        print(f"Binary not found: {BIN}", file=sys.stderr)
        sys.exit(1)

    save_fh = open(args.save, "w") if args.save else None
    sweep_and_display(sorted(args.ns), args.reps, save_fh)
    if save_fh:
        save_fh.close()
        print(f"\nResults saved to {args.save}")


if __name__ == "__main__":
    main()
