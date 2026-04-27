# SPEC-V3-014 Progress

**Started**: 2026-04-26
**Branch**: main (single PR #29 with 3 squashed commits)
**SPEC status**: FULLY IMPLEMENTED (MS-1+MS-2+MS-3 in single PR, 12 acceptance criteria)
**Completion date**: 2026-04-26

## Implementation Timeline

- 2026-04-26 `446737c`: docs — SPEC-V3-014 Banners Surface plan
- 2026-04-26 PR #29 (`bf217f2`): All 3 milestones implemented in single PR (MS-1+MS-2+MS-3) — 2408 LOC, 727 total tests
  - MS-1: Banner trait + BannerView + BannerStack (AC-V14-1~8, AC-V14-10) — 670 tests
  - MS-2: 5 Variants (Crash/Update/Lsp/Pty/Workspace) (AC-V14-9) — 722 tests
  - MS-3: RootView integration + Mock helper API (AC-V14-11, AC-V14-12) — 727 tests
- 2026-04-26 `e3712c0`: docs — SPEC-V3-014 status draft → implemented

## Milestone Status

- [x] MS-1: Banner trait + BannerView + BannerStack — PR #29 (AC-V14-1~8, AC-V14-10)
- [x] MS-2: 5 Variants (Crash/Update/Lsp/Pty/Workspace) — PR #29 (AC-V14-9)
- [x] MS-3: RootView integration + Mock helper API — PR #29 (AC-V14-11, AC-V14-12)

## Key Files Changed

### All Milestones — Single PR (10 files, 2408 LOC)

- `crates/moai-studio-ui/src/banners/mod.rs`: 484 LOC — Severity enum (5 levels with Ord impl), auto_dismiss_secs policy (Success 5s, Info 8s, others None), BannerId (Eq + Hash for dedup), ActionButton struct (label + action_id + primary), BannerData container, `should_dismiss` pure function (REQ-V14-018), BannerStack mock helper API (`push_crash/update/lsp/pty/workspace` with `cx.notify()` integration, REQ-V14-028)
- `crates/moai-studio-ui/src/banners/banner_stack.rs`: 636 LOC — BannerStack: max 3, severity priority + FIFO, push/dismiss/tick/dedup (REQ-V14-011~016)
- `crates/moai-studio-ui/src/banners/banner_view.rs`: 426 LOC — BannerView: severity to semantic token bg color, icon, body, actions, close dismiss button (REQ-V14-006~010). Severity color tokens use inline const.
- `crates/moai-studio-ui/src/banners/variants/crash.rs`: 179 LOC — CrashBanner: severity=Critical, "Agent crashed", Reopen(primary)/Dismiss, auto_dismiss=None. `format_duration` helper (pub(crate)).
- `crates/moai-studio-ui/src/banners/variants/update.rs`: 125 LOC — UpdateBanner: severity=Info, "Update v{x.y.z} available", Update(primary)/Later, auto_dismiss=8s
- `crates/moai-studio-ui/src/banners/variants/lsp.rs`: 118 LOC — LspBanner: severity=Warning, "{server} failed to start", Configure(primary)/Dismiss, auto_dismiss=None
- `crates/moai-studio-ui/src/banners/variants/pty.rs`: 130 LOC — PtyBanner: severity=Error, "Terminal failed to spawn", Restart Terminal(primary)/Dismiss, auto_dismiss=None
- `crates/moai-studio-ui/src/banners/variants/workspace.rs`: 124 LOC — WorkspaceBanner: severity=Warning, "Workspace state corrupted", Reset Workspace(primary)/Continue, auto_dismiss=None
- `crates/moai-studio-ui/src/banners/variants/mod.rs`: 25 LOC — Variant module root
- `crates/moai-studio-ui/src/lib.rs`: 162 LOC — RootView.banner_stack field (Option<Entity<BannerStack>>), initialization in `run_app`, banner_strip rendering between TitleBar and main_body (REQ-V14-026/027)

## Test Coverage

- MS-1: 62 new tests (670 total)
- MS-2: 52 new tests (722 total)
- MS-3: 5 integration tests (727 total)
  - `banner_stack_none`, `initialized`, `empty_on_init`, `push_crash`, `push_update`
- Quality gates: clippy 0 warnings, rustfmt PASS, `cargo check --release` PASS

## Known Limitations

- Severity color tokens use inline constants rather than `design::tokens.banner` — noted as deferred to a future design token consolidation.
- `format_duration` helper in `crash.rs` is pub(crate) — identified as a common extraction candidate for REFACTOR.
- Banner actions (e.g., "Restart Terminal", "Reset Workspace") trigger mock behavior only — real backend wiring required for production.
- No plan.md exists — planning was done inline in docs commit `446737c`.
