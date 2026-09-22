//! The contract for slot2-i18n. Task 01 makes these pass without editing this file.

use std::fs;

use slot2_i18n::{josa, Arg, Button, I18n, Span};

// ---------- josa (pure) ----------

#[test]
fn josa_picks_by_final_consonant() {
    assert_eq!(josa("포켓몬", "을/를"), "을");
    assert_eq!(josa("테트리스", "을/를"), "를");
    assert_eq!(josa("포켓몬", "이/가"), "이");
    assert_eq!(josa("테트리스", "이/가"), "가");
    assert_eq!(josa("젤다", "은/는"), "는");
    assert_eq!(josa("슈퍼패미컴", "과/와"), "과");
}

#[test]
fn josa_euro_ro_has_the_rieul_exception() {
    assert_eq!(josa("슈퍼패미컴", "으로/로"), "으로");
    assert_eq!(josa("GBA", "으로/로"), "으로(로)"); // latin: undecidable
    assert_eq!(josa("메가드라이브", "으로/로"), "로"); // no 받침
    assert_eq!(josa("마스터시스템", "으로/로"), "으로"); // ㅁ 받침
    assert_eq!(josa("젤다의 전설", "으로/로"), "로"); // ㄹ 받침 → 로
    assert_eq!(josa("젤다의 전설", "을/를"), "을"); // ㄹ 받침 still counts for 을/를
}

#[test]
fn josa_reads_digits_aloud() {
    assert_eq!(josa("슬롯 1", "을/를"), "을"); // 일
    assert_eq!(josa("슬롯 2", "을/를"), "를"); // 이
    assert_eq!(josa("슬롯 3", "으로/로"), "으로"); // 삼
    assert_eq!(josa("슬롯 7", "으로/로"), "로"); // 칠 → ㄹ
    assert_eq!(josa("슬롯 10", "이/가"), "이"); // 영
}

#[test]
fn josa_falls_back_to_both_when_undecidable() {
    assert_eq!(josa("Metroid Fusion", "을/를"), "을(를)");
    assert_eq!(josa("Metroid Fusion (USA)", "을/를"), "을(를)");
    assert_eq!(josa("", "을/를"), "을(를)");
    assert_eq!(josa("포켓몬  ", "을/를"), "을"); // trailing whitespace ignored
    assert_eq!(josa("포켓몬", "을"), "을"); // no slash: unchanged
}

// ---------- bundles ----------

#[test]
fn embedded_languages_load_and_name_themselves() {
    let en = I18n::embedded("en").unwrap();
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(en.code(), "en");
    assert_eq!(en.name(), "English");
    assert_eq!(ko.name(), "한국어");
    assert_eq!(en.font(), None);
    assert_eq!(ko.font().as_deref(), Some("NotoSansKR-Regular.ttf"));
    assert!(I18n::embedded("xx").is_err());
}

#[test]
fn plain_messages_and_missing_keys() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("power-off"), "Power off");
    assert_eq!(en.t("no-such-key"), "[no-such-key]");
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("resume"), "계속하기");
}

#[test]
fn arguments_and_plurals() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(
        en.t_args("cart-inserted", &[("title", "Tetris".into())]),
        "Inserted Tetris"
    );
    assert_eq!(
        en.t_args("states-count", &[("n", 1i64.into())]),
        "1 save state"
    );
    assert_eq!(
        en.t_args("states-count", &[("n", 3i64.into())]),
        "3 save states"
    );
    assert_eq!(
        en.t_args(
            "state-saved",
            &[("title", "Tetris".into()), ("n", 2i64.into())]
        ),
        "Saved Tetris to slot 2"
    );
}

#[test]
fn no_unicode_isolation_marks_leak() {
    let en = I18n::embedded("en").unwrap();
    let s = en.t_args("cart-inserted", &[("title", "Tetris".into())]);
    assert!(!s.contains('\u{2068}') && !s.contains('\u{2069}'), "{s:?}");
}

