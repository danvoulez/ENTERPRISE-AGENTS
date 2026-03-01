#!/usr/bin/env bash
set -euo pipefail
pkill -f "node dist/src/main.js" || true
