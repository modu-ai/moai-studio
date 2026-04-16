# SPEC-M2-002 Acceptance Criteria

---
spec_id: SPEC-M2-002
version: 1.0.0
status: draft
created: 2026-04-16
---

## 1. Given/When/Then 시나리오

### RG-M2.5-1: ActivePaneProvider (P-4)

**AC-1.1: 기본 environment 값 — 최초 주입 전 상태**

- **Given**: 어떤 SwiftUI 뷰가 `.environment(\.activePane, ...)` modifier 를 적용받지 않은 상태
- **When**: 해당 뷰가 `@Environment(\.activePane) var ctx: ActivePaneContext` 로 값을 조회한다
- **Then**: `ctx.paneId == nil`, `ctx.model == nil`, `ctx.workspace == nil` 이 관찰되며, `ctx == ActivePaneContext.empty` 이다

**AC-1.2: 주입 후 값 전파**

- **Given**: `ActivePaneContext(paneId: 42, model: mockModel, workspace: mockWorkspace)` 가 준비되고, 어떤 parent View 가 `.environment(\.activePane, ctx)` 를 적용한다
- **When**: parent 의 자식 뷰가 `@Environment(\.activePane) var childCtx` 를 조회한다
- **Then**: `childCtx.paneId == 42`, `childCtx.model === mockModel`, `childCtx.workspace == mockWorkspace` 로 동일 인스턴스가 관찰된다

**AC-1.3: 중첩 override — 가장 가까운 주입이 이긴다**

- **Given**: 바깥쪽 View 가 `.environment(\.activePane, ctxOuter)` 를 적용하고, 그 내부 View 가 다시 `.environment(\.activePane, ctxInner)` 를 중첩 적용한다
- **When**: 가장 안쪽 View 가 `@Environment(\.activePane) var ctx` 를 조회한다
- **Then**: `ctx == ctxInner` 이며 `ctxOuter` 는 관찰되지 않는다

**AC-1.4: Pane 전환 시 activePaneId 업데이트 + workspaceVM 동기화**

- **Given**: `PaneSplitContainerView` 가 워크스페이스 A 의 `PaneTreeModel` 을 로드하고 root leaf pane (id=100) 이 활성 상태
- **When**: 사용자가 pane 영역을 탭하거나 Cmd+\ 로 분할하여 `activePaneId` 가 200 으로 전환된다
- **Then**: `workspaceVM.activePane.paneId == 200` 이 업데이트되고, 하위 뷰 트리의 `@Environment(\.activePane)` 값도 `paneId == 200` 을 반영한다

**AC-1.5: 마지막 pane 만 남은 상태의 활성값**

- **Given**: 2개 pane 이 있고 하나를 Cmd+Shift+W 로 닫은 후 단일 pane (id=300) 만 남은 상태
- **When**: `PaneSplitContainerView` 의 `activePaneId` 가 업데이트된다
- **Then**: `activePaneId == 300` 이며 `workspaceVM.activePane.paneId == 300`. nil 이 아니다.

**AC-1.6: Split (non-leaf) 노드는 활성값이 될 수 없음**

- **Given**: `PaneTreeModel` 의 node 100 이 leaf 에서 horizontal split 으로 전환되어 non-leaf 가 된 상태
- **When**: 어떤 경로에서 `activePaneId = 100` (non-leaf) 이 할당을 시도한다
- **Then**: DEBUG 빌드에서는 assertion failure 가 발생하고, RELEASE 빌드에서는 로그 경고 + 해당 할당이 무시되어 이전 유효한 leaf id 가 유지된다

---

### RG-M2.5-2: TerminalSurface GhosttyHost 실연결 (P-1)

**AC-2.1: 신규 워크스페이스 생성 시 실 TerminalSurface 렌더**

