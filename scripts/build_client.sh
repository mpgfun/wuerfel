#!/usr/bin/env bash
set -e

cd game
wasm-pack build client --target web --out-dir ../../web/pkg
cd ..