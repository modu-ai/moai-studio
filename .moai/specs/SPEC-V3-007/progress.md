# SPEC-V3-007 Progress

**Started**: 2026-04-28
**Branch**: main (direct commit)
**SPEC status**: ms1-committed
**Completion date**: N/A (MS-2/MS-3 pending)

## Implementation Timeline

- 2026-04-28 `1b8c88e` PR #61: feat(web): SPEC-V3-007 MS-1 — WebViewSurface + wry backend skeleton
- 2026-04-28 `a65acb7`: feat(ui): SPEC-V3-007 MS-1 — WebView Surface + wry 백엔드 기반 구조 (merge conflict resolved)

## Milestone Status

- [x] MS-1: WebViewSurface trait + WryBackend struct + #[cfg(feature = "web")] gate — committed `a65acb7`
- [ ] MS-2: URL navigation + History + DevTools + sandbox — pending
- [ ] MS-3: JS bridge + Auto-detect + Persistence integration — pending

## Key Files Changed

### New Files

- `crates/moai-studio-ui/src/web/mod.rs`: WebView trait + module re-exports
- `crates/moai-studio-ui/src/web/surface.rs`: WebSurface GPUI component
- `crates/moai-studio-ui/src/web/wry_backend.rs`: WryBackend wry abstraction
- `crates/moai-studio-ui/examples/wry_spike.rs`: wry integration spike

### Modified Files

- `crates/moai-studio-ui/Cargo.toml`: wry dependency (feature-gated)
- `crates/moai-studio-ui/src/lib.rs`: #[cfg(feature = "web")] pub mod web
- `crates/moai-studio-ui/src/viewer/mod.rs`: LeafKind::Web variant

## USER-DECISION Resolutions

- Spike 0 (wry + GPUI handshake): PASSED — wry_spike.rs confirms wry builds alongside GPUI
- webview-backend-choice: wry selected (cross-platform, Rust-native)
- linux-webkit2gtk-version: deferred to MS-2 (Linux CI)
- devtools-activation-policy: deferred to MS-2
- webview-sandbox-profile: deferred to MS-3

## Notes

- Dependencies: SPEC-V3-004 (render layer) — DONE
- wry is feature-gated: cargo build -p moai-studio-ui --features web
- GPUI-only builds unaffected (no wry dependency)