- **Given**: 앱이 실행 중, `MOAI_TERMINAL_BACKEND` 환경변수 미설정 (기본 `.ghostty`), Metal Toolchain 설치 환경
- **When**: 사용자가 사이드바에서 "+" 버튼으로 새 워크스페이스 `"demo"` 를 생성하고 선택한다
- **Then**: 활성 pane 의 기본 Terminal 탭에 실제 GhosttyKit Metal surface 가 렌더되며, `"(Ghostty Metal surface will render here — wiring in MS-6)"` 텍스트는 표시되지 않는다. `grep -r "TerminalSurfacePlaceholder" app/Sources/` 결과 0건이다.

**AC-2.2: Workspace 교체 시 surface 재바인딩**

- **Given**: 워크스페이스 A 의 Terminal surface 가 렌더 중
- **When**: 사용자가 사이드바에서 워크스페이스 B 를 선택한다
- **Then**: `PaneContainer.loadModelIfNeeded(for: B)` 가 실행되어 `.environment(\.activeWorkspace, snapshotB)` 가 B 의 `WorkspaceSnapshot` 으로 업데이트되고, `TerminalSurface(workspace: snapshotB)` 가 B 의 컨텍스트로 재렌더된다

**AC-2.3: nstext 백엔드 강제 전환**

- **Given**: 앱이 실행되기 전 `MOAI_TERMINAL_BACKEND=nstext` 환경변수가 설정됨
- **When**: 앱을 실행하고 워크스페이스를 선택한다
- **Then**: `TerminalSurface.body` 의 `TerminalBackend.current == .nstext` 분기가 활성화되어 `TerminalFallback(workspace:)` 가 렌더되고, GhosttyKit 초기화는 호출되지 않는다

**AC-2.4: GhosttyKit 초기화 실패 시 fallback 전환**

- **Given**: mock 환경에서 GhosttyKit 초기화가 실패하도록 설정 (예: Metal Toolchain 부재 시뮬레이션)
- **When**: `TerminalSurface` 가 렌더를 시도한다
- **Then**: `GhosttyHost.onFailure()` 콜백이 호출되어 `loadFailed = true` 로 전환되고, 후속 렌더에서 `TerminalFallback` 이 표시된다. 앱은 크래시하지 않는다.

**AC-2.5: activeWorkspace nil 시 SurfaceRouter 안내 뷰**

- **Given**: `SurfaceRouter` 가 `.terminal` 케이스를 렌더하는 상황에서 `@Environment(\.activeWorkspace)` 가 nil
- **When**: `SurfaceRouter.body` 가 평가된다
- **Then**: 기존 `TerminalSurface(workspace:)` 대신 workspace nil 안내 뷰 (예: "워크스페이스를 선택하세요") 가 표시된다. 앱은 크래시하지 않으며 기존 탭 상태는 유지된다.

---

### RG-M2.5-3: Command Palette onSurfaceOpen (P-2)

**AC-3.1: FileTree 명령 실행 → 활성 pane 에 FileTreeSurface 새 탭 생성**

- **Given**: 워크스페이스 A 가 선택되어 있고 root leaf pane (id=100) 이 활성이며 `workspaceVM.tabModels[100]` 에 TabBarViewModel 이 등록되어 있다. Command Palette 가 Cmd+K 로 열려 있다.
- **When**: 사용자가 Command Palette 에서 `surface.filetree` ("File Tree 열기") 를 선택한다
- **Then**: `CommandRegistry.onSurfaceOpen(.filetree)` → `RootSplitView` 클로저 → `tabModels[100]!.newTab(kind: .filetree)` 호출이 발생하여 새 탭이 추가되고 즉시 활성화된다. 탭의 Surface 는 `FileTreeSurface` 로 렌더된다.

**AC-3.2: Markdown 명령 실행 → 활성 pane 에 MarkdownSurface 새 탭 + EmptyState**

- **Given**: AC-3.1 과 동일한 상태
- **When**: 사용자가 `surface.markdown` ("Markdown 파일 열기") 을 선택한다
- **Then**: `newTab(kind: .markdown)` 가 statePath nil 로 호출되고, `MarkdownSurface(filePath: "")` 가 EmptyState 로 렌더된다. 사용자가 FileTree 에서 `.md` 파일을 선택해야 실제 내용이 표시된다 (본 SPEC 범위 외, M3 이월).

