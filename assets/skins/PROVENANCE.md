# Cartridge and slot artwork

Ported from [brandonkowalski/slot](https://github.com/brandonkowalski/slot) (MIT), where they
live in `crates/slot-ui/assets/`. Unmodified.

Every one is a **white silhouette on transparency**: `fill="#fff"` and nothing else. That is
not an oversight — the shell colour is not in the file. The renderer rasterises the coverage,
uploads it as an alpha mask and tints it at draw time, which is what lets one drawing serve a
grey Game Pak, a black one and a clear one, and what lets a game's own shell colour reach the
shelf without a file per colour.

| file | viewBox | what it is |
|---|---|---|
| `cart.svg` | 240×135 | Game Boy Advance Game Pak, face on |
| `cart_detail.svg` | 240×135 | its moulded ribs and recesses, drawn over the shell |
| `gb_cart.svg` | 240×253 | Game Boy Game Pak — the notched shell, class A and B alike |
| `gb_cart_detail.svg` | 240×253 | its moulding |
| `gbc_cart.svg` | 240×253 | Game Boy Color Game Pak |
| `gbc_cart_detail.svg` | 240×253 | its moulding |
| `socket.svg` | 41×29.8 | the slot mouth a cart is pushed into |
| `sticker.svg` | 205.762×71.116 | the label plate a cart's artwork or printed title sits on |

## NES and Super Nintendo carts (original to SLOT2, 2026)

These four are **not** ported from anywhere. They are original geometric redraws made inside
this repository for SLOT2: measured off the physical proportions of the two cartridges as
generic shapes, with no manufacturer drawing, vector, photograph or other external asset traced
or copied, and nothing downloaded. There is no logo, wordmark or trademark text in them, and no
text at all.

| file | viewBox | what it is |
|---|---|---|
| `nes_cart.svg` | 210×270 | NES cartridge shell: tall body, short shoulder across the top, shallow bevel at the connector edge |
| `nes_cart_detail.svg` | 210×270 | its side grooves, lower grip ridge and lines, connector moulding |
| `snes_cart.svg` | 240×190 | Super Nintendo cartridge shell: wide body, rounded shoulders, tapering toward the connector |
| `snes_cart_detail.svg` | 240×190 | its shoulder groove, side grip lines, lower ridge and connector moulding |

Same rules as the ported files, with one deliberate difference. The shell files are a white
silhouette on transparency — coverage, tinted by `PlatformSkin::shell` at draw time. The detail
files are **white only**: their alpha channel is the shape, and `ShelfView` draws them over the
shell at 0.8× the shell colour. They are not the light/shadow pair `gb_cart_detail.svg` carries
(black shadow plus a white lit edge); a single channel is what the two new files assume.

`gb-cart-lineart.svg` in the original is the square-on line drawing the two Game Boy shells
were measured off. It is not shipped here because nothing draws it; go to the original if the
outlines ever need re-measuring.

## Mega Drive and Master System carts (original to SLOT2, 2026)

These four are original geometric redraws too, made inside this repository for SLOT2 in 2026: no
manufacturer drawing, vector, photograph or other external asset was traced or copied, and
nothing was downloaded. There is no logo, wordmark or trademark text in them, and no text at all.
The Mega Drive cart is the wide, low one with a stepped top edge and a short taper toward the
connector; the Master System cart is the tall one with cut top corners and a stepped connector
foot.

| file | viewBox | what it is |
|---|---|---|
| `md_cart.svg` | 250×180 | Mega Drive cartridge shell |
| `md_cart_detail.svg` | 250×180 | its top-face groove, side grip recesses and connector ridge |
| `sms_cart.svg` | 200×230 | Master System cartridge shell |
| `sms_cart_detail.svg` | 200×230 | its top groove, side rails and grip/connector ridges |

The shells are white silhouettes on transparency — coverage, tinted by `PlatformSkin::shell` at
draw time. The details are white only: their alpha channel is the shape, and `ShelfView` draws
them over the shell at 0.8× the shell colour.

All seven platforms now have a shell and a detail drawing of their own: three ported from the
original project above, four drawn here.

## Mouth trims (original to SLOT2, 2026)

Seven more files were drawn here in 2026 for the front of the machine: the trim around the
cartridge slot. They are original geometric redraws too — no manufacturer drawing, vector,
photograph or other external asset was traced or copied, and nothing was downloaded. No logo,
wordmark, trademark text, or text of any kind is in them.

| file | viewBox | what it is |
|---|---|---|
| `gb_port.svg` | 294×58 | angular side blocks, a circular fastener in each |
| `gbc_port.svg` | 294×58 | a rounded cap against the opening with two short ribs outside it |
| `gba_port.svg` | 294×58 | low wing-shaped side trim and one thin rail across the top |
| `nes_port.svg` | 264×58 | stepped side blocks and one long straight top rail |
| `snes_port.svg` | 294×58 | a rounded shoulder and a separate bottom corner rail on each side |
| `md_port.svg` | 304×58 | wide chamfered side blocks and a top rail broken over the middle |
| `sms_port.svg` | 254×58 | narrow stepped side trim and a short rail along the bottom |

Each is the width of its own mouth — the cart's width plus the slot's 14px of chrome — with 20px
of trim either side of that, and exactly the mouth's height, so the trim lines up with the slot
the cartridge really goes into. White coverage on transparency, like everything else here:
`ShelfView` rasterises it once and tints it with the machine's lip colour.

The **middle of every one is empty**, and that is load-bearing rather than decorative: the trim
is drawn over the machine's face and over the cartridge in the slot, so what shows through the
opening is the game.
