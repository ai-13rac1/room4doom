ROOM4DOOM: It runs Doom. The license is an NFT.

room4doom is a Doom engine in Rust. This release carries Freedoom, a free
game for Doom engines, so it is ready to play.

Start the game
--------------
- Linux: ./room4doom (needs the ALSA library, libasound2, which most
  desktops have).
- macOS: the program is not notarized. Remove the quarantine mark once:
  xattr -d com.apple.quarantine room4doom
  then start ./room4doom.

The game finds freedoom1.wad next to the program. Esc opens the menu.
"room4doom --help" shows the command-line options.

Shareware with BuildDeed
------------------------
An unregistered copy plays the first map, E1M1. A BuildDeed license (an
NFT on NEAR testnet, with three computer seats) opens every episode and
map. On the title screen:
- B opens the buy page: https://13rac1.github.io/builddeed/?product=doom
- R opens the register page for this computer. Click "Register this
  computer" there; the game shows "Registered to <account>" a few seconds
  later.

The license contract licenses.builddeed.testnet records the SHA-256 of each
official file of a release.

Files
-----
- room4doom: the game. GPL-2.0-or-later (LICENSE). It contains upstream
  room4doom by Luke Jones, MIT license (LICENSE-MIT).
- freedoom1.wad: Freedoom: Phase 1, version 0.13.0. BSD license
  (FREEDOOM-COPYING.txt); authors in FREEDOOM-CREDITS.txt.

Source code: https://github.com/ai-13rac1/room4doom
