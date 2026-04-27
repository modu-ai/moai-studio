# SPEC-V3-011 Progress

**Started**: 2026-04-25
**Branch**: main (direct commit via PR #40, #41)
**SPEC status**: MS-1 IMPLEMENTED (unsigned cross-platform build infrastructure)
**Completion date**: 2026-04-27 (MS-1)

## Implementation Timeline

- 2026-04-25 PR #40 (`96723f8`): SPEC-V3-011 USER-DECISION 4 gates RESOLVED, status ready
- 2026-04-27 PR #41 (`fbcb043`): MS-1 cross-platform unsigned build infrastructure (T1~T7)

## Milestone Status

- [x] MS-1: 3-platform build + basic packaging (unsigned) — PR #41
- [ ] MS-2: Signing + notarization — **BLOCKED** (signing certificate not held, USER-DECISION-PK-B)
- [ ] MS-3: Auto-updater + release workflow + automation — deferred (follows MS-2)

## Key Files Changed

### PR #41 — MS-1 Implementation (788 LOC added)

- `.github/workflows/release.yml`: 527 LOC — GitHub Actions release workflow with 4-target build matrix (macOS arm64/x86_64, Ubuntu x86_64, Windows x86_64), `verify-version` step, platform-specific packaging, `softprops/action-gh-release@v2` draft aggregation. Triggers on `v*.*.*` tag push or `workflow_dispatch`.
- `assets/moai-studio.desktop`: Linux Desktop Entry file
- `assets/icons/256x256/moai-studio.png`: 256x256 RGB placeholder icon
- `assets/icons/README.md`: Icon replacement guide
- `scripts/build-appimage.sh`: linuxdeploy AppImage build script (set -euo pipefail)
- `wix/main.wxs`: Windows MSI definition (upgrade-guid, path-guid, stable IDs)
- `crates/moai-studio-app/Cargo.toml`: Added `[package.metadata.bundle]`, `[package.metadata.deb]`, `[package.metadata.wix]` sections

## Test Coverage

- No Rust source code changes (RG-PK-7.6 compliance — codebase unchanged)
- Verification performed:
  - `actionlint .github/workflows/release.yml`: 0 warnings
  - `bash -n scripts/build-appimage.sh`: syntax OK
  - `cargo build --release -p moai-studio-app`: 20.32s build success
  - `cargo metadata`: bundle/deb/wix JSON parsing OK
  - `file assets/icons/256x256/moai-studio.png`: confirmed 256x256 RGB PNG

## USER-DECISION Resolution

| Gate | Decision | Rationale |
|------|----------|-----------|
| PK-A (Auto-update) | (a) Custom GitHub Releases JSON + Ed25519 | Zero external deps, Rust 100-200 LOC, platform consistency |
| PK-B (Signing cert) | (b) Not held | MS-2 BLOCKED. MS-1 unsigned builds only |
| PK-C (Tag naming) | (a) `v{x.y.z}` single format | Simple regex, CLAUDE.local.md compliant |
| PK-D (Crash reporting) | (a) Opt-out (not introduced) | Privacy-first, GitHub Issues for reports |

## Known Limitations

- MS-2 BLOCKED: No Apple Developer ID or Windows EV certificate. macOS users must bypass Gatekeeper (right-click > Open). Windows users must accept SmartScreen warning.
- MS-3 deferred: Auto-updater module not yet created. No `update.json` manifest generation.
- Placeholder icon: `assets/icons/256x256/moai-studio.png` is a 564-byte placeholder, needs real branding icon.
- CI billing: `workflow_dispatch` dry-run mode is the only usable trigger until private repo billing is resolved.
- Rust source code (`crates/*/src/`) completely unchanged per RG-PK-7.6 constraint.
