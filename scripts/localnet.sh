#!/usr/bin/env bash
set -euo pipefail
RUST_LOG=info cargo run -- --localnet --port 4001 --datadir /tmp/qc1 &
PID1=$!
sleep 1
RUST_LOG=info cargo run -- --localnet --port 4002 --peers 127.0.0.1:4001 --datadir /tmp/qc2 &
PID2=$!
sleep 2
# OPTIONAL: adapt these lines to your CLI triggers (stdin, args, or RPC)
# echo "mine" | nc -N 127.0.0.1 4001 || true
# echo "send <ADDR1> 1000" | nc -N 127.0.0.1 4002 || true
sleep 1
kill $PID1 $PID2
