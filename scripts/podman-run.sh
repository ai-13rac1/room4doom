#!/usr/bin/env bash
# Runs a command in the build image, with the repository at /src.
#
#   scripts/podman-run.sh cargo test --workspace
#
# The cargo registry cache is the Podman volume "room4doom-cargo". Build
# output goes to target/.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
SRC=${SRC:-$ROOT}
IMAGE=localhost/room4doom-build:1

if ! podman image exists "$IMAGE"; then
  podman build -t "$IMAGE" -f "$ROOT/podman/Containerfile" "$ROOT/podman"
fi

# Pass SOURCE_DATE_EPOCH only when it is set: an empty value makes GCC
# stop with "SOURCE_DATE_EPOCH must be a non-negative decimal integer".
ENV_ARGS=()
if [ -n "${SOURCE_DATE_EPOCH:-}" ]; then
  ENV_ARGS+=(-e "SOURCE_DATE_EPOCH=$SOURCE_DATE_EPOCH")
fi

exec podman run --rm \
  -v "$SRC:/src:Z" \
  -v room4doom-cargo:/cache \
  "${ENV_ARGS[@]}" \
  -w /src \
  "$IMAGE" "$@"
