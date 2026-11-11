#!/usr/bin/env bash
# Downloads Freedoom 0.13.0 once and checks it, for the release archives.
#
#   scripts/fetch-freedoom.sh [dir]     # default dir: .freedoom/
#
# A freedoom.zip that is already in the directory is used when its SHA-256
# is correct; the script downloads it only when it is not there.
#
# Writes freedoom1.wad, COPYING.txt and CREDITS.txt. Freedoom is free
# software under a BSD license (COPYING.txt), so the release can carry it.
# Needs curl and Python 3 (PYTHON, default python3).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DIR=${1:-$ROOT/.freedoom}
URL=https://github.com/freedoom/freedoom/releases/download/v0.13.0/freedoom-0.13.0.zip
ZIP_SHA256=3f9b264f3e3ce503b4fb7f6bdcb1f419d93c7b546f4df3e874dd878db9688f59
WAD_SHA256=7323bcc168c5a45ff10749b339960e98314740a734c30d4b9f3337001f9e703d

mkdir -p "$DIR"
if [ ! -f "$DIR/freedoom1.wad" ]; then
  [ -f "$DIR/freedoom.zip" ] || curl -sSL -o "$DIR/freedoom.zip" "$URL"
  "${PYTHON:-python3}" -I - "$DIR" "$ZIP_SHA256" <<'PY'
import hashlib, sys, zipfile
d, want = sys.argv[1], sys.argv[2]
path = f"{d}/freedoom.zip"
got = hashlib.sha256(open(path, "rb").read()).hexdigest()
if got != want:
    sys.exit(f"freedoom.zip has SHA-256 {got}, not {want}")
z = zipfile.ZipFile(path)
for name in ("freedoom1.wad", "COPYING.txt", "CREDITS.txt"):
    open(f"{d}/{name}", "wb").write(z.read(f"freedoom-0.13.0/{name}"))
PY
  rm -f "$DIR/freedoom.zip"
fi
"${PYTHON:-python3}" -I -c '
import hashlib, sys
got = hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest()
sys.exit(0 if got == sys.argv[2] else f"freedoom1.wad has SHA-256 {got}, not {sys.argv[2]}")
' "$DIR/freedoom1.wad" "$WAD_SHA256"
echo "$DIR/freedoom1.wad"
