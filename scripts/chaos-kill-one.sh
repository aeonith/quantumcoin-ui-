#!/usr/bin/env bash
set -euo pipefail

echo "🎯 Starting chaos engineering test..."

# Kill one replica at random, wait, then let restart:always bring it back
CANDIDATES=(ai-daemon-1 ai-daemon-2)
VICTIM=${CANDIDATES[$RANDOM % ${#CANDIDATES[@]}]}

echo "🔥 Killing $VICTIM for 10 seconds..."
docker compose -f docker-compose.production.yml kill "$VICTIM" || true

echo "⏱️  Waiting 10 seconds to test failover..."
sleep 10

echo "♻️  Restarting $VICTIM..."
docker compose -f docker-compose.production.yml up -d "$VICTIM"

echo "⏱️  Waiting for restart..."
sleep 5

echo "✅ Chaos test completed. Service should have recovered."
