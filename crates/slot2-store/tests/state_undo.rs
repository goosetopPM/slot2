//! Reversible deletion: what `take_state` and `restore_state` promise.
//!
//! Temporary directories only, and no clock, permissions or ROM anywhere: what is under test is
//! bytes on a card and where they are allowed to land.

use std::fs;
use std::path::PathBuf;

use slot2_store::{Card, Cart, Error, Platform, StateKind, Thumb};

fn tempdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("slot2-undo-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn card_and_cart(name: &str) -> (Card, Cart) {
    let d = tempdir(name);
    let card = Card::new(&d);
    card.ensure_layout();
    let cart = Cart {
        platform: Platform::Nes,
        stem: "Mappy".into(),
        title: "Mappy".into(),
        rom: d.join("Games/NES/Mappy.nes"),
    };
    (card, cart)
}

/// A real PNG beside the state, through the same writer the game uses.
fn write_with_thumb(card: &Card, cart: &Cart, kind: StateKind, state: &[u8]) {
    let rgba = vec![0x80u8; 4 * 4 * 4];
    card.write_state(
        &cart.clone(),
        kind,
        state,
        Some(Thumb {
            width: 4,
            height: 4,
            rgba: &rgba,
        }),
    )
    .unwrap();
}

fn paths(card: &Card, cart: &Cart, kind: StateKind) -> (PathBuf, PathBuf) {
    let state = card.state_path(cart, kind);
    let thumb = state.with_extension("png");
    (state, thumb)
}

#[test]
fn a_taken_state_comes_back_byte_for_byte() {
    let (card, cart) = card_and_cart("round-trip");
    write_with_thumb(&card, &cart, StateKind::Numbered(1), b"state one");
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(1));
    let state_bytes = fs::read(&state).unwrap();
    let png_bytes = fs::read(&thumb).unwrap();
    assert!(card.read_state(&cart, StateKind::Numbered(1)).is_some());

    let backup = card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .expect("the state was there");
    assert_eq!(backup.kind(), StateKind::Numbered(1));
    assert!(!state.exists(), "the state stayed on the card");
    assert!(!thumb.exists(), "the thumbnail stayed on the card");
    assert!(
        !card
            .list_states(&cart)
            .iter()
            .any(|s| s.kind == StateKind::Numbered(1)),
        "the listing still offers the slot"
    );

    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&state).unwrap(), state_bytes);
    assert_eq!(
        fs::read(&thumb).unwrap(),
        png_bytes,
        "the PNG was rewritten"
    );
    let list = card.list_states(&cart);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].kind, StateKind::Numbered(1));
    assert_eq!(list[0].thumb.as_deref(), Some(thumb.as_path()));
}

#[test]
fn a_state_with_no_thumbnail_does_not_grow_one() {
    let (card, cart) = card_and_cart("no-thumb");
    card.write_state(&cart, StateKind::Numbered(2), b"bare", None)
        .unwrap();
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(2));

    let backup = card
        .take_state(&cart, StateKind::Numbered(2))
        .unwrap()
        .expect("the state was there");
    assert!(!state.exists());
    assert!(!thumb.exists());

    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&state).unwrap(), b"bare");
    assert!(!thumb.exists(), "a restore invented a thumbnail");
    assert!(card.list_states(&cart)[0].thumb.is_none());
}

#[test]
fn a_thumbnail_that_is_not_a_png_comes_back_exactly_as_it_was() {
    let (card, cart) = card_and_cart("corrupt");
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(1));
    fs::create_dir_all(state.parent().unwrap()).unwrap();
    fs::write(&state, b"state bytes").unwrap();
    let junk = b"\x89PNG\r\n\x1a\n but not really, and not even long";
    fs::write(&thumb, junk).unwrap();

    let backup = card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .expect("the state was there");
    assert!(!thumb.exists());
    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&state).unwrap(), b"state bytes");
    assert_eq!(fs::read(&thumb).unwrap(), junk, "the bytes were rewritten");
}

#[test]
fn a_state_that_is_not_there_is_none_and_leaves_the_orphan_picture_alone() {
    let (card, cart) = card_and_cart("missing");
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(4));
    fs::create_dir_all(state.parent().unwrap()).unwrap();
    fs::write(&thumb, b"orphan").unwrap();

    assert!(
        card.take_state(&cart, StateKind::Numbered(4))
            .unwrap()
            .is_none(),
        "a state that is not there came back as one"
    );
    assert_eq!(
        fs::read(&thumb).unwrap(),
        b"orphan",
        "the orphan thumbnail was touched"
    );
    assert!(card.list_states(&cart).is_empty());
}

#[test]
fn resume_is_refused_and_left_where_it_is() {
    let (card, cart) = card_and_cart("resume");
    write_with_thumb(&card, &cart, StateKind::Resume, b"resume bytes");
    let (state, thumb) = paths(&card, &cart, StateKind::Resume);
    let state_bytes = fs::read(&state).unwrap();
    let png_bytes = fs::read(&thumb).unwrap();

    let e = card.take_state(&cart, StateKind::Resume).unwrap_err();
    assert!(matches!(e, Error::Invalid(_)), "{e}");
    assert_eq!(fs::read(&state).unwrap(), state_bytes, "resume was touched");
    assert_eq!(fs::read(&thumb).unwrap(), png_bytes, "resume was touched");
    assert_eq!(card.list_states(&cart).len(), 1);
}

