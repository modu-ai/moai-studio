# SPEC-M2-002 Compact: M2.5 Polish

> id: SPEC-M2-002 | version: 1.0.0 | status: draft | priority: High | issue: 0

M2 (SPEC-M2-001, completed 2026-04-15) 잔존 placeholder 4건 (P-1~P-4) 해소. swift-bridge FFI 변경 0, regression 0 (339 테스트 유지) + 신규 ≥15 테스트.

---

## 요구사항 요약

### RG-M2.5-1: ActivePaneProvider `@Environment` (P-4, 선행)
- `app/Sources/Shell/Splits/ActivePaneProvider.swift` 신설
- `ActivePaneContext` struct: `paneId: Int64?`, `model: PaneTreeModel?`, `workspace: WorkspaceSnapshot?`
- `ActivePaneProviderKey: EnvironmentKey` + `EnvironmentValues.activePane`
- `WorkspaceEnvironmentKey: EnvironmentKey` + `EnvironmentValues.activeWorkspace`
- `WorkspaceViewModel.activePane: ActivePaneContext` `@Observable` 프로퍼티 추가 (Command Palette 오버레이 경로)
- `PaneSplitContainerView` activePaneId 변경 시 workspaceVM 동기화 + 하위 환경값 주입
- 활성 pane 은 반드시 leaf (split 노드 금지)

### RG-M2.5-2: TerminalSurface GhosttyHost 실연결 (P-1)
- `TerminalSurfacePlaceholder` struct + 호출 지점 전량 제거 (`PaneSplitView.swift`)
- `SurfaceRouter.terminal` 케이스를 `TerminalSurface(workspace:)` 로 교체, `@Environment(\.activeWorkspace)` 소비
- `PaneContainer` 가 `.environment(\.activeWorkspace, snapshot)` 주입
- `GhosttyHost.body` 의 placeholder 텍스트 3줄 제거, 실 GhosttyKit Metal surface 래퍼 교체
- `MOAI_TERMINAL_BACKEND=nstext` 시 `TerminalFallback` 유지, 초기화 실패 시 `onFailure()` → fallback
- GhosttyKit 부재 시 크래시 금지 (기존 WARN 격리 정책 유지)

### RG-M2.5-3: Command Palette `onSurfaceOpen` 콜백 (P-2)
- `RootSplitView.setupPaletteController` 의 `onSurfaceOpen: { _ in }` + `TODO(MS-7)` 주석 제거
- `workspaceVM.activePane.paneId` + `workspaceVM.tabModels[paneId]` 참조하여 `TabBarViewModel.newTab(kind:)` 호출
- 5종 SurfaceKind 지원: `.filetree`, `.markdown`, `.image`, `.browser`, `.terminal`
- M3+ 이월 kind (`.code`, `.agentRun`, `.kanban`, `.memory`, `.instructionsGraph`) 는 `NotYetImplementedSurface` 폴백
- 활성 pane 없을 시 no-op + info 로그
- `.markdown` / `.image` 는 statePath 없이 EmptyState 렌더 (파일 선택은 M3 이월)

### RG-M2.5-4: Command Palette `onPaneSplit` 콜백 (P-3)
- `onPaneSplit: { _ in }` + `TODO(MS-7)` 주석 제거
- `workspaceVM.activePane.model.splitActive(paneId, direction:)` 호출 (키보드 단축키 Cmd+\ 경로와 동일 시퀀스)
- `PaneSplitDirection → SplitKind` 매핑
- 새 leaf pane id 로 `activePaneId` 업데이트 (단축키 경로와 동일)
- 활성 pane 없을 시 no-op, 크래시 금지

---

## 핵심 Acceptance Criteria

