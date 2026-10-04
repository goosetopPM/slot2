//! The core-scoped state store: one core's states live in their own namespace under the game,
//! and the states a card already had can be moved into one of them once.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use slot2_store::{Card, Cart, Error, Platform, StateKind, StateNamespace, Thumb};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-core-states-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn card_with_cart(name: &str) -> (Card, Cart, PathBuf) {
    let d = tempdir(name);
    let card = Card::new(&d);
    card.ensure_layout();
    let rom = card.games_dir(Platform::Gba).join("arm.gba");
    fs::create_dir_all(rom.parent().unwrap()).unwrap();
    fs::write(&rom, b"rom").unwrap();
    let cart = Cart {
        platform: Platform::Gba,
        stem: "arm".into(),
        title: "arm".into(),
        rom,
    };
    (card, cart, d)
}

fn ns(name: &str) -> StateNamespace {
    StateNamespace::new(name).expect("a valid namespace")
}

fn thumb<'a>(rgba: &'a [u8]) -> Thumb<'a> {
    Thumb {
        width: 2,
        height: 2,
        rgba,
    }
}

#[test]
fn a_namespace_is_a_small_plain_name() {
    let long = "x".repeat(65);
    let exactly_64 = "x".repeat(64);
    for name in [
        "mgba_libretro",
        "gambatte_libretro",
        "gpsp_libretro",
        "a",
        "abc_123",
        exactly_64.as_str(),
    ] {
        assert_eq!(ns(name).as_str(), name, "{name:?} was refused");
    }

    for name in [
        "",                 // nothing at all
        long.as_str(),      // one byte too long
        "MGBA_LIBRETRO",    // uppercase
        "mgba_libretro ",   // trailing space
        "mgba libretro",    // a space inside
        "mgba-libretro",    // a hyphen is not in the alphabet
        "mgba.libretro",    // nor is a dot
        ".",                // the current directory
        "..",               // the parent directory
        "../mgba",          // traversal
        "mgba/../gambatte", // traversal in the middle
        "a/b",              // a separator
        "a\\b",             // the other separator
        "코어",             // non-ASCII
        "mgba\u{0}",        // a NUL
    ] {
        assert!(
            StateNamespace::new(name).is_err(),
            "{name:?} was accepted as a directory name"
        );
    }
}

#[test]
fn two_cores_keep_their_own_states_for_one_game() {
    let (card, cart, d) = card_with_cart("two-cores");
    let mgba = ns("mgba_libretro");
    let gpsp = ns("gpsp_libretro");

    assert!(card.scoped_list_states(&cart, &mgba).is_empty());
    assert_eq!(card.scoped_next_state_number(&cart, &mgba), 1);

    // The same game, the same slot numbers, different bytes: mGBA's serialization is not
    // gpSP's and the two must never be read as one.
    card.scoped_write_state(&cart, &mgba, StateKind::Resume, b"mgba resume", None)
        .unwrap();
    card.scoped_write_state(&cart, &mgba, StateKind::Numbered(1), b"mgba one", None)
        .unwrap();
    card.scoped_write_state(&cart, &gpsp, StateKind::Resume, b"gpsp resume", None)
        .unwrap();
    card.scoped_write_state(&cart, &gpsp, StateKind::Numbered(1), b"gpsp one", None)
        .unwrap();
    assert_eq!(
        card.scoped_read_state(&cart, &mgba, StateKind::Resume)
            .unwrap(),
        b"mgba resume"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &gpsp, StateKind::Resume)
            .unwrap(),
        b"gpsp resume"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &mgba, StateKind::Numbered(1))
            .unwrap(),
        b"mgba one"
    );
    assert_eq!(
        card.scoped_read_state(&cart, &gpsp, StateKind::Numbered(1))
            .unwrap(),
        b"gpsp one"
    );

    let kinds = |namespace: &StateNamespace| -> Vec<StateKind> {
        card.scoped_list_states(&cart, namespace)
            .iter()
            .map(|s| s.kind)
            .collect()
    };
    assert_eq!(kinds(&mgba), [StateKind::Resume, StateKind::Numbered(1)]);
    assert_eq!(kinds(&gpsp), [StateKind::Resume, StateKind::Numbered(1)]);
    assert_eq!(card.scoped_next_state_number(&cart, &mgba), 2);

    // Deleting one core's slot leaves the other's alone, and the paths are where the contract
    // says they are.
    assert_eq!(
        card.scoped_state_path(&cart, &mgba, StateKind::Numbered(1)),
        d.join("States/GBA/arm/mgba_libretro/1.state")
    );
    card.scoped_delete_state(&cart, &mgba, StateKind::Numbered(1))
        .unwrap();
    assert_eq!(
        card.scoped_read_state(&cart, &mgba, StateKind::Numbered(1)),
        None
    );
    assert_eq!(
        card.scoped_read_state(&cart, &gpsp, StateKind::Numbered(1))
            .unwrap(),
        b"gpsp one"
    );

    // And neither core's states are in the game's own flat directory.
    assert!(card.list_states(&cart).is_empty());
    assert!(!d.join("States/GBA/arm/1.state").exists());
    assert!(!d.join("States/GBA/arm/resume.state").exists());
}

