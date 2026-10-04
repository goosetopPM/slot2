//! The binary's library face, so integration tests can drive the same code the app runs.

pub mod app;
pub mod overlay;
pub mod probe;
pub mod session;

/// The language the frontend should start in: the explicit override this process was given, or
/// what the card says.
///
/// The environment is read once, by the binary at boot; this is the decision itself, so the
/// precedence can be tested without touching a process-global variable. An override is used
/// exactly as it was spelled — including an empty or unknown one, which fails to load and leaves
/// the built-in English running rather than quietly falling back to the card: whoever set the
/// variable asked for a language, and a card they were not thinking about must not answer with a
/// different one.
///
/// Nothing here looks for a pack, parses a file, normalises a code or writes anything. Whether
/// the requested language can actually be spoken is `slot2_ui::UiCtx::new`'s answer, and the
/// language the frontend ends up running is the `code()` of the context it built.
pub fn requested_language(card: &slot2_store::Card, env_override: Option<&str>) -> String {
    match env_override {
        Some(value) => value.to_owned(),
        None => card.read_language(),
    }
}

/// Apply the language the player chose, once, before the frame that would draw it.
///
/// A context belongs to the loop that draws with it, so a language change is a whole context at a
/// time: build the new one, check that it really speaks the chosen language, save the choice, and
/// only then hand it over. The order is the contract — load, save, swap — because each step can
/// fail and a failure has to leave the machine as it was: the old context, the old language and a
/// card nobody half-wrote. The one thing worse than not changing the language is changing half of
/// it, which is a card asking for one language while the screen speaks another.
///
/// Returns whether the context was replaced. A frame with no request is not an error: it is what
/// almost every frame looks like.
pub fn service_language_request(
    app: &mut app::App,
    ctx: &mut slot2_ui::UiCtx,
    card_lang_dir: &std::path::Path,
) -> bool {
    let Some(requested) = app.pending_language().map(str::to_owned) else {
        return false;
    };

    // Built beside the running context, not out of it: the loop keeps drawing with the language
    // it has until this one is known to be good, so a candidate that fails costs nothing to throw
    // away. The profile and the font directories are the machine's, not the language's, and are
    // carried over rather than re-detected.
    let candidate = slot2_ui::UiCtx::new(
        ctx.profile,
        &requested,
        ctx.font_dirs.clone(),
        Some(card_lang_dir),
    );
    let effective = candidate.i18n.code().to_owned();
    if effective != requested {
        // The pack is not there, or will not parse, and what the candidate would run is English:
        // the player asked for something else, so nothing is saved and nothing is replaced.
        eprintln!(
            "slot2: language requested={requested:?} effective={effective:?}; keeping the running one"
        );
        app.language_load_failed();
        return false;
    }

    if !app.commit_language(&requested) {
        // The write did not land. The candidate is dropped with the request it was built for, and
        // the context the loop is drawing with has not been touched.
        return false;
    }

    *ctx = candidate;
    eprintln!("slot2: language requested={requested:?} effective={effective:?}");
    true
}

/// What the cores may spend on this machine.
///
/// The panel comes from the detected profile — SLOT2 runs on 640x480, 720x480 and 720x720
/// handhelds and a core configured for one of those is not configured for the others. The
/// cycles question is cruder: every device target here is an Allwinner H700, so only the
/// host build gets to claim a fast CPU.
pub fn tuning_for(profile: &slot2_platform::Profile) -> slot2_retro::Tuning {
    let (w, h) = profile.geometry.size();
    if profile.target == "host" {
        slot2_retro::Tuning {
            geometry: (w, h),
            fast_cpu: true,
        }
    } else {
        slot2_retro::Tuning::handheld((w, h))
    }
}
