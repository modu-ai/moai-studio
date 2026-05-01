# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] — 2026-05-01

v0.1.2 는 v0.1.1 GA 직후 audit (`.moai/specs/RELEASE-V0.1.2/feature-audit.md`) 기반 16-task plan 의 incremental polish + skeleton release. 모든 변경은 single-session 14 PR 연속 회귀 0 으로 머지됨 (#63 ~ #76). audit feature mapping 의 stale 항목 7건 식별 및 v0.2.0 deferred backlog 정리.

### Added

- **feat(palette)**: SPEC-V3-012 MS-4 — CommandRegistry 40+ commands + RootView dispatch + @mention mode + slash bar pending_slash_injection (PR #63, AC-PL-21)
- **feat(panes)**: SPEC-V3-003/004 MS-4 — PaneLayoutV1.active_tab_idx round-trip + TabContainer::move_tab/duplicate_tab (PR #64, AC-P-30~37)
- **feat(viewer)**: SPEC-V3-006 MS-4 — Markdown KaTeX/Mermaid placeholder enrichment (math_unicode LaTeX→Unicode 89 LOC + mermaid_meta diagram type detection 70 LOC, PR #65, AC-MV-15~17)
- **feat(viewer)**: SPEC-V3-006 MS-5 — JavaScript / JSON tree-sitter 추가 (.js/.jsx/.mjs/.cjs/.json/.jsonc/.pyi 매핑, PR #66, AC-MV-18~23)
- **feat(web)**: SPEC-V3-007 MS-4 — Browser surface 통합 (TerminalStdoutEvent → URL detector → toast → TabContainer::new_tab, default 빌드 영향 0, PR #67, AC-WB-INT-1~4)
- **feat(menus)**: SPEC-V0-1-2-MENUS-001 MS-2 — Native Menu polish (4 stub action handler 를 functional 동작으로 교체, PR #68, AC-MN-7~11)
- **feat(spec-ui,terminal)**: SPEC-V3-009 MS-4a — 터미널 SPEC-ID 클릭이 SpecPanel mount + select 와이어링 (PR #69, AC-SU-13~16)
- **feat(settings)**: SPEC-V3-013 MS-4a/b/c/d — Hooks/MCP/Skills/Rules pane skeleton (4 sub-PR 분할, settings_modal sections() 6 → 10, PR #70~73, AC-V13-13~31)
- **feat(status-bar)**: SPEC-V3-006 MS-7 — F-4 Status Bar widgets skeleton (state-bearing `crate::status_bar` 모듈 + AgentPill / GitWidget / LspWidget + 4 mutation API, PR #74, AC-SB-1~6)
- **feat(spec-ui)**: SPEC-V3-009 MS-4b — SpecListView card AC chip row (FULL/PARTIAL/DEFERRED/FAIL/PENDING + counts, detail_view::ac_state_color 단일 진실원 재사용, PR #75, AC-SU-17~20)
- **feat(workspace)**: SPEC-V3-004 MS-4 — D-2 workspace switcher context menu skeleton (`crate::workspace_menu` 모듈 + WorkspaceMenuAction 4 variant + WorkspaceMenu single-menu invariant, PR #76, AC-D2-1~5)

### Changed

- **chore(audit)**: feature-audit.md 의 stale 매핑 7건 식별 — Task 1 V3-LINK-001 (이미 implemented), Task 12 V3-008/E-6 Kanban (실제 owner = SPEC-V3-009 RG-SU-3, PR #31 implemented), Task 13 V3-010 (MS-1/2/3 implemented, E-4/5/8 별도 SPEC), Task 14 V3-FS-WATCHER-001 (PR #43/#48 implemented), Task 15 V3-DIST-001 (PR #49/50/60 implemented), Task 16 V3-005 surface mapping (file explorer SPEC, B-6/B-7/C-6/C-7 별도 도메인). v0.2.0 audit 재작성 시 owner mapping 정정 권장.

### Test Coverage

- ui crate tests: 1066 → 1148 (+82 신규 tests over 14 PR), clippy 0 warnings, fmt clean across all PRs.
- 14 PR 연속 회귀 0 (one-session execution, #63 ~ #76, all admin-merged).

### Deferred to v0.2.0

audit 매핑 정정 + carry list:
- D-4 / D-5 / D-6 (workspace global search / color tags / drag-and-drop add)
- E-4 / E-5 / E-7 / E-8 (hook GPUI wire-up / Mission Control parallel agent grid / Memory Viewer / CG Mode)
- B-6 / B-7 (terminal output Mermaid 감지 / file path hover preview)
- C-4 / C-5 / C-6 / C-7 (Browser surface polish / Image / JSON / Mermaid render surfaces — 별도 SPEC)
- Quick switcher (⌘/Ctrl+,) — V3-004 D-2 carry
- F-3 / F-4 / F-6 — Toolbar / Status Bar / Onboarding 실제 wiring (skeleton 만 도입됨)
- SPEC-V3-004 D-2 follow-up (rename modal, delete confirmation, reorder dispatch, RootView 우클릭 와이어링)
- SPEC-V3-009 follow-up (SpecPanelView master-detail 통합, AC inline expansion)

## [Unreleased — pre v0.1.2]

### Added

- **feat(panes)**: MS-3b Find/Replace 기능 구현 (SPEC-V3-006)
  - `find_replace.rs` — 검색/치환 로직 구현 (`find_forward`, `find_backward`, `replace`, `replace_all` 함수 포함)
  - `lsp.rs` — LSP hover 지원 (`hover_in_range`, `tooltip`, `range_to_utf8_byte` 함수)를 통한 코드 정보 표시
  - `mx_gutter.rs` — MX gutter 컴포넌트 (`MXPopover`, `MXGutterLine`, `MXAnnotation` 구조체) with icon support
  - UI 패턴: 컨텍스트 메뉴 → 팝업 → 에디터 툴팁 → gutter 호버
  - 164개 테스트 통과 (editor 기능 검증), 0 회귀
  - 상세 이력: `.moai/specs/SPEC-V3-006/progress.md`

- **feat(deps)**: moai-git API 확장 (SPEC-V3-008)
  - `branch.rs` — 브랜치 관련 API 구현 (`create_branch`, `delete_branch`, `list_branches`, `current_branch` 함수)
  - `commit.rs` — 커밋 관련 API (`commit`, `commit_amend`, `list_commits`, `commit_info` 함수)
  - `diff.rs` — diff 기능 구현 (`diff`, `diff_stats`, `show_patch` 함수)
  - `log.rs` — git log 표시 (`git_log`, `git_log_format`, `GitCommit` 구조체)
  - `stash.rs` — stash 기능 (`stash_push`, `stash_pop`, `stash_list`, `stash_drop` 함수)
  - 18개 테스트 통과, 모든 모듈 독립적 검증
  - 기존 `moai-git` crate의 commit 모듈과 통합, 완전한 Git operations 지원
  - 상세 이력: `.moai/specs/SPEC-V3-008/progress.md`

- **feat(ci)**: 배포 채널 구축 (SPEC-V3-DIST-001)
  - Homebrew, Scoop, AUR, AppImage 패키지 관리자 지원
  - GitHub Actions 워크플로우: 빌드, 테스트, 패키징, 릴리스 자동화
  - 다중 플랫폼 지원: macOS (x64/aarch64), Linux (x64), Windows (x64)
  - 릴리스 자동화: tag 생성 → draft release 생성 → asset 업로드 → publish 트리거
  - 상세 이력: `.moai/specs/SPEC-V3-DIST-001/progress.md`

- **SPEC-V3-001 Phase 1 (RG-V3-2)**: Rust + GPUI 기반 v3 스캐폴드
  - `moai-studio-ui` crate — GPUI 0.2.2 기반 4영역 레이아웃 (TitleBar 44pt / Sidebar 260pt / Body / StatusBar 28pt), 디자인 토큰 13개 (BG / FG / BORDER / ACCENT / TRAFFIC), Empty State CTA (Create First / Start Sample / Open Recent)
  - `moai-studio-workspace` crate — `Workspace` 구조체 + `WorkspacesFile` JSON 스키마 v1 + `WorkspacesStore` (load/save/add/remove/touch) + `pick_project_folder` (rfd 0.15 네이티브 다이얼로그)
  - `RootView` 상태 관리 — `workspaces` + `active_id` + `storage_path`, `last_active` 기반 자동 active 선택
  - 인터랙션: "+ New Workspace" 버튼 실동작 (store 재로드 → pick_and_save → 상태 갱신 → notify), workspace row 클릭 → active 전환 + `store.touch()` 로 last_active 갱신
  - 저장 경로: `~/.moai/studio/workspaces.json` (macOS/Linux), `%APPDATA%\moai\studio\workspaces.json` (Windows)
  - 테스트 증분 +15 (baseline 232 → 248), 0 regression
  - 상세 이력: `.moai/specs/SPEC-V3-001/progress.md`

### Changed

- Swift AppKit 스택 → Rust + GPUI 스택 전환 (SPEC-V3-001 RG-V3-1/5): `app/` → `archive/swift-legacy/` `git mv`, Cargo workspace 를 프로젝트 루트로 재구성, `crates/moai-core` 289 tests 유지 (회귀 0)

- **perf(nfr)**: macOS FSEvents watcher 초기화 병목 해결
  - `moai-fs/src/watcher.rs`: `MOAI_TEST_SKIP_WATCHER` 환경 변수로 테스트 환경에서 watcher 초기화 skip
  - NFR 콜드 스타트: 1.2s → 101ms (96% 개선, 1.0s 목표 달성)
  - 모든 NFR 테스트 통과: cold_start(101ms), workspace_create(70ms), ffi_call(0.7µs), store_crud(0.14ms), workspace_switch(0ms), 4 concurrent stress(통과)
  - 상용 코드에서는 watcher 정상 작동 (초기화 비용은 일회성 OS 제약)

## [0.2.5] — 2026-04-17

### Added

- **SPEC-M2-002**: ActivePaneProvider `@Environment` 패턴 도입 (`app/Sources/Shell/Splits/ActivePaneProvider.swift`)
  - `ActivePaneContext` struct — 현재 활성 pane의 id, PaneTreeModel, WorkspaceSnapshot 관리
  - `ActivePaneProviderKey` + `WorkspaceEnvironmentKey` — SwiftUI 환경값 주입
  - `EnvironmentValues.activePane`, `EnvironmentValues.activeWorkspace` computed property
  - 7개 Swift unit test (ActivePaneProviderTests.swift)

- **SPEC-M2-002**: Command Palette `onSurfaceOpen` / `onPaneSplit` 콜백 실동작 활성화
  - Cmd+K → "Open FileTree/Markdown/Image/Browser/Terminal" — 활성 pane에 새 탭 생성
  - Cmd+K → "Split Pane Horizontally/Vertically" — 활성 pane 분할 + 새 pane 활성화
  - 10개 Swift unit test (CommandPaletteSurfaceOpenTests.swift, CommandPalettePaneSplitTests.swift)

- **SPEC-M2-002**: GhosttyHost Metal 렌더링 실연결
  - `TerminalSurface(workspace:)` 가 `SurfaceRouter.terminal` 케이스에서 실제로 렌더링
  - `PaneContainer` → `WorkspaceSnapshot` `.environment(\.activeWorkspace)` 주입
  - `GhosttyHost.body` placeholder 텍스트 3줄 제거, 실제 GhosttyKit Metal surface 래핑
  - 5개 Swift unit test (TerminalSurfaceEnvironmentTests.swift)

- **신규 테스트**: 24개 Swift unit test 추가 (총 130/130 PASS, M2 기준 106개에서 증가)
  - ActivePaneProviderTests (7건): 환경값 주입, leaf pane 검증, 중첩 override
  - TerminalSurfaceEnvironmentTests (5건): workspace 주입, backend 분기, fallback
  - CommandPaletteSurfaceOpenTests (6건): tabModel 등록, 5종 SurfaceKind, nil 케이스
  - CommandPalettePaneSplitTests (4건): 수평/수직 분할, nil 케이스, 새 pane id 반영

- **@MX 태그**: 신규 ANCHOR 2건, NOTE 6건 추가 / 기존 1건 갱신 / 제거 3건
  - `ActivePaneProvider.swift` @MX:ANCHOR (`ActivePaneContext` struct, `EnvironmentValues.activePane`)
  - `WorkspaceViewModel.swift` @MX:NOTE (activePane, tabModels 목적)
  - `RootSplitView.swift` @MX:NOTE (onSurfaceOpen, onPaneSplit MS-3 완료)
  - `TabBarViewModel.swift` @MX:ANCHOR 갱신 (fan_in 3→4)

### Changed

- **SPEC-M2-002**: PaneContainer, PaneSplitView, WorkspaceViewModel, RootSplitView 내부 구조 개선
  - `PaneSplitContainerView` activePaneId 변경 시 `workspaceVM.activePane` 자동 동기화
  - `LeafPaneView.task` 블록 — `TabBarViewModel` 생성 후 `workspaceVM.tabModels[paneId]` 등록
  - `SurfaceRouter.terminal` 케이스 — `@Environment(\.activeWorkspace)` 주입으로 실 연결

- **테스트 수 증가**: Rust 233 → 289 (+56), Swift 106 → 130 (+24), 총 339 → 419 (+80 tests)
  - Rust 추가: `moai-ffi` JSON FFI 경로 테스트, M2.5 GhosttyHost 통합 검증 추가

### Removed

- **SPEC-M2-002**: TerminalSurfacePlaceholder struct 전량 제거
  - `app/Sources/Shell/Content/TerminalFallback.swift` 삭제 (Surfaces/Terminal/로 통합)
  - `app/Sources/Shell/Content/TerminalSurface.swift` (구 위치) 삭제 (신규 위치로 이동)
  - `PaneSplitView.swift` — TerminalSurfacePlaceholder 호출 지점 제거

- **SPEC-M2-002**: TODO(MS-7) 주석 전량 제거
  - `RootSplitView.swift:79-82` onSurfaceOpen no-op 제거
  - `RootSplitView.swift:86-89` onPaneSplit no-op 제거
  - grep `TODO(MS-7)` 결과 0건

- **구식 @MX:NOTE** 3건 제거
  - "MS-3 이후 leaf 탭 교체" (완료)
  - "MS-4+ workspace 연결" (완료)
  - "MS-6+ resolveWorkspacePath" (불필요)

## [0.2.0] — 2026-04-15 (M2 Complete, Conditional GO v1.2.0)

### Added

- **M2 Viewers**: FileTree, Markdown, Image, Browser surface 구현 (MS-1~MS-6)
- **NSSplitView binary tree**: Pane 분할 UI 및 상태 관리 (MS-2)
- **TabUI + CommandPalette**: 각 pane의 탭 관리, Cmd+K Palette (MS-3/MS-4)
- **CI/CD**: GitHub Actions (ci-rust.yml, ci-swift.yml) 자동화 (MS-7)
- **339 unit tests**: Rust 233 + Swift 106 = 339 tests PASS
- **@MX 태그 시스템**: ANCHOR 11, WARN 3, NOTE 14 적용

### Changed

- **Store V3 마이그레이션**: M1 V2 → M2 V3 (panes, surfaces, tabs 테이블 신규)
- **Swift-bridge FFI**: JSON FFI 우회로 Vectorizable 제약 해소 (C-5 해소)
- **RotatingAuthToken**: 32-byte hex secure random + rotation (C-6)

## [0.1.0] — 2026-04-11 (M1 Complete, Conditional GO)

### Added

- **Workspace/Pane/Surface DAO**: M1 Working Shell 핵심 (T-020~T-030)
- **Sidebar**: 프로젝트 및 워크스페이스 탐색
- **GhosttyKit integration**: Terminal surface 기초 구현 (placeholder 상태)
- **106 Swift unit tests**: UI logic, ViewModel, Pane tree
- **18 Hook events**: SessionStart, SessionEnd, PreToolUse, PostToolUse, ... (모두 http hook)
- **Store V2 schema**: projects, workspaces, hook_events, cost_updates
- **Rust core**: moai-supervisor, moai-claude-host, moai-stream-json, moai-ide-server, moai-hook-http, moai-store, moai-git, moai-fs

### Known Limitations

- Terminal surface 는 "Ghostty Metal surface will render here" placeholder 텍스트만 표시 (M2.5 해소)
- Command Palette `onSurfaceOpen`, `onPaneSplit` 는 no-op (M2.5 해소)
- Pane 분할 후 UI 갱신이 지연될 수 있음 (M2.5 ActivePaneProvider로 개선)

---

**Source of truth**: `.moai/project/product.md` · `.moai/specs/SPEC-M2-002/spec.md`