**AC-3.3: Browser / Image / Terminal 명령도 동일 시나리오**

- **Given**: AC-3.1 과 동일한 상태
- **When**: 사용자가 `surface.browser` / `surface.image` / `surface.terminal` 중 하나를 선택한다
- **Then**: 각각 `newTab(kind: .browser|.image|.terminal)` 이 호출되고 해당 Surface 가 렌더된다. `.terminal` 의 경우 AC-2.1 과 연계되어 GhosttyHost Metal surface 가 렌더된다 (Metal Toolchain 있는 환경에서).

**AC-3.4: 활성 pane 이 없는 상태에서 명령 무시**

- **Given**: 워크스페이스 선택 전 (EmptyState 표시) 또는 `workspaceVM.activePane.paneId == nil`
- **When**: 사용자가 Cmd+K 로 팔레트를 열고 `surface.filetree` 를 선택한다 (이 상황 자체가 드물지만 가능)
- **Then**: `onSurfaceOpen` 클로저가 activePane nil 을 감지하고 즉시 return 한다. 크래시나 silent error 가 발생하지 않으며, 로그 레벨 `info` 로 `"Command Palette: onSurfaceOpen ignored — no active pane"` 가 기록된다.

**AC-3.5: M3+ 이월 SurfaceKind 는 NotYetImplementedSurface 로 폴백**

- **Given**: `onSurfaceOpen(.code)` 또는 `.agentRun` / `.kanban` / `.memory` / `.instructionsGraph` 중 하나가 호출된 상황 (현재는 CommandRegistry 에 등록되지 않아 도달 불가이지만, 향후 register 확장 시 검증)
- **When**: `SurfaceRouter` 가 해당 kind 를 라우팅한다
- **Then**: `NotYetImplementedSurface(kind: ...)` 가 렌더되어 "M3+ 구현 예정" 안내가 표시된다. 크래시 없음.

---

### RG-M2.5-4: Command Palette onPaneSplit (P-3)

**AC-4.1: 수평 분할 명령 — 활성 pane 이 좌우로 분할되고 새 pane 활성화**

- **Given**: 워크스페이스 A 의 단일 leaf pane (id=100) 이 활성이며 `workspaceVM.activePane.model === paneTreeModelA`
- **When**: 사용자가 Command Palette 에서 `pane.split.horizontal` ("Pane 수평 분할") 을 선택한다
- **Then**: `onPaneSplit(.horizontal)` → `paneTreeModelA.splitActive(100, direction: .horizontal)` 호출이 발생하여 기존 pane 100 이 horizontal split 노드로 전환되고 자식 childA (좌측, 기존 surface), childB (우측, 새 leaf) 가 생성된다. `activePaneId` 가 childB 로 업데이트되고 `workspaceVM.activePane.paneId == childB` 이다. UI 상 좌우 NSSplitView 가 렌더된다. 결과는 Cmd+\ 단축키 경로와 **완전히 동일**하다.

**AC-4.2: 수직 분할 명령 — 활성 pane 이 상하로 분할**

- **Given**: AC-4.1 과 동일한 상태
- **When**: 사용자가 `pane.split.vertical` ("Pane 수직 분할") 을 선택한다
- **Then**: `splitActive(100, direction: .vertical)` 호출이 발생하여 상하 NSSplitView 가 렌더된다. DB 에도 동일한 pane tree 가 영속되어 앱 재시작 후 복원 가능 (AC-G.4).

**AC-4.3: 활성 pane 컨텍스트가 없는 상태에서 명령 무시**

- **Given**: `workspaceVM.activePane.paneId == nil` 또는 `workspaceVM.activePane.model == nil`
- **When**: 사용자가 `pane.split.horizontal` 을 선택한다
- **Then**: `onPaneSplit` 클로저가 nil 을 감지하고 즉시 return 한다. `splitActive` 호출이 발생하지 않으며 크래시 없음. 로그 레벨 `info` 로 `"Command Palette: onPaneSplit ignored — no active pane context"` 가 기록된다.