| AC | 시나리오 | 기준 |
|----|----------|------|
| AC-1.1 | ActivePaneContext 기본값 | 환경 미주입 시 모든 필드 nil, `ActivePaneContext.empty` 와 동일 |
| AC-1.2 | 환경값 주입 후 조회 | `.environment(\.activePane, ctx)` 주입 후 자식이 동일 ctx 관찰 |
| AC-1.3 | 중첩 override | 가장 안쪽 `.environment` 가 이긴다 |
| AC-1.4 | activePaneId 변경 시 동기화 | workspaceVM.activePane.paneId 업데이트 + 환경값 반영 |
| AC-1.6 | split 노드는 활성 불가 | DEBUG 에서 assertion, RELEASE 에서 무시 |
| AC-2.1 | 신규 워크스페이스 → 실 Metal surface | placeholder 텍스트 0건, 실제 GhosttyKit 렌더 |
| AC-2.3 | nstext 백엔드 | `TerminalFallback` 전용 표시, GhosttyKit 미초기화 |
| AC-2.4 | 초기화 실패 | onFailure → loadFailed=true → 폴백 |
| AC-3.1 | FileTree 명령 | `tabModels[activePaneId]!.newTab(.filetree)` 호출, 탭 활성화 |
| AC-3.4 | 활성 pane nil 시 | no-op + info 로그, 크래시 없음 |
| AC-4.1 | 수평 분할 | `splitActive(paneId, .horizontal)` 호출, activePaneId 업데이트 |
| AC-4.4 | 팔레트/단축키 동일성 | 두 경로 결과 pane tree + DB 완전 동일 |
| AC-G.1 | Rust regression | `cargo test --workspace` 233/233 PASS |
| AC-G.2 | Swift regression | 106 + 신규 ≥15 = 121+ PASS |
| AC-G.4 | 레이아웃 복원 | 앱 재시작 후 pane/tab/surface 100% 복원 |

---

## Exclusions (12건)

1. FileTree expand 재귀 리스팅 — M3
2. MarkdownSurface KaTeX/Mermaid 오프라인 번들 — M3
3. BrowserSurface statePath 영속화 — M3
4. statePathCache DB 영속화 — M3
5. C-2 Claude CLI AC-4.1 E2E CI 자동화 — M3+
6. C-3 10분 4-ws stress CI 자동화 — M3+
7. C-4 Metal 60fps 실측 전체 측정 — 별도 스프린트
8. Code/AgentRun/Kanban/Memory/InstructionsGraph Surface — M3~M5
9. Cross-pane 탭 drag-and-drop — M3+
10. ActivePaneProvider 기반 surface-to-surface 이벤트 버스 — M3
11. Command Palette 히스토리 영속화 — M3+
12. TerminalSurface Claude subprocess 실행 통합 — 기존 SlashInjector 유지

---

## 수정 파일 목록

### Swift 신규
- `app/Sources/Shell/Splits/ActivePaneProvider.swift`
- `app/Tests/ActivePaneProviderTests.swift` (≥5건)
- `app/Tests/TerminalSurfaceEnvironmentTests.swift` (≥3건)
- `app/Tests/CommandPaletteSurfaceOpenTests.swift` (≥6건)
- `app/Tests/CommandPalettePaneSplitTests.swift` (≥4건)

### Swift 수정
- `app/Sources/ViewModels/WorkspaceViewModel.swift` (activePane, tabModels + register/unregister)
- `app/Sources/Shell/Splits/PaneSplitView.swift` (PaneSplitContainerView 동기화, SurfaceRouter.terminal 교체, TerminalSurfacePlaceholder 제거, LeafPaneView tabModels 등록)
- `app/Sources/Shell/Content/PaneContainer.swift` (`.environment(\.activeWorkspace)` 주입)
- `app/Sources/Surfaces/Terminal/TerminalSurface.swift` (GhosttyHost body 실 GhosttyKit 래퍼 교체)
- `app/Sources/Shell/RootSplitView.swift` (onSurfaceOpen/onPaneSplit 실제 구현)

### Rust
- 변경 0건 (FFI 표면 유지)

### CI/CD
- 변경 0건 (SPEC-M2-001 MS-7 워크플로우 재사용)

---

## 태스크 (T-M2.5-001 ~ T-M2.5-018, 18건)

| Sprint | Tasks | 범위 |
|--------|-------|------|
| MS-1 | T-M2.5-001~T-M2.5-005 | ActivePaneProvider + 환경값 + tests |
| MS-2 | T-M2.5-006~T-M2.5-009 | TerminalSurface 실연결 + tests |
| MS-3 | T-M2.5-010~T-M2.5-015 | Palette 콜백 활성화 + tests |
| Cross | T-M2.5-016~T-M2.5-018 | MX 태그 + regression + 수동 검증 |

---

## 성능 / 품질 게이트

- Metal surface ≥60 fps (GhosttyMetalBenchmarkTests 하네스 통과)
- FFI P95 < 1 ms (FFIBenchmarkTests regression 0)
- Command Palette 열기→실행 반영 ≤ 300 ms
- Pane 분할 반응 < 100 ms
- Swift 빌드 시간 증가 < 5%
- RSS (8 pane, 8 tab) < 600 MB
- 레이아웃 복원 100%
- `grep TerminalSurfacePlaceholder app/Sources/` = 0
- `grep TODO(MS-7) app/Sources/Shell/RootSplitView.swift` = 0
