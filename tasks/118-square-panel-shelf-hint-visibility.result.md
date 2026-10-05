# Task 118 - Codex final review

## Verdict

**Passed (attempt 1/2).** The square-panel shelf hint defect found by Task117 is fixed, the implementation is
geometry-derived rather than platform-specific, and the regenerated host evidence is visually acceptable.
No second worker call is needed.

## Reviewed change

- `ShelfView` derives the stable hint position from the actual row placements and moves the line above carts
  only when their rectangles would cover its current band. It does not branch on GB, platform name, or
  720x720.
- A stable nonempty shelf still shows the play or resume/new-game hint, centred and inside the safe area.
- `draw_insert(seat == 0.0)` keeps the stable shelf frame, including the hint position.
- Positive insert/eject travel no longer draws the pre-action hint, preventing both occlusion and the late
  reappearance seen in Task117.
- Cart travel, row placement, face/port occlusion, timing, App state and audio behavior are unchanged.

## Test and build evidence

- Focused suites: `shelf_resume` 6 passed, `insert` 28 passed, `shelf_draw` 13 passed.
- New matrices cover 84 stable cases, 42 stable/seat-zero comparisons, and 168 positive-travel cases.
- Workspace: 978 passed, 0 failed, 0 ignored; this is five above the 973-pass baseline.
- Workspace clippy completed with zero warnings.
- Device distribution completed with final line `==> done`.
- `git diff --check` passed.
- Only the three allowed production/test files changed; existing Task117 records and Codex handoff changes
  were preserved.

The Task118 wording asked one `shelf_shot` command to refresh all 17 Task117 images, although that binary
owns only 15. The worker correctly refreshed all 15 affected shelf/insert images and reported the two
unaffected GL/splash images instead of repeating unrelated successful checks. This specification mismatch
is not a worker failure and does not limit this fix.

## Visual review

- `shelf-720x720.png`: `A play` is clearly visible above the tall GB cart, centred, unclipped, and separated
  from the cart.
- `insert-720x720-00.png`: identical stable first-frame hint placement.
- `insert-720x720-{05,08,10}.png`: no hint remains or flashes back during travel.
- `640x480` and `720x480` stable hints retain their previous below-cart position; their travel frames hide
  the hint cleanly.
- The machine face and port continue to occlude the moving cart as designed.

The three-geometry shelf/hint issue is closed. Task117/118 still do not provide host visual evidence for the
in-game menu and child menus, state switcher, shelf settings/dialogs, disabled rows, or transient
toasts/errors. Those surfaces are the next host UI acceptance task. All physical-device checks remain user
work and release remains blocked.
