# SPEC-M2-003 Progress

**Started**: N/A
**Branch**: N/A
**SPEC status**: superseded (Swift-era SPEC, requires V3 rewrite)
**Completion date**: N/A

## Status Assessment (2026-04-28 audit)

SPEC-M2-003 was written for the **Swift architecture** (pre-v3 migration). It defines:
- `surfaces.state_json` schema expansion (Surface kind-specific JSON contracts)
- `TabBarViewModel.statePathCache` replacement with DB round-trips
- FileTree recursive expand + expansion state persistence
- Browser URL persistence
- Workspace path `@Environment` injection

All references point to Swift code paths:
- `app/Sources/Shell/Tabs/TabBarViewModel.swift` (statePathCache)
- `app/Sources/Shell/Splits/PaneSplitView.swift` (resolveWorkspacePath)
- `app/Sources/Surfaces/FileTree/FileTreeSurface.swift`
- `app/Sources/Bridge/RustCore+Generated.swift`

After the **v3 migration to Rust/GPUI**, the entire Shell + Bridge + Surface layer was rewritten. The SPEC's RG-M2-P-1 through RG-M2-P-5 requirement groups remain conceptually valid but require a complete V3 rewrite targeting:
- `crates/moai-studio-workspace/src/persistence.rs` (state_json schema)
- `crates/moai-studio-ui/src/viewer/` (surface state serialization)
- GPUI Entity state management patterns

## Recommendation

- Create a new SPEC (e.g., SPEC-V3-PERSIST-001) that redefines these requirements for the Rust/GPUI architecture
- Mark SPEC-M2-003 as superseded with a pointer to the new SPEC
- The conceptual requirements (P-5 through P-8) should be preserved in the new SPEC
