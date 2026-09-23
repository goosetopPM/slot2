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

`gb-cart-lineart.svg` in the original is the square-on line drawing the two Game Boy shells
were measured off. It is not shipped here because nothing draws it; go to the original if the
outlines ever need re-measuring.

There is no NES, SNES, Mega Drive or Master System artwork yet. Those four shelves borrow the
Game Boy Advance skin and say so in the table, rather than pretending to a drawing that does
not exist.
