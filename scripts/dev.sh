#!/usr/bin/env bash

set -e

./scripts/build_client.sh

cargo run --manifest-path=game/server/Cargo.toml