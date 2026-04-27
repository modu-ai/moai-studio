# SPEC-V3-DIST-001 Progress

**Started**: 2026-04-27
**Branch**: main (implementation via PR #60, docs via PRs #49, #50)
**SPEC status**: ms1-ms2-implemented (MS-3 release.yml automation not yet done)
**Completion date**: 2026-04-27

## Implementation Timeline

- 2026-04-27 PR #49 (`91c1ea1`): SPEC-V3-DIST-001 v1.0.0 draft — Distribution Channel Registration
- 2026-04-27 PR #50 (`6b53bfa`): SPEC-V3-DIST-001 v1.1.0 — USER-DECISION-DIST-A/B/C resolved (status ready)
- 2026-04-27 PR #60 (`82a11b2`): Implementation — Distribution channels + SPEC-V3-006/V3-008 combined PR

## USER-DECISION Resolution

| Gate | Decision | Rationale |
|------|----------|-----------|
| DIST-A (Homebrew channel) | (a) modu-ai/homebrew-tap custom tap | Full control, no homebrew/homebrew-cask PR overhead |
| DIST-B (Scoop channel) | (a) modu-ai/scoop-bucket custom bucket | Full control, no ScoopInstaller/Extras PR overhead |
| DIST-C (Automation) | (a) release.yml automated | Cask/scoop manifest bump on release publish |

## Milestone Status

- [x] MS-1: Distribution artifacts created (Homebrew Cask, Scoop manifest, AUR PKGBUILD, AppImage README)
- [x] MS-2: README Installation section updated with all 4 channels
- [ ] MS-3: release.yml automation for cask/scoop bump — defined in spec but not yet in release.yml

## Key Files Changed

### PR #60 — Distribution Implementation

- `dist/homebrew/Casks/moai-studio.rb`: Homebrew Cask formula stub supporting arm64 + x86_64 architectures. References modu-ai/homebrew-tap custom tap.
- `dist/scoop/moai-studio.json`: Scoop manifest for MSI installer. References modu-ai/scoop-bucket custom bucket.
- `dist/aur/PKGBUILD`: Arch Linux PKGBUILD for `moai-studio-bin` package. Standard AUR format with sha256sums placeholder.
- `dist/aur/.SRCINFO`: AUR source info metadata file
- `dist/appimage/README.md`: AppImage usage instructions (download, chmod +x, execute)
- `README.md`: Installation section updated with 4 channels:
  - macOS (Homebrew): `brew tap modu-ai/tap && brew install moai-studio`
  - Windows (Scoop): `scoop bucket add moai-studio https://github.com/modu-ai/scoop-bucket && scoop install moai-studio`
  - Arch Linux (AUR): `yay -S moai-studio-bin` or other AUR helpers
  - Linux AppImage: Direct download from GitHub Releases
  - Quarantine bypass guidance for unsigned binaries

### Additional Files in PR #60 (combined with V3-006/V3-008)

- `crates/moai-git/src/branch.rs`: 172 LOC — branches(), create_branch(), checkout()
- `crates/moai-git/src/commit.rs`: 235 LOC — stage(), unstage(), commit(), log()
- `crates/moai-git/src/diff.rs`: 163 LOC — diff_file(), diff_workdir(), parse_diff()
- `crates/moai-git/src/log.rs`: 138 LOC — diff_commit(), show_commit()
- `crates/moai-git/src/stash.rs`: 142 LOC — stash_push() (MS-3 partial)
- Various UI fixes: toolbar.rs, viewer/image.rs, wizard.rs, link.rs

## Test Coverage

- No dedicated distribution artifact tests (artifacts are static configuration files, not code)
- README installation instructions verified manually
- dist/ files are templates/stubs — real testing requires actual release artifacts

## Known Limitations

- Stub artifacts: Homebrew Cask, Scoop manifest, and AUR PKGBUILD contain placeholder URLs and checksums. They need updating with actual release artifact URLs and SHA256 hashes upon first real release.
- External repos not yet created: `modu-ai/homebrew-tap` and `modu-ai/scoop-bucket` GitHub repositories need to be created and populated.
- AUR package not yet uploaded: Requires authenticated AUR account and package submission.
- MS-3 release.yml automation: The spec defines automatic cask/scoop manifest bumping on release publish, but this step is not yet implemented in `.github/workflows/release.yml`.
- No plan.md or research.md — only spec.md exists. This was a focused distribution SPEC with narrow scope.
- Implementation was bundled with SPEC-V3-006/V3-008 in PR #60, making it harder to isolate distribution-only changes.
