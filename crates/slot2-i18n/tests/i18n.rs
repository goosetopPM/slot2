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
    assert_eq!(ko.font().as_deref(), Some("NotoSansKR-Regular.otf"));
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
fn display_menu_messages_read_naturally_in_both_packs() {
    let en = I18n::embedded("en").unwrap();
    // The title is the in-game row the player pressed to get here, so it is that message.
    assert_eq!(en.t("ingame-display"), "Display");
    assert_eq!(en.t("display-platform-default"), "Platform default");
    assert_eq!(en.t("display-integer"), "Integer scale");
    assert_eq!(en.t("display-aspect-fit"), "Fit aspect ratio");
    assert_eq!(en.t("display-fill"), "Fill screen");

    // Exact Korean on purpose: a key the ko pack lacks falls back to English.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("ingame-display"), "화면");
    assert_eq!(ko.t("display-platform-default"), "플랫폼 기본값");
    assert_eq!(ko.t("display-integer"), "정수 배율");
    assert_eq!(ko.t("display-aspect-fit"), "화면 비율 맞춤");
    assert_eq!(ko.t("display-fill"), "화면 채우기");
}

#[test]
fn shader_menu_messages_read_naturally_in_both_packs() {
    // Both packs carry all six: a shader menu that fell back to English would read as
    // half-translated rather than as missing, and the preset names are the frontend's own.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("shader-title"), "Shader");
    assert_eq!(en.t("shader-off"), "Off");
    assert_eq!(en.t("shader-sharp-bilinear"), "Sharp bilinear");
    assert_eq!(en.t("shader-lcd3x"), "LCD 3x");
    assert_eq!(en.t("shader-zfast-crt"), "zfast CRT");
    assert_eq!(en.t("shader-scanline"), "Scanlines");
    assert_eq!(
        en.t("shader-save-failed"),
        "Could not save the shader setting"
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English. The first row has
    // no shader of its own to name, so both menus use the display menu's word for it.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("shader-title"), "셰이더");
    assert_eq!(ko.t("shader-off"), "끄기");
    assert_eq!(ko.t("shader-sharp-bilinear"), "선명한 이중선형");
    assert_eq!(ko.t("shader-lcd3x"), "LCD 3배");
    assert_eq!(ko.t("shader-zfast-crt"), "zfast CRT");
    assert_eq!(ko.t("shader-scanline"), "스캔라인");
    assert_eq!(
        ko.t("shader-save-failed"),
        "셰이더 설정을 저장하지 못했습니다"
    );
    assert_eq!(ko.t("display-platform-default"), "플랫폼 기본값");

    // Six labels for six rows: a shared message would put the same words on two of them.
    for pack in [&en, &ko] {
        let labels = [
            "shader-off",
            "shader-sharp-bilinear",
            "shader-lcd3x",
            "shader-zfast-crt",
            "shader-scanline",
        ];
        for (i, a) in labels.iter().enumerate() {
            for b in &labels[i + 1..] {
                assert_ne!(pack.t(a), pack.t(b), "{a} and {b} say the same thing");
            }
        }
    }
}

#[test]
fn overscan_menu_messages_read_naturally_in_both_packs() {
    // Both packs carry all three: a key the ko pack lacks falls back to English, and a menu
    // that mixes the two reads as half-translated rather than as missing.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("overscan-title"), "Overscan");
    assert_eq!(en.t("overscan-crop"), "Crop edges");
    assert_eq!(en.t("overscan-full"), "Show full image");
    assert_eq!(
        en.t("overscan-save-failed"),
        "Could not save the overscan setting"
    );

    // Exact Korean on purpose. The first row has no crop of its own to name, so both menus use
    // the display menu's word for it.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("overscan-title"), "오버스캔");
    assert_eq!(ko.t("overscan-crop"), "가장자리 자르기");
    assert_eq!(ko.t("overscan-full"), "전체 이미지 표시");
    assert_eq!(
        ko.t("overscan-save-failed"),
        "오버스캔 설정을 저장하지 못했습니다"
    );
    assert_eq!(ko.t("display-platform-default"), "플랫폼 기본값");

    // Three labels for three rows: a shared message would put the same words on two of them,
    // and the platform-default row is a different message again.
    for pack in [&en, &ko] {
        assert_ne!(pack.t("overscan-crop"), pack.t("overscan-full"));
        for key in ["overscan-crop", "overscan-full"] {
            assert_ne!(
                pack.t(key),
                pack.t("display-platform-default"),
                "{key} repeats the platform-default row"
            );
        }
    }
}

#[test]
fn overlay_menu_messages_read_naturally_in_both_packs() {
    // Both packs carry all three: a key the ko pack lacks falls back to English, and a menu
    // that mixes the two reads as half-translated rather than as missing.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("overlay-title"), "Overlay");
    assert_eq!(en.t("overlay-on"), "On");
    assert_eq!(en.t("overlay-off"), "Off");
    assert_eq!(
        en.t("overlay-save-failed"),
        "Could not save the overlay setting"
    );

    // Exact Korean on purpose. The first row has no choice of its own to name, so all three
    // menus use the display menu's word for it.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("overlay-title"), "오버레이");
    assert_eq!(ko.t("overlay-on"), "켜기");
    assert_eq!(ko.t("overlay-off"), "끄기");
    assert_eq!(
        ko.t("overlay-save-failed"),
        "오버레이 설정을 저장하지 못했습니다"
    );
    assert_eq!(ko.t("display-platform-default"), "플랫폼 기본값");

    // Two labels for two real choices: a shared message would put the same words on both, and
    // the platform-default row is a different message again.
    for pack in [&en, &ko] {
        assert_ne!(pack.t("overlay-on"), pack.t("overlay-off"));
        for key in ["overlay-on", "overlay-off"] {
            assert_ne!(
                pack.t(key),
                pack.t("display-platform-default"),
                "{key} repeats the platform-default row"
            );
        }
    }
}

#[test]
fn core_picker_messages_read_naturally_in_both_packs() {
    // The six names are the frontend's own words for cores it ships, not the library file
    // names, and they are proper nouns: both packs spell them the same way. The title is the
    // in-game row the player pressed to get here, so it is that same message.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("ingame-core"), "Core");
    assert_eq!(en.t("core-name-mgba"), "mGBA");
    assert_eq!(en.t("core-name-gambatte"), "Gambatte");
    assert_eq!(en.t("core-name-gpsp"), "gpSP");
    assert_eq!(en.t("core-name-fceumm"), "FCEUmm");
    assert_eq!(en.t("core-name-snes9x"), "Snes9x");
    assert_eq!(en.t("core-name-genesis-plus-gx"), "Genesis Plus GX");
    assert_eq!(en.t("core-current"), "Current");
    assert_eq!(en.t("core-picker-empty"), "No cores available");
    assert_eq!(
        en.t("core-picker-restart"),
        "Changing core restarts the game"
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English, and a picker
    // that mixes the two reads as a half-finished translation.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("ingame-core"), "코어");
    assert_eq!(ko.t("core-name-mgba"), "mGBA");
    assert_eq!(ko.t("core-name-gambatte"), "Gambatte");
    assert_eq!(ko.t("core-name-gpsp"), "gpSP");
    assert_eq!(ko.t("core-name-fceumm"), "FCEUmm");
    assert_eq!(ko.t("core-name-snes9x"), "Snes9x");
    assert_eq!(ko.t("core-name-genesis-plus-gx"), "Genesis Plus GX");
    assert_eq!(ko.t("core-current"), "현재");
    assert_eq!(ko.t("core-picker-empty"), "사용 가능한 코어가 없습니다");
    assert_eq!(
        ko.t("core-picker-restart"),
        "코어를 바꾸면 게임을 다시 시작합니다"
    );

    // A core's name is a name, not the file the frontend loads.
    for pack in [&en, &ko] {
        for key in [
            "core-name-mgba",
            "core-name-gambatte",
            "core-name-gpsp",
            "core-name-fceumm",
            "core-name-snes9x",
            "core-name-genesis-plus-gx",
        ] {
            let name = pack.t(key);
            assert!(!name.contains("libretro"), "{key} = {name}");
            assert!(!name.ends_with(".so"), "{key} = {name}");
        }
    }
}

#[test]
fn core_switch_messages_read_naturally_in_both_packs() {
    // Each failure says what is running now, not which internal step went wrong: the player
    // has one question after a core change, and it is which core they are on.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("core-no-alternatives"), "No other core for this shelf");
    assert_eq!(
        en.t("core-checkpoint-failed"),
        "Could not save the game safely, so the core was not changed"
    );
    assert_eq!(
        en.t("core-switch-failed"),
        "Could not start that core; back on the one you had"
    );
    assert_eq!(
        en.t("core-setting-save-failed"),
        "Could not save the core choice; back on the one you had"
    );
    assert_eq!(
        en.t("core-recovery-state-failed"),
        "The core is back, but the game could not be resumed"
    );
    assert_eq!(
        en.t("core-recovery-failed"),
        "Could not get the game back after the core change"
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English, and a
    // half-translated failure is worse than an obviously untranslated one.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(
        ko.t("core-no-alternatives"),
        "이 선반에서 고를 수 있는 다른 코어가 없습니다"
    );
    assert_eq!(
        ko.t("core-checkpoint-failed"),
        "게임 상태를 저장하지 못해 코어를 바꾸지 않았습니다"
    );
    assert_eq!(
        ko.t("core-switch-failed"),
        "선택한 코어를 시작하지 못해 이전 코어로 돌아갔습니다"
    );
    assert_eq!(
        ko.t("core-setting-save-failed"),
        "코어 선택을 저장하지 못해 이전 코어로 돌아갔습니다"
    );
    assert_eq!(
        ko.t("core-recovery-state-failed"),
        "코어는 되돌렸지만 플레이 위치를 이어하지 못했습니다"
    );
    assert_eq!(
        ko.t("core-recovery-failed"),
        "코어 전환 뒤 게임을 되돌리지 못했습니다"
    );
}

#[test]
fn device_menu_messages_read_naturally_in_both_packs() {
    // The title is the in-game row the player pressed to get here, and the rows are named the
    // way they are in the packs.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("ingame-device"), "Device");
    assert_eq!(en.t("device-volume"), "Volume");
    assert_eq!(en.t("device-brightness"), "Brightness");
    assert_eq!(en.t("device-blue-light"), "Blue light");
    assert_eq!(en.t("device-muted"), "Muted");
    assert_eq!(en.t("device-unavailable"), "Unavailable");
    // The number is an argument of the message, not a sentence the code glues together.
    assert_eq!(
        en.t_args("device-percent", &[("value", 42i64.into())]),
        "42%"
    );
    assert_eq!(en.t_args("device-percent", &[("value", 0i64.into())]), "0%");
    assert_eq!(
        en.t_args("device-percent", &[("value", 100i64.into())]),
        "100%"
    );
    assert_eq!(
        en.spans("hint-adjust", &[]),
        vec![
            Span::Btn(Button::Left),
            Span::Btn(Button::Right),
            Span::Text(" adjust".into())
        ]
    );
    assert_eq!(
        en.spans("hint-mute", &[]),
        vec![Span::Btn(Button::A), Span::Text(" mute".into())]
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English, and a half
    // translated screen is worse than an obviously untranslated one.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("ingame-device"), "기기");
    assert_eq!(ko.t("device-volume"), "볼륨");
    assert_eq!(ko.t("device-brightness"), "밝기");
    assert_eq!(ko.t("device-blue-light"), "블루라이트");
    assert_eq!(ko.t("device-muted"), "음소거");
    assert_eq!(ko.t("device-unavailable"), "사용 불가");
    assert_eq!(
        ko.t_args("device-percent", &[("value", 42i64.into())]),
        "42%"
    );
    assert_eq!(
        ko.spans("hint-adjust", &[]),
        vec![
            Span::Btn(Button::Left),
            Span::Btn(Button::Right),
            Span::Text(" 조절".into())
        ]
    );
    assert_eq!(
        ko.spans("hint-mute", &[]),
        vec![Span::Btn(Button::A), Span::Text(" 음소거".into())]
    );
}

#[test]
fn timezone_menu_messages_read_naturally_in_both_packs() {
    // Both packs carry all six: a key the ko pack lacks falls back to English, and a screen
    // that mixes the two reads as half-translated rather than as missing. The offset is an
    // argument of the message, so the pack owns the word in front of the number.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("timezone-title"), "Time zone");
    assert_eq!(
        en.t_args("timezone-value", &[("offset", "+09:00".into())]),
        "UTC+09:00"
    );
    assert_eq!(
        en.t_args("timezone-value", &[("offset", "-12:00".into())]),
        "UTC-12:00"
    );
    assert_eq!(
        en.t("timezone-note"),
        "Display only; the system clock stays on UTC"
    );
    // The pack writes the table's two spaces between the minutes and the hours; Fluent keeps
    // them, so the two directions read as two groups rather than as one run of numbers.
    assert_eq!(en.t("timezone-hint-adjust"), "[←][→] 15 min  [↑][↓] 1 hour");
    assert_eq!(en.t("timezone-hint-apply"), "[A] apply");
    assert_eq!(en.t("timezone-hint-cancel"), "[B] cancel");

    // Exact Korean on purpose: a key the ko pack lacks falls back to English, and a half
    // translated time zone screen is worse than an obviously untranslated one.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("timezone-title"), "시간대");
    assert_eq!(
        ko.t_args("timezone-value", &[("offset", "+09:00".into())]),
        "UTC+09:00"
    );
    assert_eq!(
        ko.t("timezone-note"),
        "표시에만 적용하며 시스템 시각은 UTC로 유지합니다"
    );
    assert_eq!(ko.t("timezone-hint-adjust"), "[←][→] 15분  [↑][↓] 1시간");
    assert_eq!(ko.t("timezone-hint-apply"), "[A] 적용");
    assert_eq!(ko.t("timezone-hint-cancel"), "[B] 취소");

    // The one failure the screen can show: the card would not take the value, so the clock went
    // back to what the card holds. Both packs carry it themselves.
    assert_eq!(
        en.t("timezone-save-failed"),
        "Could not save the time zone; restored the previous value"
    );
    assert_eq!(
        ko.t("timezone-save-failed"),
        "시간대를 저장하지 못해 이전 값으로 돌아갔습니다"
    );

    // The words stand where the pack puts them, around the buttons it named: an id a pack does
    // not know would render as `[?up]` text instead of a cap, and the two directions would be
    // indistinguishable on screen.
    assert_eq!(
        en.spans("timezone-value", &[("offset", "+09:00".into())]),
        vec![Span::Text("UTC+09:00".into())]
    );
    assert_eq!(
        en.spans("timezone-hint-apply", &[]),
        vec![Span::Btn(Button::A), Span::Text(" apply".into())]
    );
    assert_eq!(
        ko.spans("timezone-hint-cancel", &[]),
        vec![Span::Btn(Button::B), Span::Text(" 취소".into())]
    );
    for (pack, lang) in [(&en, "en"), (&ko, "ko")] {
        let adjust = pack.spans("timezone-hint-adjust", &[]);
        let buttons: Vec<Button> = adjust
            .iter()
            .filter_map(|span| match span {
                Span::Btn(b) => Some(*b),
                Span::Text(_) => None,
            })
            .collect();
        assert_eq!(
            buttons,
            vec![Button::Left, Button::Right, Button::Up, Button::Down],
            "{lang}: the adjust hint does not name the four direction buttons"
        );
        // And the two hints are different messages: one message on both lines would tell the
        // player that A and B do the same thing.
        assert_ne!(
            pack.t("timezone-hint-apply"),
            pack.t("timezone-hint-cancel")
        );
    }
}

#[test]
fn shelf_menu_messages_read_naturally_in_both_packs() {
    // Both packs carry all six: a key the ko pack lacks falls back to English, and a settings
    // menu that mixes the two reads as half-translated rather than as missing. The time zone
    // row is not here — it reuses that screen's own title, which has its own test.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("shelf-menu-title"), "Settings");
    assert_eq!(en.t("shelf-language"), "Language");
    assert_eq!(en.t("shelf-display-defaults"), "Display defaults");
    assert_eq!(en.t("shelf-boot-logo"), "Boot logo");
    assert_eq!(en.t("shelf-sync"), "Sync");
    assert_eq!(en.t("shelf-about"), "About");

    // Exact Korean on purpose: a key the ko pack lacks falls back to English.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("shelf-menu-title"), "설정");
    assert_eq!(ko.t("shelf-language"), "언어");
    assert_eq!(ko.t("shelf-display-defaults"), "기본 화면 설정");
    assert_eq!(ko.t("shelf-boot-logo"), "부팅 로고");
    assert_eq!(ko.t("shelf-sync"), "연동 모드");
    assert_eq!(ko.t("shelf-about"), "정보");

    // Six rows for six labels: a shared message would put the same words on two of them, and
    // the menu would look like it had forgotten one.
    for pack in [&en, &ko] {
        let labels = [
            "shelf-language",
            "shelf-display-defaults",
            "shelf-boot-logo",
            "shelf-sync",
            "shelf-about",
        ];
        for (i, a) in labels.iter().enumerate() {
            for b in &labels[i + 1..] {
                assert_ne!(pack.t(a), pack.t(b), "{a} and {b} say the same thing");
            }
        }
        // And the menu is not the time zone screen under another name.
        assert_ne!(pack.t("shelf-menu-title"), pack.t("timezone-title"));
    }
}

#[test]
fn about_sticker_messages_read_naturally_in_both_packs() {
    // Both packs carry all six: a key the ko pack lacks falls back to English, and a sticker
    // that mixes the two reads as half-translated rather than as missing. The version and the
    // target are the messages' own arguments, so the pack owns the words in front of them.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("about-title"), "About");
    assert_eq!(en.t("about-wordmark"), "SLOT2");
    assert_eq!(
        en.t_args("about-version", &[("version", "0.1.0".into())]),
        "Version 0.1.0"
    );
    assert_eq!(
        en.t_args("about-target", &[("target", "rgsp".into())]),
        "Device rgsp"
    );
    assert_eq!(en.t("about-license"), "SLOT2 · MIT");
    assert_eq!(
        en.t("about-notices"),
        "Core and font notices: System/licenses"
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English, and the sticker is
    // the one screen a player checks when they want to know what they are running.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("about-title"), "정보");
    assert_eq!(ko.t("about-wordmark"), "SLOT2");
    assert_eq!(
        ko.t_args("about-version", &[("version", "0.1.0".into())]),
        "버전 0.1.0"
    );
    assert_eq!(
        ko.t_args("about-target", &[("target", "rgsp".into())]),
        "기기 rgsp"
    );
    assert_eq!(ko.t("about-license"), "SLOT2 · MIT");
    assert_eq!(ko.t("about-notices"), "코어 및 글꼴 고지: System/licenses");

    // The product's own name and its licence are the same words in every language, so those two
    // are the two the packs agree on by design.
    for key in ["about-wordmark", "about-license"] {
        assert_eq!(en.t(key), ko.t(key), "{key} is fixed product information");
    }
    for key in [
        "about-title",
        "about-version",
        "about-target",
        "about-notices",
    ] {
        assert_ne!(en.t(key), ko.t(key), "{key} is not translated");
    }
}

#[test]
fn cheat_menu_messages_read_naturally_in_both_packs() {
    // Both packs carry all four keys: a language that fell back to English here would read as
    // a half-translated menu rather than as a missing one.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("cheat-empty"), "No cheats for this game");
    assert_eq!(en.t("cheat-enabled"), "On");
    assert_eq!(en.t("cheat-disabled"), "Off");
    assert_eq!(en.t("hint-cheat-toggle"), "[A] toggle");

    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("cheat-empty"), "이 게임에는 치트가 없습니다");
    assert_eq!(ko.t("cheat-enabled"), "켜짐");
    assert_eq!(ko.t("cheat-disabled"), "꺼짐");
    assert_eq!(ko.t("hint-cheat-toggle"), "[A] 켜기/끄기");
}

#[test]
fn cheat_failure_messages_name_the_game_in_both_packs() {
    // The game the file belonged to is the only useful part of either message, so both take
    // `$title` and both packs substitute it.
    let en = I18n::embedded("en").unwrap();
    assert_eq!(
        en.t_args("cheat-toggle-failed", &[("title", "Pokemon Ruby".into())]),
        "Could not change the cheat for Pokemon Ruby"
    );
    assert_eq!(
        en.t_args("cheat-load-failed", &[("title", "Pokemon Ruby".into())]),
        "Could not read the cheats for Pokemon Ruby"
    );

    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(
        ko.t_args("cheat-toggle-failed", &[("title", "포켓몬 루비".into())]),
        "포켓몬 루비의 치트를 바꾸지 못했습니다"
    );
    assert_eq!(
        ko.t_args("cheat-load-failed", &[("title", "포켓몬 루비".into())]),
        "포켓몬 루비의 치트를 읽지 못했습니다"
    );
}

#[test]
fn time_control_messages_read_naturally_in_both_packs() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("time-rewind"), "Rewind");
    assert_eq!(
        en.t_args("time-fast-forward", &[("speed", 4i64.into())]),
        "4×"
    );
    assert_eq!(
        en.t_args("time-fast-forward", &[("speed", 8i64.into())]),
        "8×",
        "the speed is the badge's, not a hard-coded four"
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("time-rewind"), "되감기");
    assert_eq!(
        ko.t_args("time-fast-forward", &[("speed", 4i64.into())]),
        "4배속"
    );
}

#[test]
fn resume_launch_messages_read_naturally_in_both_packs() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("hint-play"), "[A] play");
    assert_eq!(
        en.spans("hint-resume", &[]),
        vec![Span::Btn(Button::A), Span::Text(" resume".into())]
    );
    assert_eq!(en.t("hint-new-game"), "Hold [A] for a new game");
    assert_eq!(
        en.t("resume-load-failed"),
        "Could not resume that game; starting a new one"
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("hint-play"), "[A] 실행");
    assert_eq!(
        ko.spans("hint-resume", &[]),
        vec![Span::Btn(Button::A), Span::Text(" 이어하기".into())]
    );
    assert_eq!(ko.t("hint-new-game"), "[A] 길게 눌러 새로 시작");
    assert_eq!(
        ko.t("resume-load-failed"),
        "이어할 수 없어 새 게임을 시작합니다"
    );
}

