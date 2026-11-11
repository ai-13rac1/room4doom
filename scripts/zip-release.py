"""Makes a release zip: the program, the licenses, the README, and Freedoom.

    python3 scripts/zip-release.py <out.zip> <folder> <src:name> [<src:name> ...]

Each <src:name> puts the file <src> in the zip as <folder>/<name>, with the
Unix mode 755 (a program). The files have a fixed time, so the zip depends
only on its contents. The other files come from the repository and from
.freedoom/ (scripts/fetch-freedoom.sh).
"""

import sys
import zipfile

FIXED_TIME = (2026, 1, 1, 0, 0, 0)


def add(z, src, name, mode):
    info = zipfile.ZipInfo(name, FIXED_TIME)
    info.external_attr = mode << 16
    info.compress_type = zipfile.ZIP_DEFLATED
    with open(src, "rb") as f:
        z.writestr(info, f.read())


def main():
    out, folder, programs = sys.argv[1], sys.argv[2], sys.argv[3:]
    with zipfile.ZipFile(out, "w") as z:
        for item in programs:
            src, name = item.rsplit(":", 1)
            add(z, src, f"{folder}/{name}", 0o100755)
        add(z, "LICENSE", f"{folder}/LICENSE", 0o100644)
        add(z, "LICENSE-MIT", f"{folder}/LICENSE-MIT", 0o100644)
        add(z, "packaging/release/README.txt", f"{folder}/README.txt", 0o100644)
        add(z, ".freedoom/freedoom1.wad", f"{folder}/freedoom1.wad", 0o100644)
        add(z, ".freedoom/COPYING.txt", f"{folder}/FREEDOOM-COPYING.txt", 0o100644)
        add(z, ".freedoom/CREDITS.txt", f"{folder}/FREEDOOM-CREDITS.txt", 0o100644)


if __name__ == "__main__":
    main()
