#!/usr/bin/env bash
# Run the slopfork harness (compare + check) on every goldens case set.
# Usage: scripts/verify.sh [goldens-dir] [profile]
set -euo pipefail
cd "$(dirname "$0")/.."
G=${1:-../slopfork-mazelib-goldens}
P=${2:-tx}
H=.claude/skills/slopfork/scripts/diff_harness.py
cargo build --release --quiet --bin mazelib-adapter
A=target/release/mazelib-adapter
mkdir -p target/reports
status=0
for f in "$G"/cases/*.jsonl; do
  s=$(basename "$f" .jsonl)
  echo "== $s"
  python3 $H compare --batch --profile "$P" --cases "$f" --expected "$G/expected/$s.jsonl" \
    --cmd "$A" --json-report "target/reports/$s.compare.json" || status=1
  python3 $H check --batch --profile "$P" --cases "$f" --cmd "$A" \
    --checker "python3 $G/checkers/invariants.py" --json-report "target/reports/$s.check.json" || status=1
done
exit $status
