#!/bin/sh
set -eu

mkdir -p target dist-dev
exec env CARGO_TARGET_DIR=target/trunk-dev trunk serve --dist dist-dev