#[test]
fn restoring_through_another_card_is_refused_and_writes_nothing() {
    let (card, cart) = card_and_cart("origin");
    write_with_thumb(&card, &cart, StateKind::Numbered(1), b"one");
    let backup = card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .unwrap();

    let other = Card::new(tempdir("elsewhere"));
    other.ensure_layout();
    let e = other.restore_state(&backup).unwrap_err();
    assert!(matches!(e, Error::Invalid(_)), "{e}");
    let (state, thumb) = paths(&other, &cart, StateKind::Numbered(1));
    assert!(!state.exists(), "it wrote into the wrong card");
    assert!(!thumb.exists(), "it wrote into the wrong card");

    // And the refusal did not cost the backup: the card it came from still takes it.
    card.restore_state(&backup).unwrap();
    assert!(card.read_state(&cart, StateKind::Numbered(1)).is_some());
}

#[test]
fn an_existing_state_blocks_the_restore_without_overwriting_it() {
    let (card, cart) = card_and_cart("conflict");
    card.write_state(&cart, StateKind::Numbered(1), b"old", None)
        .unwrap();
    let backup = card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .unwrap();
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(1));
    fs::write(&state, b"newer").unwrap();

    let e = card.restore_state(&backup).unwrap_err();
    assert!(matches!(e, Error::Invalid(_)), "{e}");
    assert_eq!(
        fs::read(&state).unwrap(),
        b"newer",
        "the newer state was overwritten"
    );
    assert!(!thumb.exists(), "a picture was hung on the newer state");

    // The backup is still usable once the way is clear.
    fs::remove_file(&state).unwrap();
    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&state).unwrap(), b"old");
}

#[test]
fn an_existing_thumbnail_blocks_the_restore_and_never_leaves_half_of_one() {
    let (card, cart) = card_and_cart("thumb-conflict");
    write_with_thumb(&card, &cart, StateKind::Numbered(2), b"two");
    let backup = card
        .take_state(&cart, StateKind::Numbered(2))
        .unwrap()
        .unwrap();
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(2));
    // The picture the delete left behind is what a half-finished card looks like; the restore
    // must not write its state beside it, and must not touch it.
    fs::write(&thumb, b"in the way").unwrap();

    let e = card.restore_state(&backup).unwrap_err();
    assert!(matches!(e, Error::Invalid(_)), "{e}");
    assert!(!state.exists(), "the state landed beside a strange picture");
    assert_eq!(fs::read(&thumb).unwrap(), b"in the way");

    // And the same backup is still good once the way is clear.
    fs::remove_file(&thumb).unwrap();
    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&state).unwrap(), b"two");
    assert!(
        thumb.exists(),
        "the picture did not come back with the state"
    );
}

#[test]
fn a_second_restore_after_success_is_refused_and_changes_nothing() {
    let (card, cart) = card_and_cart("twice");
    write_with_thumb(&card, &cart, StateKind::Numbered(1), b"one");
    let backup = card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .unwrap();
    card.restore_state(&backup).unwrap();
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(1));
    let state_bytes = fs::read(&state).unwrap();
    let png_bytes = fs::read(&thumb).unwrap();

    let e = card.restore_state(&backup).unwrap_err();
    assert!(matches!(e, Error::Invalid(_)), "{e}");
    assert_eq!(fs::read(&state).unwrap(), state_bytes);
    assert_eq!(fs::read(&thumb).unwrap(), png_bytes);
    assert_eq!(card.list_states(&cart).len(), 1, "a second copy appeared");
}

#[test]
fn a_state_write_that_fails_takes_its_picture_back_out() {
    let (card, cart) = card_and_cart("write-fails");
    write_with_thumb(&card, &cart, StateKind::Numbered(1), b"one");
    let backup = card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .unwrap();
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(1));

    // A directory where the state's own temporary file would go: the picture can be written
    // and the state cannot, which is the one order the restore must not leave behind.
    let mut tmp = state.clone().into_os_string();
    tmp.push(".tmp");
    let block = PathBuf::from(tmp);
    fs::create_dir_all(&block).unwrap();

    let e = card.restore_state(&backup).unwrap_err();
    assert!(matches!(e, Error::Io(..)), "{e}");
    assert!(!state.exists(), "a state landed after all");
    assert!(
        !thumb.exists(),
        "the picture outlived the state that never arrived"
    );

    // The failure did not consume the backup: it restores once the way is clear.
    fs::remove_dir_all(&block).unwrap();
    card.restore_state(&backup).unwrap();
    assert_eq!(fs::read(&state).unwrap(), b"one");
    assert!(thumb.exists(), "the picture did not come back");
}

#[test]
fn the_permanent_delete_is_still_permanent() {
    let (card, cart) = card_and_cart("permanent");
    write_with_thumb(&card, &cart, StateKind::Numbered(1), b"one");
    write_with_thumb(&card, &cart, StateKind::Numbered(3), b"three");
    card.delete_state(&cart, StateKind::Numbered(1)).unwrap();
    let (state, thumb) = paths(&card, &cart, StateKind::Numbered(1));
    assert!(!state.exists());
    assert!(!thumb.exists());
    assert!(card
        .take_state(&cart, StateKind::Numbered(1))
        .unwrap()
        .is_none());
    assert_eq!(
        card.next_state_number(&cart),
        4,
        "the number the delete freed was handed out again"
    );
}
