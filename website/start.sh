#!/usr/bin/env bash
set -euo pipefail

SERVICE="website"

sudo systemctl daemon-reload
sudo systemctl restart "${SERVICE}"

sleep 2

sudo systemctl --no-pager --full status "${SERVICE}"

journalctl -u "${SERVICE}" -n 50 --no-pager