# Translation guide

한국어: [docs/TRANSLATING.ko.md](TRANSLATING.ko.md)

Every string the player sees in SLOT2 comes from a Fluent `.ftl` file. There are no user-visible
strings in the code, and no sentence is ever glued together from fragments. This document is what a
translator needs to add a language or correct an existing one.

## 1. Files and how they relate

| Where | What | Completeness |
| --- | --- | --- |
| [`assets/lang/en.ftl`](../assets/lang/en.ftl) | the canonical language: the source sentence for every key | every key |
| [`assets/lang/ko.ftl`](../assets/lang/ko.ftl) | the built-in Korean pack | **the same 125 keys as English**, same variables and button placeholders |
| `<card>/System/Lang/<code>.ftl` | a language the card brings, or an override of a built-in one | **a part of the keys is enough** |

- Files are UTF-8, the extension is `.ftl`, and the name is the language code (`ja.ftl`,
  `pt-BR.ftl`). Both the extension and the name are case-sensitive.
- A card pack is layered on top of the built-in packs. Where both define a key, **the card wins**; a
  key the card lacks comes from the built-in language, and a key that is in neither comes from
  English.
- A card-only language (a code with no built-in pack) is layered over English. It may contain a part
  of the keys; the rest are shown in English. Only a **built-in** language has to be complete, like
  the built-in `ko`.
- A card can also override English itself by adding `System/Lang/en.ftl`. That layered English is
  then what every other language falls back to, so a card pack can correct the canonical text for
  its own card without touching the repository.
- Dropping `<code>.ftl` into a card makes it appear in the language picker without a reboot: the
  list is built by reading `System/Lang` again each time it is opened.

## 2. Procedure

1. Copy `assets/lang/en.ftl` to `System/Lang/<code>.ftl`. For a card that is the whole job — one file,
   no code change.
2. Set `lang-name` to **the name the language calls itself** (`日本語`, `Português`). That value is
   the row in the language picker. Do not put the English name there.
3. Translate the sentences one by one. Leave the key names, `$variables`, `{ BTN("...") }` and
   `{ JOSA(...) }` exactly as they are, and change only the words between them.
4. Run the validation commands (§7). A card pack is not covered by the built-in contract test, so on
   a card check by eye that the language appears in the picker and that no sentence is cut off.

To add a **built-in** language, put the `.ftl` in `assets/lang/` as well, add one
`(code, include_str!(...))` line to [`EMBEDDED`](../crates/slot2-i18n/src/lib.rs), and add that code to
the completeness checks in
[`pack_contract.rs`](../crates/slot2-i18n/tests/pack_contract.rs). That is a code change, so it
is reviewed together with the pack. A card-only language needs none of this.

## 3. Fonts (`lang-font`)