#[test]
fn switcher_delete_and_undo_messages_read_naturally_in_both_packs() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(
        en.t_args("state-deleted", &[("n", 3i64.into())]),
        "Deleted slot 3"
    );
    assert_eq!(en.t("state-delete-failed"), "Could not delete the state");
    assert_eq!(
        en.t_args("state-restored", &[("n", 3i64.into())]),
        "Put slot 3 back"
    );
    assert_eq!(en.t("state-restore-failed"), "Could not put the state back");
    assert_eq!(en.t("undo-empty"), "Nothing to undo");
    // The hints name real buttons: an id the pack does not know renders as `[?x]` text.
    assert_eq!(
        en.spans("hint-load", &[]),
        vec![Span::Btn(Button::A), Span::Text(" load".into())]
    );
    assert_eq!(
        en.spans("hint-delete", &[]),
        vec![Span::Btn(Button::X), Span::Text(" delete".into())]
    );
    assert_eq!(
        en.spans("hint-undo", &[]),
        vec![Span::Btn(Button::Y), Span::Text(" undo".into())]
    );

    // Exact Korean on purpose: a key the ko pack lacks falls back to English.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(
        ko.t_args("state-deleted", &[("n", 3i64.into())]),
        "3번 슬롯을 지웠습니다"
    );
    assert_eq!(
        ko.t("state-delete-failed"),
        "세이브 스테이트를 지우지 못했습니다"
    );
    assert_eq!(
        ko.t_args("state-restored", &[("n", 3i64.into())]),
        "3번 슬롯을 되돌렸습니다"
    );
    assert_eq!(
        ko.t("state-restore-failed"),
        "세이브 스테이트를 되돌리지 못했습니다"
    );
    assert_eq!(ko.t("undo-empty"), "되돌릴 것이 없습니다");
    assert_eq!(
        ko.spans("hint-delete", &[]),
        vec![Span::Btn(Button::X), Span::Text(" 지우기".into())]
    );
    assert_eq!(
        ko.spans("hint-undo", &[]),
        vec![Span::Btn(Button::Y), Span::Text(" 되돌리기".into())]
    );
    assert_eq!(
        ko.spans("hint-load", &[]),
        vec![Span::Btn(Button::A), Span::Text(" 불러오기".into())]
    );
}