#[test]
fn a_scoped_thumbnail_lives_with_its_own_core() {
    let (card, cart, d) = card_with_cart("thumbs");
    let mgba = ns("mgba_libretro");
    let gambatte = ns("gambatte_libretro");
    let rgba = vec![7u8; 16];

    card.scoped_write_state(
        &cart,
        &mgba,
        StateKind::Numbered(1),
        b"mgba",
        Some(thumb(&rgba)),
    )
    .unwrap();
    card.scoped_write_state(&cart, &gambatte, StateKind::Numbered(2), b"gambatte", None)
        .unwrap();

    let slots = card.scoped_list_states(&cart, &mgba);
    assert_eq!(slots.len(), 1);
    let picture = slots[0].thumb.clone().expect("mGBA's slot has a picture");
    assert_eq!(picture, d.join("States/GBA/arm/mgba_libretro/1.png"));
    assert!(picture.is_file());

    // The other core's namespace has no picture: it was never given one.
    let other = card.scoped_list_states(&cart, &gambatte);
    assert_eq!(other.len(), 1);
    assert!(other[0].thumb.is_none(), "{:?}", other[0].thumb);

    // Deleting takes the picture with it, and stops at the namespace's edge.
    card.scoped_delete_state(&cart, &mgba, StateKind::Numbered(1))
        .unwrap();
    assert!(!picture.exists(), "the picture outlived its state");
    assert!(card
        .scoped_read_state(&cart, &gambatte, StateKind::Numbered(2))
        .is_some());
}

#[test]
fn a_scoped_backup_goes_back_where_it_came_from() {
    let (card, cart, _d) = card_with_cart("backup");
    let mgba = ns("mgba_libretro");
    let rgba = vec![3u8; 16];
    card.scoped_write_state(
        &cart,
        &mgba,
        StateKind::Numbered(1),
        b"one",
        Some(thumb(&rgba)),
    )
    .unwrap();
    let path = card.scoped_state_path(&cart, &mgba, StateKind::Numbered(1));
    let picture = path.with_extension("png");
    let png = fs::read(&picture).unwrap();

    let backup = card
        .scoped_take_state(&cart, &mgba, StateKind::Numbered(1))
        .unwrap()
        .expect("the state was there");
    assert_eq!(backup.kind(), StateKind::Numbered(1));
    assert!(!path.exists(), "take left the state behind");
    assert!(!picture.exists(), "take left the picture behind");

    // Another card cannot be given it: the origin travels with the bytes.
    let elsewhere = Card::new(tempdir("backup-elsewhere"));
    elsewhere.ensure_layout();
    assert!(matches!(
        elsewhere.restore_state(&backup),
        Err(Error::Invalid(_))
    ));

    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"one");
    assert_eq!(
        fs::read(&picture).unwrap(),
        png,
        "the picture came back as something other than the bytes that were taken"
    );

    // A destination in the way refuses, and the backup survives for a retry.
    let again = card
        .scoped_take_state(&cart, &mgba, StateKind::Numbered(1))
        .unwrap()
        .unwrap();
    fs::write(&path, b"somebody else's").unwrap();
    assert!(matches!(card.restore_state(&again), Err(Error::Invalid(_))));
    assert_eq!(fs::read(&path).unwrap(), b"somebody else's");
    fs::remove_file(&path).unwrap();
    card.restore_state(&again).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"one");
}

