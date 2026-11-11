#!/usr/bin/env bash
# Builds the Linux release of the game in Podman and writes dist/.
#
#   scripts/build-linux.sh
#
# The build is reproducible: the same commit gives the same bytes. File
# times in the archive are SOURCE_DATE_EPOCH (the commit time). The archive
# carries Freedoom (freedoom1.wad, BSD license), so it is ready to play.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/.." && pwd)
export SRC=${SRC:-$ROOT}
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-$(git -C "$SRC" log -1 --format=%ct)}
"$HERE/fetch-freedoom.sh" "$SRC/.freedoom" >/dev/null

"$HERE/podman-run.sh" bash -c '
set -euo pipefail
export RUSTFLAGS="--remap-path-prefix=/src=. --remap-path-prefix=$CARGO_HOME=/cargo -C link-arg=-Wl,--build-id=none"
cargo build --locked --release -p room4doom
V=$(sed -n "/^\[workspace.package\]/,/^\[/s/^version = \"\(.*\)\"/\1/p" Cargo.toml)
N=room4doom-$V-linux-x86_64
rm -rf dist /tmp/pkg && mkdir -p dist /tmp/pkg/$N
install -m 755 target/release/room4doom dist/$N
install -m 755 target/release/room4doom /tmp/pkg/$N/room4doom
install -m 644 LICENSE LICENSE-MIT /tmp/pkg/$N/
install -m 644 packaging/release/README.txt /tmp/pkg/$N/README.txt
install -m 644 .freedoom/freedoom1.wad /tmp/pkg/$N/freedoom1.wad
install -m 644 .freedoom/COPYING.txt /tmp/pkg/$N/FREEDOOM-COPYING.txt
install -m 644 .freedoom/CREDITS.txt /tmp/pkg/$N/FREEDOOM-CREDITS.txt
find /tmp/pkg -exec touch -h -d "@$SOURCE_DATE_EPOCH" {} +
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
    -C /tmp/pkg -cf - $N | gzip -n -9 > dist/$N.tar.gz
cd dist && sha256sum -- * > SHA256SUMS && cat SHA256SUMS
'
