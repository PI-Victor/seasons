# Audio Sync Algorithm Upgrade (Live Doc)

Started: 2026-04-09T12:49:33Z  
Status: Complete

## Goal
Improve Hue entertainment audio sync so it feels closer to polished apps:
- stronger, faster peak response
- better frequency-to-intensity mapping (bass body + mids/snares drive main pulse)
- cleaner transients without random flicker
- better brightness utilization under configured brightness ceiling

## Scope
Backend DSP/render pipeline only:
- `src-tauri/src/audio/analysis.rs`
- `src-tauri/src/audio/sync.rs`

No UI redesign in this implementation pass.

## Phased Plan
- [x] Phase 0: Create live doc and implementation checklist
- [x] Phase 1: Add mel-style band extraction and transient helpers in analyzer
- [x] Phase 2: Add adaptive gain control (AGC-like) stage in sync loop
- [x] Phase 3: Rework intensity composition for fast top-out and controlled decay
- [x] Phase 4: Tune color-band routing for low+mids pulse priority
- [x] Phase 5: Add/update tests and run focused checks
- [x] Phase 6: Commit in small logical units

## Acceptance Criteria
- Audio sync reaches visible peaks faster on kick/snare events.
- Idle/noise floor does not produce constant flicker.
- Mids/percussion materially influence flashes, not just treble.
- Brightness ceiling remains enforced.
- Existing tests pass and new behavior is covered with focused tests.

## Progress Log
- 2026-04-09T12:49:33Z: Initialized live doc and phased plan.
- 2026-04-09T12:53:10Z: Phase 1 implemented in `analysis.rs` with mel-style triangular filterbank + mel-band spectral flux.
- 2026-04-09T12:54:20Z: Focused test run blocked by environment (`No space left on device` while writing target artifacts).
- 2026-04-09T13:08:40Z: Upgraded project crate editions to Rust 2024 (`Cargo.toml`, `src-tauri/Cargo.toml`).
- 2026-04-09T13:11:12Z: Phase 2 implemented with AGC stage (`AdaptiveGain`) on steady/transient/main pulse drivers.
- 2026-04-09T13:12:58Z: Phase 3 implemented with faster top-out shaping and stronger transient flash composition while keeping brightness ceiling clamp.
- 2026-04-09T13:13:44Z: Phase 4 tuned around bass+mid pulse priority with transient lift bias in motion/flash drivers.
- 2026-04-09T13:14:29Z: Focused validation passed:
  - `RUSTC_WRAPPER= cargo check -p seasons --lib`
  - `RUSTC_WRAPPER= cargo check -p seasons-ui`
  - `RUSTC_WRAPPER= cargo test -p seasons audio::analysis::tests -- --nocapture`
  - `RUSTC_WRAPPER= cargo test -p seasons audio::sync::tests -- --nocapture`
- 2026-04-09T13:18:30Z: Phase 6 completed with two logical commits (`build` edition bump + `feat(audio-sync)` DSP upgrade).