#[test]
fn legacy_states_move_into_one_namespace() {
    let (card, cart, d) = card_with_cart("adopt");
    let flat = card.states_dir(&cart);
    fs::create_dir_all(&flat).unwrap();
    fs::write(flat.join("resume.state"), b"old resume").unwrap();
    fs::write(flat.join("1.state"), b"old one").unwrap();
    let rgba = vec![9u8; 16];
    card.write_state(
        &cart,
        StateKind::Numbered(2),
        b"old two",
        Some(thumb(&rgba)),
    )
    .unwrap();

    /// One slot as it was before the move: kind, bytes, timestamp, and the picture's bytes.
    type Slot = (StateKind, Vec<u8>, SystemTime, Option<Vec<u8>>);

    // What the card held before the move: kind, bytes, timestamp and the picture's own bytes.
    let before: Vec<Slot> = card
        .list_states(&cart)
        .iter()
        .map(|s| {
            (
                s.kind,
                fs::read(&s.path).unwrap(),
                s.modified,
                s.thumb.as_ref().map(|t| fs::read(t).unwrap()),
            )
        })
        .collect();
    assert_eq!(before.len(), 3, "{before:?}");

    let mgba = ns("mgba_libretro");
    assert_eq!(card.adopt_legacy_states(&cart, &mgba).unwrap(), 3);

    // Gone from the flat view, and gone from the flat directory.
    assert!(card.list_states(&cart).is_empty());
    assert!(!flat.join("resume.state").exists());
    assert!(!flat.join("2.png").exists());

    // Visible under the namespace, byte for byte and timestamp for timestamp, in the same
    // order and with the same pictures.
    let after = card.scoped_list_states(&cart, &mgba);
    assert_eq!(after.len(), 3);
    for (slot, (kind, bytes, modified, had_picture)) in after.iter().zip(before) {
        assert_eq!(slot.kind, kind);
        assert_eq!(slot.path, card.scoped_state_path(&cart, &mgba, kind));
        assert_eq!(
            fs::read(&slot.path).unwrap(),
            bytes,
            "{kind:?} lost its bytes"
        );
        assert_eq!(slot.modified, modified, "{kind:?} lost its timestamp");
        assert_eq!(
            slot.thumb.as_ref().map(|t| fs::read(t).unwrap()),
            had_picture,
            "{kind:?} lost its picture, or came back as different bytes"
        );
    }
    assert_eq!(after[0].kind, StateKind::Resume);
    assert_eq!(after[1].kind, StateKind::Numbered(1));
    assert_eq!(after[2].kind, StateKind::Numbered(2));
    assert_eq!(card.scoped_next_state_number(&cart, &mgba), 3);
    assert!(d.join("States/GBA/arm/mgba_libretro/2.png").is_file());

    // Another core's namespace saw none of it, and a second call has nothing left to do.
    assert!(card
        .scoped_list_states(&cart, &ns("gambatte_libretro"))
        .is_empty());
    assert_eq!(card.adopt_legacy_states(&cart, &mgba).unwrap(), 0);
    assert_eq!(card.scoped_list_states(&cart, &mgba).len(), 3);
}

#[test]
fn one_destination_in_the_way_stops_the_whole_move() {
    for collision in ["state", "picture"] {
        let (card, cart, _d) = card_with_cart(&format!("collide-{collision}"));
        let flat = card.states_dir(&cart);
        fs::create_dir_all(&flat).unwrap();
        fs::write(flat.join("resume.state"), b"old resume").unwrap();
        fs::write(flat.join("1.state"), b"old one").unwrap();
        let rgba = vec![5u8; 16];
        card.write_state(
            &cart,
            StateKind::Numbered(1),
            b"old one again",
            Some(thumb(&rgba)),
        )
        .unwrap();

        let mgba = ns("mgba_libretro");
        let ns_dir = card.scoped_states_dir(&cart, &mgba);
        fs::create_dir_all(&ns_dir).unwrap();
        match collision {
            // Somebody already has this slot in the namespace.
            "state" => fs::write(ns_dir.join("resume.state"), b"theirs").unwrap(),
            // Or the state is free and its picture is not.
            _ => fs::write(ns_dir.join("1.png"), b"theirs").unwrap(),
        }

        let moved = card.adopt_legacy_states(&cart, &mgba);
        assert!(matches!(moved, Err(Error::Invalid(_))), "{moved:?}");

        // Nothing moved at all: every legacy file is still flat, and the namespace has only
        // what it already had.
        assert!(flat.join("resume.state").is_file());
        assert!(flat.join("1.state").is_file());
        assert!(flat.join("1.png").is_file());
        assert_eq!(card.list_states(&cart).len(), 2);
        let their_resume = card.scoped_read_state(&cart, &mgba, StateKind::Resume);
        match collision {
            "state" => assert_eq!(their_resume.as_deref(), Some(&b"theirs"[..])),
            _ => assert_eq!(
                their_resume, None,
                "a picture collision moved the state anyway"
            ),
        }
    }
}

