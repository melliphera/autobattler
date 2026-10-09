#!/usr/bin/env bash
# Reproduces the optimisation log in README.md: builds each listed commit in a
# temporary worktree and runs the single-threaded benchmark for 8 seconds.
# Usage: scripts/bench_history.sh [commit ...]
set -euo pipefail
commits=("$@")
[ ${#commits[@]} -eq 0 ] && commits=(1054c43 b4c7f6c 7db4492 73882a4 0f657b6 9a7a806 ec5d98d 1135970 8782854 HEAD)
work=$(mktemp -d)
export CARGO_TARGET_DIR="$work/target"
trap 'git worktree remove -f "$work/wt" 2>/dev/null; rm -rf "$work"' EXIT
for c in "${commits[@]}"; do
  git worktree add -q -f "$work/wt" "$c"
  echo 'fn main(){ autobattler::dev_tools::profiling::profile(8 as _); }' > "$work/wt/src/bin/zz_bench.rs"
  fights=$(cd "$work/wt" && cargo run -q --release --bin zz_bench 2>/dev/null \
    | grep 'simulations completed' | tail -1 | awk '{print $1}')
  echo "$c: $(( ${fights:-0} / 8 )) fights/s"
  git worktree remove -f "$work/wt"
done