#[test]
fn josa_function_inside_korean_messages() {
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(
        ko.t_args("cart-inserted", &[("title", "포켓몬".into())]),
        "포켓몬을 꽂았습니다"
    );
    assert_eq!(
        ko.t_args("cart-ejected", &[("title", "테트리스".into())]),
        "테트리스를 꺼냈습니다"
    );
    assert_eq!(
        ko.t_args("platform-switched", &[("platform", "슈퍼패미컴".into())]),
        "슈퍼패미컴으로 전환했습니다"
    );
    assert_eq!(
        ko.t_args("platform-switched", &[("platform", "GBA".into())]),
        "GBA으로(로) 전환했습니다"
    );
    assert_eq!(
        ko.t_args(
            "state-saved",
            &[("title", "젤다의 전설".into()), ("n", 2i64.into())]
        ),
        "2번 슬롯에 젤다의 전설을 저장했습니다"
    );
    assert_eq!(
        ko.t_args("states-count", &[("n", 3i64.into())]),
        "세이브 스테이트 3개"
    );
}

#[test]
fn numbers_format_plainly_without_grouping() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(
        en.t_args(
            "state-saved",
            &[("title", "T".into()), ("n", 1234i64.into())]
        ),
        "Saved T to slot 1234"
    );
}

#[test]
fn fallback_to_english_for_missing_korean_keys() {
    // Card pack that only defines one key, layered over the ko built-in, en underneath.
    let dir = tempdir();
    fs::write(dir.join("ko.ftl"), "power-off = 끄기\n").unwrap();
    let ko = I18n::load("ko", Some(&dir)).unwrap();
    assert_eq!(ko.t("power-off"), "끄기"); // card wins
    assert_eq!(ko.t("resume"), "계속하기"); // built-in ko still there
                                            // A card-only language falls through to en for everything it lacks.
    fs::write(dir.join("ja.ftl"), "lang-name = 日本語\n").unwrap();
    let ja = I18n::load("ja", Some(&dir)).unwrap();
    assert_eq!(ja.name(), "日本語");
    assert_eq!(ja.t("power-off"), "Power off");
    assert_eq!(ja.t("no-such-key"), "[no-such-key]");
}

#[test]
fn available_lists_builtins_plus_card_packs_sorted_unique() {
    let dir = tempdir();
    fs::write(dir.join("ja.ftl"), "lang-name = 日本語\n").unwrap();
    fs::write(dir.join("ko.ftl"), "power-off = 끄기\n").unwrap();
    fs::write(dir.join("notes.txt"), "ignored\n").unwrap();
    assert_eq!(I18n::available(Some(&dir)), vec!["en", "ja", "ko"]);
    assert_eq!(I18n::available(None), vec!["en", "ko"]);
}

#[test]
fn parse_errors_are_reported_not_swallowed() {
    let dir = tempdir();
    fs::write(dir.join("ko.ftl"), "this is not = = valid\n{{{\n").unwrap();
    let err = I18n::load("ko", Some(&dir)).unwrap_err();
    assert!(err.to_string().contains("ko.ftl"), "{err}");
}

// ---------- BTN spans ----------

#[test]
fn btn_renders_as_bracketed_label_in_plain_text() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("hint-eject"), "Hold [MENU] to eject");
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("hint-eject"), "[MENU] 길게 눌러 꺼내기");
}

#[test]
fn btn_spans_keep_the_language_order() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(
        en.spans("hint-eject", &[]),
        vec![
            Span::Text("Hold ".into()),
            Span::Btn(Button::Menu),
            Span::Text(" to eject".into())
        ]
    );
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(
        ko.spans("hint-eject", &[]),
        vec![
            Span::Btn(Button::Menu),
            Span::Text(" 길게 눌러 꺼내기".into())
        ]
    );
    assert_eq!(
        en.spans("power-off", &[]),
        vec![Span::Text("Power off".into())]
    );
}

#[test]
fn unknown_button_ids_are_visible_not_fatal() {
    let dir = tempdir();
    fs::write(
        dir.join("en.ftl"),
        "hint-x = Press { BTN(\"turbo\") } now\n",
    )
    .unwrap();
    let en = I18n::load("en", Some(&dir)).unwrap();
    assert_eq!(en.t("hint-x"), "Press [?turbo] now");
    assert_eq!(
        en.spans("hint-x", &[]),
        vec![Span::Text("Press [?turbo] now".into())]
    );
}

#[test]
fn button_ids_round_trip() {
    for b in Button::ALL {
        assert_eq!(Button::from_id(b.id()), Some(b));
        assert_eq!(Button::from_id(&b.id().to_uppercase()), Some(b));
    }
    assert_eq!(Button::from_id("turbo"), None);
    let _ = Arg::from("unused");
}

// ---------- helpers ----------

fn tempdir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-i18n-{}-{}", std::process::id(), unique()));
    fs::create_dir_all(&d).unwrap();
    d
}

fn unique() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}