#[test]
fn state_switcher_messages_read_naturally_in_both_packs() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(en.t("state-switcher-title"), "Save states");
    assert_eq!(en.t_args("state-slot", &[("n", 3i64.into())]), "Slot 3");

    // Exact Korean on purpose: a key the ko pack lacks falls back to English. The empty case
    // reuses the message the quick-load chord already reports rather than adding a second
    // one that says the same thing, so it is checked here too.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(ko.t("state-switcher-title"), "세이브 스테이트");
    assert_eq!(ko.t_args("state-slot", &[("n", 3i64.into())]), "3번 슬롯");
    assert_eq!(
        ko.t("states-empty"),
        "아직 저장한 세이브 스테이트가 없습니다"
    );
}

#[test]
fn quick_state_messages_read_naturally_in_both_packs() {
    let en = I18n::embedded("en").unwrap();
    assert_eq!(
        en.t_args("state-loaded", &[("n", 3i64.into())]),
        "Loaded slot 3"
    );
    assert_eq!(en.t("states-empty"), "No save states yet");
    assert_eq!(en.t("state-save-failed"), "Could not save the state");
    assert_eq!(en.t("state-load-failed"), "Could not load the state");

    // Exact Korean on purpose: a key the ko pack lacks falls back to English, and these are
    // what the quick save and quick load chords say when they go wrong.
    let ko = I18n::embedded("ko").unwrap();
    assert_eq!(
        ko.t_args("state-loaded", &[("n", 3i64.into())]),
        "3번 슬롯을 불러왔습니다"
    );
    assert_eq!(
        ko.t("states-empty"),
        "아직 저장한 세이브 스테이트가 없습니다"
    );
    assert_eq!(
        ko.t("state-save-failed"),
        "세이브 스테이트를 저장하지 못했습니다"
    );
    assert_eq!(
        ko.t("state-load-failed"),
        "세이브 스테이트를 불러오지 못했습니다"
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
