//! Korean particle selection. Given the word a particle follows and a candidate pair such as
//! `"을/를"`, return the right one for that word's final sound.
//!
//! Rules (docs/DESIGN.md §8):
//! - Final character is a Hangul syllable (U+AC00..=U+D7A3): `(cp - 0xAC00) % 28` is the
//!   final-consonant (받침) index; 0 means none. Pairs choose the first candidate when
//!   there is a 받침 and the second when there is not — except `으로/로`, which also takes
//!   `로` when the 받침 is ㄹ (index 8).
//! - Final character is an ASCII digit: judged by how the digit is read aloud. 0 영, 1 일,
//!   3 삼, 6 육, 7 칠, 8 팔 have a 받침; 2 이, 4 사, 5 오, 9 구 do not. (For `으로/로`, 1 일,
//!   7 칠 and 8 팔 end in ㄹ → `로`; 0, 3, 6 → `으로`.)
//! - Anything else (Latin, symbols, empty, closing bracket) is undecidable: return the pair
//!   written as `첫째(둘째)`, e.g. `을(를)`, which is the convention Korean software uses.
//! - Trailing whitespace is ignored when finding the final character. A trailing `)` or
//!   `]` is treated as undecidable (it usually closes a region tag like `(USA)`), not skipped.
//!
//! `pair` is always `"<받침 있음>/<받침 없음>"`. A `pair` without `/` is returned unchanged.

/// `josa("포켓몬", "을/를") == "을"`, `josa("테트리스", "을/를") == "를"`,
/// `josa("Metroid", "을/를") == "을(를)"`.
pub fn josa(word: &str, pair: &str) -> String {
    let (first, second) = match pair.split_once('/') {
        Some((f, s)) => (f, s),
        None => return pair.to_string(),
    };

    let trimmed = word.trim_end();
    let last_char = trimmed.chars().last();

    enum Batchim {
        None,
        Has,
        Rieul,
        Undecidable,
    }

    let batchim = match last_char {
        Some(c) if ('\u{AC00}'..='\u{D7A3}').contains(&c) => {
            let cp = c as u32;
            let b = (cp - 0xAC00) % 28;
            match b {
                0 => Batchim::None,
                8 => Batchim::Rieul,
                _ => Batchim::Has,
            }
        }
        Some('0') | Some('3') | Some('6') => Batchim::Has,
        Some('1') | Some('7') | Some('8') => Batchim::Rieul,
        Some('2') | Some('4') | Some('5') | Some('9') => Batchim::None,
        _ => Batchim::Undecidable,
    };

    if pair == "으로/로" {
        match batchim {
            Batchim::None | Batchim::Rieul => second.to_string(),
            Batchim::Has => first.to_string(),
            Batchim::Undecidable => format!("{first}({second})"),
        }
    } else {
        match batchim {
            Batchim::None => second.to_string(),
            Batchim::Has | Batchim::Rieul => first.to_string(),
            Batchim::Undecidable => format!("{first}({second})"),
        }
    }
}

/// The word with its particle attached: `"포켓몬" + "을"`. What `JOSA($x, "을/를")` in a
/// message expands to.
pub fn attach(word: &str, pair: &str) -> String {
    let mut s = word.to_owned();
    s.push_str(&josa(word, pair));
    s
}