**AC-4.4: 팔레트 경로와 단축키 경로의 결과 동일성**

- **Given**: 동일한 초기 pane tree 상태 (단일 leaf id=100)
- **When**: 실험 A 에서 사용자가 Cmd+\ 를 누르고, 실험 B 에서 사용자가 Cmd+K → "Pane 수평 분할" 을 선택한다 (두 실험은 독립)
- **Then**: 두 실험 모두 동일한 `PaneTreeModel.nodes` 구조 (split=.horizontal, 자식 2개) 로 수렴하며, 동일한 DB 행 구조 (`panes` 테이블) 가 영속된다. `activePaneId` 도 동일하게 새 leaf (childB) 로 설정된다.

---

## 2. Edge Cases

| 시나리오 | 기대 동작 |
|----------|-----------|
| `ActivePaneContext.model` 이 `@MainActor` 외부에서 접근됨 | Swift 6 concurrency 체크에 의해 컴파일 오류. `@MainActor` 격리 유지 필수 |
| `PaneSplitContainerView` 내부에서 `activePaneId` 가 nil 이 되는 상황 (close 후 일시적) | `workspaceVM.activePane.paneId = nil` 로 동기화. Command Palette 경로는 AC-3.4 / AC-4.3 no-op |
| `LeafPaneView.task` 블록이 `await model.load()` 중일 때 사용자가 Cmd+K 로 `surface.filetree` 실행 | `tabModels[paneId]` 가 아직 등록되지 않음 → AC-3.4 와 동일 no-op 경로. load 완료 후 다시 시도하면 정상 동작 |
| `splitActive` 가 nil 을 반환하는 edge case (FFI 실패) | `workspaceVM.activePane.paneId` 는 변경되지 않음. 사용자 UI 변화도 없음 (현재 pane 유지) |
| Command Palette 가 열려 있는 상태에서 워크스페이스 전환 | Palette 는 닫힘 (Escape 또는 자동), `workspaceVM.activePane` 이 새 워크스페이스의 root leaf 로 갱신됨 |
| GhosttyKit 초기화가 부분적으로 성공하지만 일부 rendering 실패 | `onFailure()` 호출 경로 (AC-2.4). `TerminalFallback` 으로 전환. 사용자는 로그 또는 fallback 안내로 상황 인지 |
| Metal Toolchain 미설치 환경에서 CI 실행 | `MOAI_TERMINAL_BACKEND=nstext` 로 CI 환경변수 고정. `TerminalFallback` 만 렌더 → CI 테스트는 통과 (UI 는 제한적) |
| `SurfaceRouter` 가 `.terminal` 를 렌더하는 중 activeWorkspace 가 nil 로 전환 | AC-2.5 안내 뷰로 교체. 기존 `TerminalSurface` 인스턴스는 해제 (SwiftUI 가 수행) |
| 100개 pane 분할 후 Command Palette 로 추가 분할 | 각 pane 최소 200pt 규칙 (SPEC-M2-001 AC-1.3) 적용으로 화면 부족 시 `splitActive` 가 실패 반환. onPaneSplit 은 no-op |
| 동일 pane 에서 Cmd+K 를 연속 실행하여 여러 탭 생성 | 각 호출마다 `newTab(kind:)` 이 순차 실행되어 tab_order 가 0, 1, 2, ... 로 증가. 마지막 탭이 활성화 |
| `WorkspaceViewModel.tabModels` 에 stale entry (이미 닫힌 pane) 잔존 | close 경로에서 명시적 제거. 제거 누락 시 Cmd+K 명령 실행해도 해당 tabModel 은 UI 에 나타나지 않음 (`LeafPaneView` 파괴 시 SwiftUI 가 참조 해제) — graceful degradation |
| Swift 6 strict concurrency 에서 `@Sendable` 위반 | `ActivePaneContext` 가 `PaneTreeModel` (non-Sendable reference) 를 보유하므로 `@MainActor` 격리. 모든 사용 지점이 `@MainActor` 이어야 컴파일 통과 |