- `lang-font` is **one file name** for the UI body font that language prefers. It is not a path.
  - Allowed: `NotoSansKR-Regular.otf`, `MyFont.ttf`
  - Refused: `System/Fonts/MyFont.ttf`, `../MyFont.ttf`, `sub/MyFont.ttf`, `.`, `..`
  - Both `/` and `\` are refused, because Windows and Linux disagree about the separator. A name
    with more than one component is refused as well.
  - If the language has no preferred font, leave the key out. The built-in English line
    `lang-font = { "" }` is the canonical metadata deliberately left empty.
  - Nothing is corrected for you: no extension is tried on your behalf and no case is folded. The
    name is looked up exactly as the pack spells it.
- Lookup order: the card's `System/Fonts/` first, then the font directories built into the build. The
  first file that exists wins.
- That file is read the first time a glyph needs it (lazy loading). If it is missing, or the name is
  not a safe plain file name, the line is ignored and the text is drawn with the default chain
  (`OpenSans` → the Korean/CJK `Noto Sans KR`), with one line written to the log. A font problem
  never breaks the language switch or the screen.
- A font added to the repository ships with its license. The notices for the fonts already there are
  [`NotoSansKR-OFL.txt`](../assets/fonts/NotoSansKR-OFL.txt) and
  [`OpenSans-OFL.txt`](../assets/fonts/OpenSans-OFL.txt), and they are copied into
  `System/licenses/` in a release (the folder the About screen's core and font notices row points
  at). Do not add commercial fonts, or fonts whose license is unclear, to the repository.

## 4. Sentence rules

- **Sentences are whole.** The code never joins fragments, so do not contribute a fragment like
  `"saved"`; write the complete sentence into the key.
- **Variable names stay.** Renaming `$title` to `$titel` fails. Only move where the value goes.
- **Word order belongs to the language.** `Saved { $title } to slot { $n }` becomes
  `{ $n }번 슬롯에 { JOSA($title, "을/를") } 저장했습니다` in Korean. How often a variable appears, in
  which order, and where a button sits in the sentence need not match English. What must match
  between `en` and `ko` is the **set of variable names** and the **order and count of `BTN` ids**.
- **Do not lose or invent variables.** The same key in `en` and `ko` must name exactly the same
  `$variables` and the same `BTN` ids; the built-in pack contract test checks this.
- Technical identifiers are not translated: the product name (`SLOT2`), core names (`mGBA`, `gpSP`
  and the rest), paths (`System/cores`, `System/licenses`), and version, device-name and
  language-code values.

## 5. Buttons (`BTN`)

A button is a placeholder inside a sentence. The real button cap is drawn in its place.

```ftl
hint-eject = Hold { BTN("menu") } to eject
hint-eject = { BTN("menu") } 길게 눌러 꺼내기
```

- Position is the language's choice (start of the sentence, end of it, wherever it reads best).
  Swapping a button for another one, or changing how many there are, is not allowed.
- Valid ids (case-insensitive; the source list is [`button.rs`](../crates/slot2-input/src/button.rs)):

| id | cap | id | cap | id | cap |
| --- | --- | --- | --- | --- | --- |
| `a` | A | `l1` | L1 | `select` | SELECT |
| `b` | B | `r1` | R1 | `start` | START |
| `x` | X | `l2` | L2 | `menu` | MENU |
| `y` | Y | `r2` | R2 | `power` | POWER |
| `up` / `down` | ↑ / ↓ | `left` / `right` | ← / → | `vol+` / `vol-` | VOL+ / VOL− |

- An id that does not exist is drawn literally, as `[?turbo]`. That is a safety net for a typo, not
  a feature: the built-in packs are not allowed to contain one, and the test fails if they do.
- **No literal caps.** Never write `[A]`, `[MENU]` or `[L1]` by hand; always use `BTN("...")`. `[A]`
  is only the letter A in the font — it is drawn as punctuation, not as a button. (Using `[` in an
  ordinary sentence is fine; what is forbidden is spelling out a cap.)

## 6. Particles (`JOSA`) — Korean only

Korean particles change with the final sound of the word before them (받침). When a particle follows a
value that is only known at runtime, such as a game title, use `JOSA` (the rules are implemented in
[`josa.rs`](../crates/slot2-i18n/src/josa.rs)).

```ftl
cart-inserted = { JOSA($title, "을/를") } 꽂았습니다
platform-switched = { JOSA($platform, "으로/로") } 전환했습니다
cheat-load-failed = { $title }의 치트를 읽지 못했습니다
```

- The first argument must be a variable the sentence already uses (`$title`). A literal string is
  not allowed, and the pair must be one the runtime implements:
  `을/를`, `이/가`, `은/는`, `과/와`, `으로/로`, `아/야`, `이여/여`.
  `으로/로` also takes `로` after a ㄹ final, and a value ending in a digit is judged by how the
  digit is read aloud.
- A value whose final sound cannot be decided (an English title, for example) is written with both
  forms, `을(를)`. That is the intended behavior, not a bug.
- **Never attach a fixed particle to a variable.** `{ $title }을 저장했습니다` is wrong for either
  "포켓몬을" or "테트리스를". Use `JOSA`, or rewrite the sentence so it needs no particle.
  - Rewriting is always allowed when it loses no information: `{ $title }을 저장했습니다` →
    `{ $n }번 슬롯에 저장했습니다`.
- Particles that do not change shape with 받침 may follow a variable directly (`{ $title }의 치트`).
  Only the ones that change need `JOSA`: `은/는`, `이/가`, `을/를`, `으로/로`, `과/와`, `아/야`.
- The English pack never uses `JOSA`; the particle belongs to the Korean pack (the test enforces
  this).

## 7. Validation

From the repository root, after finishing a translation:

```text
cargo test -p slot2-i18n --test pack_contract
cargo test -p slot2-i18n
```

`pack_contract` checks the built-in `en` and `ko` packs:

- the two packs carry the same key set and no duplicate ids, exactly 125 keys;
- no value is empty, except the fallback pack's `lang-font`;
- every `$variable` set matches between the packs;
- every `BTN` id list matches and each id is a button the frontend knows;
- `JOSA` is used only by Korean, only with a variable the message already uses, and only with one of
  the seven supported pairs;
- no pack spells a button cap out literally;
- no pack calls a function the runtime does not provide (`BTN`, `JOSA`, `NUMBER`, `DATETIME`);
- both packs load, and every canonical key formats without the visible `[key]` marker.

Card packs are deliberately outside this test, because they may be partial: on a card, check the
picker and the screens by eye.

## 8. What happens when something goes wrong

- **A key is missing**: English is shown. A screen is never empty and no blank sentence is drawn. If
  English does not have it either, the key is drawn visibly as `[key]` — never a panic.
- **A pack does not parse**: it does not load. If the language requested at boot is that pack, the
  frontend boots in English and leaves the stored setting alone. A pack that fails to load is left
  out of the language picker, because a row nobody could switch to is a promise the frontend cannot
  keep — English is the one exception, since it is compiled into every build.
- **A switch fails while running**: the previous language is kept and the screen says so. The chosen
  language is written to the card only after the new one has really loaded, so the card never names
  a language the screen is not speaking.
- **A pack with only some keys**: that is normal. The rest are filled in from the built-in language
  and then from English.

## 9. What does not exist yet

- There is no GUI tooling, no automatic translation and no enforced glossary. This document is all
  of it.
- The language **list** (the order of rows in the picker) is a plain sort by language code, with no
  locale collation. The `System/Lang/<code>.ftl` naming rule in this document is about translation
  packs only; it is separate from the ROM filename rules.
- The authority for ROM titles and filenames is [DESIGN.md](DESIGN.md) §8–§9: titles are sorted by
  Unicode scalar value with no locale collation, no natural-number ordering and no case folding, and
  a filename keeps every character of its stem (one allowed extension is removed; no normalization,
  no sanitizing, no renaming). A translator never has to rename a ROM, and renaming one re-keys its
  save, label and state files.
- There is no Noto Sans KR subsetting script (`tools/subset-font.sh`) yet. What ships today is the
  repository's original `NotoSansKR-Regular.otf`.
- The full-Korean-on-a-real-device pass (the M5 acceptance item) is tracked separately.

## 10. Contributing

- Use the [translation form](../.github/ISSUE_TEMPLATE/translation.yml) to coordinate a new built-in
  language, or to raise a change that needs discussion. Disclose machine or AI assistance there and
  say who reviewed the result; disclosure is welcome and is not forbidden.
- A card-only pack needs no issue and no code change: write `System/Lang/<code>.ftl`, put it on the
  card and test it.
- For a built-in contribution, identify the files current source requires — the pack in
  `assets/lang/`, the line in [`EMBEDDED`](../crates/slot2-i18n/src/lib.rs), and the code in
  [`pack_contract.rs`](../crates/slot2-i18n/tests/pack_contract.rs) — run the §7 commands, and submit
  the change for review.
- A correction names the keys it changes and the context the sentence appears in. Without the
  context a translation cannot be judged.
- Record who reviewed the text and where it came from. Do not describe yourself as the author of the
  project, do not claim copyright ownership or a legal conclusion on anyone's behalf, and do not
  treat disclosure of assistance as a license grant.
- Fonts and assets need redistributable original license text. Do not include ROMs, BIOS files,
  screenshots with unlicensed game content, secrets, generated build output, or card data.
- Do not weaken a test, invent product behavior, translate technical identifiers, or change the
  English semantics just to make a translation pass: if a sentence cannot be translated as it
  stands, say so in the issue and change the canonical English with it.
