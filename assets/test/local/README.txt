# Local test ROMs

Anything in this directory is picked up by `cargo test -p slot2-retro --test cores` and used
in place of the hand-built images, matched to a platform by file extension. It is ignored by
git: commercial ROMs cannot live in a repository that ships under MIT.

`$SLOT2_TEST_ROMS` does the same for a directory elsewhere, and is searched first.

Useful things to put here, in the order they pay off:

| file | what it unblocks |
|---|---|
| any `.gb`, any `.gbc` | the only way to test these two shelves at all — mGBA refuses a cartridge whose Nintendo logo does not match, so this suite cannot build one |
| a Mega Drive game that runs 320 wide and one that runs 256 | `SET_GEOMETRY` handling; a hand-built ROM never changes mode |
| a SNES game with a hi-res (512-wide) screen | the same, at the size that breaks naive scaling |
| a Mega Drive game with 6-button support | the input map that the 3-button pad does not reach |
| a game with battery save RAM, any platform | save RAM actually round-tripping through the card |

One file per platform is enough; the first by name is used.