---

## 3. 성능 / 품질 게이트 (Quality Gate)

### 3.1 코드 품질

- [ ] `cargo check --workspace` 0 errors, 0 warnings (Rust 변경 없음이므로 기존 233 통과 유지)
- [ ] `cargo clippy --workspace -- -D warnings` PASS
- [ ] `cargo fmt --all -- --check` PASS
- [ ] `cargo test --workspace` 233/233 통과 (regression 0)
- [ ] Xcode `xcodebuild build-for-testing` 0 errors, 0 warnings 증가
- [ ] Xcode `xcodebuild test` 106 + 신규 ≥15 = **121+ 통과** (AC-G.2)
- [ ] SwiftUI Preview 동작 (ActivePaneProvider, PaneContainer, SurfaceRouter)

### 3.2 테스트 커버리지

- [ ] `ActivePaneProviderTests.swift` ≥5건 (AC-1.1 ~ AC-1.5 대응)
- [ ] `TerminalSurfaceEnvironmentTests.swift` ≥3건 (AC-2.3 ~ AC-2.5 대응)
- [ ] `CommandPaletteSurfaceOpenTests.swift` ≥6건 (AC-3.1 ~ AC-3.5 + nil 케이스)
- [ ] `CommandPalettePaneSplitTests.swift` ≥4건 (AC-4.1 ~ AC-4.4 대응)
- [ ] 신규 Swift 코드 70%+ 라인 커버리지 (SPEC-M2-001 기준선)
- [ ] 기존 PaneTreeModelTests 10건, TabBarViewModelTests 21건, CommandPaletteTests 등 전부 regression 0

### 3.3 NFR 달성 (spec.md §5)

- [ ] Metal surface 렌더 ≥60 fps @ 1080p — `GhosttyMetalBenchmarkTests` 하네스 통과 (Metal Toolchain 환경)
- [ ] FFI P95 < 1 ms — `FFIBenchmarkTests` regression 0
- [ ] Command Palette 열기 → 실행 → 결과 반영 ≤ 300 ms — 수동 timing 측정
- [ ] Pane 분할 반응 (팔레트 경로) < 100 ms — 단축키 경로와 동일 FFI 호출이므로 기존 NFR 유지
- [ ] Swift 빌드 시간 증가 < 5% — CI `xcodebuild build-for-testing` 시간 비교
- [ ] RSS (8 pane, 8 tab, 실 GhosttyHost 8개) < 600 MB — Instruments 또는 `ps` 샘플링
- [ ] 레이아웃 복원 정확도 100% — 앱 재시작 후 pane tree + tab order + surface kind 비교 (AC-G.4)

### 3.4 TRUST 5

- [ ] **Tested**: 위 3.2 커버리지 달성
- [ ] **Readable**: clippy + fmt clean, 한국어 코드 주석 (code_comments: ko), 명확한 타입명 (`ActivePaneContext`, `WorkspaceEnvironmentKey`)
- [ ] **Unified**: `EnvironmentKey` 패턴 일관 적용, 기존 `@Observable` 주입 패턴 준수
- [ ] **Secured**: 신규 공격 표면 없음 (환경값 전파만 추가). OWASP 해당 없음
- [ ] **Trackable**: conventional commits, T-M2.5-001 ~ T-M2.5-018 참조 포함

### 3.5 @MX Tags (plan.md §6 대응)

