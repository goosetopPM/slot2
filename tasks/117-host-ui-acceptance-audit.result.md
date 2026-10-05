# Task 117 - Codex final review

## Verdict

**Audit passed (attempt 1/2), but host UI acceptance remains open.** The worker obeyed the scope, all three
prescribed real-GL tests passed, and all 17 PNG files were freshly regenerated and structurally validated.
Codex reviewed every generated image rather than accepting the machine checks as visual proof.

## Verified evidence

- `slot2-gfx` real-GL screenshot: 1 passed, 0 failed.
- Korean splash real-GL screenshot: 1 passed, 0 failed.
- Three-geometry shelf/insert real-GL screenshot test: 1 passed, 0 failed.
- All 17 expected PNGs have the reported dimensions, are nonempty, and have fresh timestamps.
- No tracked product, test, asset, documentation, workflow, manifest, or configuration file changed.
- The splash, 640x480 shelf, and 720x480 shelf have no visible text clipping or overlap. The selected cart,
  adjacent-carousel crop, port opening, and panel extension are coherent in those captured frames.

## Visual finding

The 720x720 GB shelf does not show the `A play` hint in its stable frame, while the 640x480 and 720x480 GBA
frames do. The same hint is hidden through the early square-panel insertion frames and becomes visible again
only after the tall GB cart has moved below it in the final frame. This produces a missing stable affordance
and a possible late-transition flash.

The images match the implementation: `ShelfView::draw` and `draw_insert` draw the hint before the carts, so
the taller GB cart covers the hint at its current y position. This is an actionable host-visible UI defect,
not a physical-device-only uncertainty. It blocks claiming the three-geometry shelf UI visually accepted.

The front band crossing the cart during the 0.8 insertion frames is consistent with the documented draw
order in which the cart passes behind the machine face and port trim; it is not independently classified as
a defect from these still images.

## Remaining coverage

Task117 does not render the in-game menu and children, state switcher, shelf settings/dialogs, disabled rows,
toasts/errors, or a full English/Korean traversal. Physical input, audio, Mali behavior, frame pacing, lid,
power, card mounting, and device-specific output remain user hardware checks.

## Next action

Create one focused task that fixes the square-panel/tall-cart hint occlusion without making the hint overlay
the moving cart, adds a regression assertion for stable and insertion frames, and expands host visual
evidence to the currently uncovered menu/dialog surfaces. Do not proceed to hardware acceptance or release
until the regenerated host evidence is reviewed again.