#[test]
fn an_orphan_picture_in_the_namespace_stops_the_move() {
    let (card, cart, d) = card_with_cart("orphan-collision");
    let flat = card.states_dir(&cart);
    fs::create_dir_all(&flat).unwrap();
    // No picture beside the flat state: the collision is the namespace's alone.
    fs::write(flat.join("resume.state"), b"old resume").unwrap();
    fs::write(flat.join("1.state"), b"old one").unwrap();

    let mgba = ns("mgba_libretro");
    let ns_dir = card.scoped_states_dir(&cart, &mgba);
    fs::create_dir_all(&ns_dir).unwrap();
    fs::write(ns_dir.join("1.png"), b"somebody else's picture").unwrap();

    let flat_before: Vec<Option<Vec<u8>>> = ["resume.state", "1.state"]
        .iter()
        .map(|name| fs::read(flat.join(name)).ok())
        .collect();

    let moved = card.adopt_legacy_states(&cart, &mgba);
    assert!(
        matches!(moved, Err(Error::Invalid(_))),
        "a state was paired with another source's picture: {moved:?}"
    );

    // Nothing moved: every flat target is byte for byte where it was, and the picture that was
    // in the way is untouched.
    for (name, before) in ["resume.state", "1.state"].iter().zip(flat_before) {
        assert_eq!(fs::read(flat.join(name)).ok(), before, "{name} moved");
    }
    assert_eq!(
        fs::read(ns_dir.join("1.png")).unwrap(),
        b"somebody else's picture"
    );
    assert!(!ns_dir.join("resume.state").exists());
    assert!(!ns_dir.join("1.state").exists());
    assert_eq!(card.list_states(&cart).len(), 2);
    assert!(d.join("States/GBA/arm/1.state").is_file());

    // With the orphan picture gone the same move goes through, which is what says the refusal
    // was about that picture and not about the states themselves.
    fs::remove_file(ns_dir.join("1.png")).unwrap();
    assert_eq!(card.adopt_legacy_states(&cart, &mgba).unwrap(), 2);
    assert_eq!(fs::read(ns_dir.join("1.state")).unwrap(), b"old one");
    assert!(
        !ns_dir.join("1.png").exists(),
        "a picture appeared for a state that never had one"
    );
}

#[test]
fn a_path_that_is_not_a_directory_is_an_error_not_a_no_op() {
    let (card, cart, _d) = card_with_cart("not-a-directory");
    let flat = card.states_dir(&cart);
    // A regular file where the game's state directory belongs: `exists()` is true and the
    // directory cannot be read, which is a card this call cannot answer for.
    fs::create_dir_all(flat.parent().unwrap()).unwrap();
    fs::write(&flat, b"not a folder").unwrap();

    let mgba = ns("mgba_libretro");
    let moved = card.adopt_legacy_states(&cart, &mgba);
    assert!(
        matches!(moved, Err(Error::Io(_, _))),
        "a file where the state directory belongs was reported as {moved:?}"
    );
    assert!(flat.is_file(), "the blocking file was replaced");
    assert_eq!(fs::read(&flat).unwrap(), b"not a folder");
    assert!(!card.scoped_states_dir(&cart, &mgba).exists());
}

#[test]
fn nothing_to_move_makes_no_directory() {
    let (card, cart, _d) = card_with_cart("nothing");
    let mgba = ns("mgba_libretro");

    // No flat directory at all.
    assert_eq!(card.adopt_legacy_states(&cart, &mgba).unwrap(), 0);
    assert!(!card.scoped_states_dir(&cart, &mgba).exists());

    // A directory with nothing recognisable in it: an orphan picture, another extension, a
    // zero, an overflowed number, a signed one, a non-canonical one, a subdirectory, and a
    // namespace-shaped directory with a state of its own.
    let flat = card.states_dir(&cart);
    fs::create_dir_all(&flat).unwrap();
    fs::write(flat.join("orphan.png"), b"no state beside me").unwrap();
    fs::write(flat.join("notes.txt"), b"not a state").unwrap();
    fs::write(flat.join("0.state"), b"zero").unwrap();
    fs::write(flat.join("4294967296.state"), b"one past u32").unwrap();
    fs::write(flat.join("-1.state"), b"signed").unwrap();
    fs::write(flat.join("01.state"), b"not canonical").unwrap();
    fs::create_dir_all(flat.join("subdir.state")).unwrap();
    let inner = flat.join("gambatte_libretro");
    fs::create_dir_all(&inner).unwrap();
    fs::write(inner.join("1.state"), b"already scoped").unwrap();

    assert_eq!(card.adopt_legacy_states(&cart, &mgba).unwrap(), 0);
    assert!(!card.scoped_states_dir(&cart, &mgba).exists());
    for kept in [
        "orphan.png",
        "notes.txt",
        "0.state",
        "4294967296.state",
        "-1.state",
        "01.state",
        "subdir.state",
        "gambatte_libretro/1.state",
    ] {
        assert!(flat.join(kept).exists(), "{kept} was moved or removed");
    }
    assert_eq!(fs::read(inner.join("1.state")).unwrap(), b"already scoped");
}