- [ ] 신규 `@MX:ANCHOR`: `ActivePaneContext` (fan_in ≥ 3), `EnvironmentValues.activePane`
- [ ] 신규 `@MX:NOTE`: `GhosttyHost` (placeholder 제거), `RootSplitView.onSurfaceOpen` / `onPaneSplit` (MS-3 완료), `SurfaceRouter.terminal` (MS-2 완료), `WorkspaceViewModel.activePane` (환경값 경로 설명), `WorkspaceViewModel.tabModels` (등록/해제 규약)
- [ ] 갱신 `@MX:ANCHOR`: `TabBarViewModel` (fan_in+1 — RootSplitView 추가, @MX:REASON 업데이트)
- [ ] 제거 `@MX:NOTE`: `PaneSplitView.swift` 의 구식 "MS-3/MS-4/MS-6+ 예정" 3건 (plan.md §6.4)
- [ ] TODO 0건 — 모든 `TODO(MS-7)` 주석이 RootSplitView.swift 에서 제거됨

### 3.6 CI/CD

- [ ] GitHub Actions Rust CI 통과 (기존 워크플로우 재사용)
- [ ] GitHub Actions Swift CI 통과 (기존 워크플로우 재사용)
- [ ] Metal Toolchain 부재 CI 환경에서 `MOAI_TERMINAL_BACKEND=nstext` 로 빌드/테스트 성공

### 3.7 수동 검증 체크리스트 (spec.md §7.4, Metal Toolchain 설치 환경)

- [ ] **체크 1**: 앱 실행 → 기본 워크스페이스 생성 → 기본 탭에 실제 GhosttyKit Metal surface 가 렌더된다 (placeholder 텍스트 없음) — AC-2.1
- [ ] **체크 2**: Cmd+K → `surface.filetree` / `surface.markdown` / `surface.image` / `surface.browser` / `surface.terminal` 각 명령 실행 → 활성 pane 에 새 탭 생성 — AC-3.1 ~ AC-3.3
- [ ] **체크 3**: Cmd+K → `pane.split.horizontal` → 좌우 분할, `pane.split.vertical` → 상하 분할 — AC-4.1, AC-4.2
- [ ] **체크 4**: 분할된 두 pane 중 다른 쪽을 탭 → `activePaneId` 전환 → Cmd+K → `surface.filetree` 실행 → 올바른 (클릭한) pane 에 새 탭이 생성된다 — AC-1.4 + AC-3.1
- [ ] **체크 5**: 앱 종료 → 재실행 → 위 체크 1~4 로 만든 레이아웃 (pane tree + tab order + surface kind) 100% 복원 — AC-G.4

---

## 4. Regression Gate (최종)

| 게이트 | 기준 | 확인 방법 |
|--------|------|-----------|
| Rust 테스트 | 233/233 통과 | `cargo test --workspace` |
| Swift 기존 테스트 | 106/106 통과 | `xcodebuild test` (MS-7 시점 기준) |
| Swift 신규 테스트 | ≥15 (AC-G.2) | 위와 동일 |
| Rust 코드 품질 | clippy + fmt clean | 전용 커맨드 |
| Swift 코드 품질 | warning 증가 0 | `xcodebuild build-for-testing` 로그 diff |
| grep 검증 (P-1) | `TerminalSurfacePlaceholder` 0건 | `grep -r "TerminalSurfacePlaceholder" app/Sources/` |
| grep 검증 (P-2/P-3) | `TODO(MS-7)` 0건 in RootSplitView.swift | 동일 |
| 수동 UI 검증 | 체크 1~5 모두 통과 | Metal Toolchain 환경에서 수행 |

모든 게이트 통과 시 SPEC-M2-002 는 `completed` 판정, M2 conditional GO → **Full GO 승격** 가능. Metal 60fps 실측 (C-4) 은 별도 스프린트 (Exclusions §7).

---

## 5. Definition of Done

- 본 문서의 §3 Quality Gate 전체 체크리스트 해당 항목 체크 완료
- 본 문서의 §4 Regression Gate 8개 항목 모두 PASS
- spec.md §8 Exclusions 12개 항목 범위 준수 (초과 구현 없음)
- `m2-completion-report.md` 대응 후속 보고서 작성 (예: `.moai/specs/SPEC-M2-002/polish-completion-report.md`) — Sync phase 에서 manager-docs 가 수행
- git commit 메시지에 `SPEC-M2-002` 및 해당 T-M2.5-xxx 참조 포함
