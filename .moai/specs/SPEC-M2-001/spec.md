# SPEC-M2-001: M2 Viewers -- Pane Splitting + Tab UI + Command Palette + 4 Surfaces + CI/CD (ARCHIVED — v2 Swift design)

> **⚠️ SUPERSEDED (2026-04-24)**: 본 SPEC 은 Swift/AppKit 기반 v2 아키텍처를 전제한다. 2026-04-21 v3 pivot (GPUI + Rust) 으로 기존 구현 경로가 `archive/swift-legacy/` 로 이관되었으며, Pane splitting + Tab UI 기능은 SPEC-V3-003 Pane Core (MS-1 ~ MS-3) 로 재설계되었다.
>
> **후속 조치**: (b) `status: archived-v2-design` 로 동결 채택 (2026-04-24 Priority Low 정비).

---
id: SPEC-M2-001
version: 1.3.0-archived
status: archived-v2-design
created: 2026-04-13
updated: 2026-04-24
superseded_by: SPEC-V3-003
author: MoAI (manager-spec)
priority: High
issue_number: 0
labels: [archived, v2-swift, m2, viewers, pane-splitting, tab-ui, superseded]
revision: v1.3.0-archived (Priority Low 정비 2026-04-24 — v3 pivot 으로 archive, superseded_by 명시)
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.3.0-archived | 2026-04-24 | v3 pivot 으로 archive. Swift/AppKit 기반 설계는 Rust + GPUI 기반 v3 SPEC (SPEC-V3-003) 로 계승. status: completed → archived-v2-design. Priority Low 정비 PR. |
| 1.2.0 | 2026-04-14 | MS-7 완료 (최종). CI/CD + C-1~C-8 carry-over 해소. 233 Rust + 106 Swift = 339 테스트. status: completed. |
| 1.1.0 | 2026-04-14 | MS-1~MS-3 구현 완료. DB V3 + FFI pane/surface + NSSplitView binary tree + Tab UI + SurfaceProtocol. 213 Rust + 41 Swift 테스트 통과. |
| 1.0.0 | 2026-04-13 | 초안 작성. M1 conditional GO 기반, M1 carry-over 8건 포함 |

---

## 1. 개요

MoAI Studio 의 "Viewers" 마일스톤. M1 에서 달성한 Working Shell (단일 TerminalSurface + 사이드바 + 다중 워크스페이스) 위에 **Pane Splitting (NSSplitView binary tree), Tab UI, Command Palette, 4종 Surface (FileTree/Markdown/Image/Browser)** 를 구현하고, **CI/CD 파이프라인**을 구축하며, **M1 carry-over 8건**을 해소한다.

**성공 기준**: 앱 실행 -> Cmd+\ 로 pane split -> 좌측 FileTree + 우측 Terminal -> FileTree 에서 .md 파일 클릭 -> 새 탭에 Markdown surface 열림 -> Cmd+K 로 Command Palette 열어 `/moai plan` 입력 -> 이미지 파일 클릭 -> Image surface 에 표시 -> Browser surface 로 localhost:3000 열기 -> 모든 상태 앱 재시작 후 복원.

**선행 조건**: SPEC-M1-001 conditional GO (186 tests, 12 crates). C-1~C-3 수동 검증 M2 진행 중 병렬 해소.

**M1 carry-over 항목 (8건)**:
- C-1: Xcode UITest 서명 + E2EWorkingShellTests 실행
- C-2: Real Claude CLI AC-4.1 응답 수신 테스트
- C-3: 10min 4-ws stress + RSS <400MB
- C-4: GhosttyKit Metal 60fps 측정
- C-5: swift-bridge Vectorizable workaround 제거
- C-6: Auth token rotation (hook-http)
- C-7: Swift FFI <1ms XCTest benchmark
- C-8: State machine force_paused 정식 API

**참조 문서**:
- `DESIGN.v4.md` SS3.1, SS6 (Pane/Surface DB 스키마)
- `SPEC-M1-001/spec.md` (M1 산출물 기준)
- `SPEC-M1-001/m1-completion-report.md` (carry-over 항목)
- `.moai/project/product.md` SS4 (핵심 기능 14개)
- `.moai/project/tech.md` (Swift/Rust 스택)

---

## 2. M1 carry-over 제약

M1 에서 이월된 8개 항목이 M2 의 병렬/선행 작업으로 포함된다.

| # | 분류 | 제약 | 적용 대상 |
|---|------|------|-----------|
| C-1 | 테스트 | Xcode UITest 서명 자동화 — 개발자 인증서 또는 CI 서명 설정 필요 | RG-M2-9 |
| C-2 | 테스트 | 실제 Claude CLI 바이너리로 AC-4.1 응답 수신 E2E 검증 | RG-M2-9 |
| C-3 | 성능 | 10분 4-ws stress + RSS <400MB — Instruments 또는 ps 샘플링 | RG-M2-9 |
| C-4 | 성능 | GhosttyKit Metal 60fps@4K 측정 — 런타임 GPU 벤치마크 | RG-M2-9 |
| C-5 | 기술부채 | swift-bridge Vectorizable workaround 제거 — 상위 버전 호환 시 | RG-M2-9 |
| C-6 | 보안 | Auth token rotation (moai-hook-http) — 토큰 만료/갱신 처리 | RG-M2-9 |
| C-7 | 검증 | Swift FFI <1ms XCTest benchmark — 앱 타겟 내 성능 테스트 | RG-M2-9 |
| C-8 | 기술부채 | state machine force_paused 정식 API — public method 승격 | RG-M2-9 |

---

## 3. 요구사항 그룹 (EARS 형식)

### RG-M2-1: Pane Splitting (NSSplitView binary tree)

**[Ubiquitous]** Rust Store 는 `panes` 테이블 (V3 마이그레이션) 을 **포함해야 한다** (shall include). 스키마: `id INTEGER PK AUTOINCREMENT, workspace_id INTEGER NOT NULL REFERENCES workspaces(id), parent_id INTEGER, split TEXT (horizontal|vertical|leaf), ratio REAL`.

**[Event-Driven]** 사용자가 pane split 단축키 (Cmd+\ 수평, Cmd+Shift+\ 수직) 를 누르면 (When), 시스템은 현재 활성 pane 을 binary tree 분할하여 두 개의 자식 pane 을 **생성해야 한다** (shall create).

**[Event-Driven]** 사용자가 pane 경계를 드래그하면 (When), 시스템은 ratio 값을 실시간으로 **업데이트해야 한다** (shall update). 최소 pane 크기는 200pt 로 **제한해야 한다** (shall constrain).

**[Event-Driven]** 사용자가 pane 을 닫으면 (When), 시스템은 해당 leaf 노드를 제거하고 형제 노드를 부모로 **승격해야 한다** (shall promote). 마지막 pane 은 닫을 수 **없어야 한다** (shall not close).

**[State-Driven]** 앱이 종료 상태일 때 (While terminated), 시스템은 pane tree 구조와 ratio 를 DB 에 **영속해야 한다** (shall persist). 재시작 시 동일한 레이아웃을 **복원해야 한다** (shall restore).

**[Ubiquitous]** Swift 측 `Splits/` 디렉토리에 NSSplitView 를 감싸는 `PaneSplitView` 를 **구현해야 한다** (shall implement). SwiftUI `NSViewRepresentable` wrapper 로 제공한다.

**[Ubiquitous]** FFI 레이어는 pane CRUD (create, read, update_ratio, delete, list_by_workspace) 함수를 **노출해야 한다** (shall expose).

**산출물**: `moai-store` V3 마이그레이션, `moai-ffi` pane FFI, `app/Sources/Shell/Splits/` NSSplitView wrapper

---

### RG-M2-2: Tab UI

**[Ubiquitous]** 각 pane (leaf 노드) 는 상단에 tab bar 를 **표시해야 한다** (shall display). 각 탭은 하나의 surface 인스턴스를 나타낸다.

**[Event-Driven]** 사용자가 + 버튼 또는 Cmd+T 를 누르면 (When), 시스템은 활성 pane 에 새 탭을 **생성해야 한다** (shall create). 새 탭은 EmptyState 로 초기화된다.

**[Event-Driven]** 사용자가 탭의 X 버튼 또는 Cmd+W 를 누르면 (When), 시스템은 해당 탭을 **닫아야 한다** (shall close). 마지막 탭을 닫으면 pane 자체를 닫는다 (RG-M2-1 규칙 적용).

**[Event-Driven]** 사용자가 탭을 드래그하면 (When), 시스템은 같은 pane 내에서 탭 순서를 **재배치해야 한다** (shall reorder).

**[Ubiquitous]** 활성 탭은 시각적으로 구분되는 표시기 (밑줄 또는 배경색) 를 **가져야 한다** (shall have).

**[Ubiquitous]** Rust Store 는 `surfaces` 테이블 (V3 마이그레이션) 을 **포함해야 한다** (shall include). 스키마: `id INTEGER PK AUTOINCREMENT, pane_id INTEGER NOT NULL REFERENCES panes(id), kind TEXT NOT NULL, state_json BLOB, tab_order INTEGER NOT NULL DEFAULT 0`.

**[State-Driven]** 앱 종료 시 (While terminated), 탭 순서와 활성 탭 인덱스를 DB 에 **영속해야 한다** (shall persist).

**산출물**: `app/Sources/Shell/Tabs/` tab bar 컴포넌트, `moai-store` surfaces 테이블, FFI surface CRUD

---

### RG-M2-3: Command Palette

**[Event-Driven]** 사용자가 Cmd+K 를 누르면 (When), 시스템은 Command Palette 오버레이를 **표시해야 한다** (shall show). Escape 키로 **닫아야 한다** (shall dismiss).

**[Event-Driven]** 사용자가 텍스트를 입력하면 (When), 시스템은 등록된 명령 목록에서 fuzzy matching 으로 결과를 **필터링해야 한다** (shall filter).

**[Ubiquitous]** Command Palette 는 다음 명령 카테고리를 **포함해야 한다** (shall include):
- `/moai *` 14개 슬래시 커맨드 (slash injection -> Rust core -> SDKUserMessage -> Claude subprocess)
- Surface 열기 (FileTree, Markdown, Image, Browser)
- Workspace 조작 (생성, 전환, 삭제)
- Pane 조작 (수평 분할, 수직 분할, 닫기)

**[Event-Driven]** 사용자가 `/moai` 로 시작하는 명령을 선택하면 (When), 시스템은 해당 텍스트를 Rust core slash injection 파이프라인으로 **전달해야 한다** (shall relay).

**[Ubiquitous]** 키보드 네비게이션 (Arrow Up/Down, Enter 선택, Escape 취소) 을 **지원해야 한다** (shall support).

**[Optional]** 최근 사용한 명령 히스토리를 상단에 **표시할 수 있다** (may display).

**산출물**: `app/Sources/Shell/CommandPalette/` 컴포넌트, 명령 레지스트리 시스템

---

### RG-M2-4: Surface Protocol + FileTree Surface

**[Ubiquitous]** Swift 에 `SurfaceProtocol` 을 **정의해야 한다** (shall define). 공통 인터페이스로 10종 surface 타입의 lifecycle (init, activate, deactivate, destroy) 과 toolbar items 를 통일한다.

**[Ubiquitous]** `SurfaceProtocol` 은 다음 속성과 메서드를 **포함해야 한다** (shall include):
- `var surfaceKind: SurfaceKind { get }` (enum: terminal, filetree, markdown, image, browser, code, agentRun, kanban, memory, instructionsGraph)
- `var toolbarItems: [ToolbarItem] { get }` (surface 별 툴바 아이템)
- `func activate()` / `func deactivate()` (탭 전환 시 호출)
- `func destroy()` (탭 닫기 시 호출)

**[Event-Driven]** FileTree surface 가 활성화되면 (When), 시스템은 현재 워크스페이스의 루트 디렉토리를 기준으로 파일 트리를 **렌더링해야 한다** (shall render).

**[Event-Driven]** 사용자가 디렉토리 노드를 클릭하면 (When), 시스템은 해당 디렉토리를 expand/collapse **토글해야 한다** (shall toggle).

**[Ubiquitous]** 파일 트리는 파일 타입별 아이콘과 git status 색상 (modified=노랑, added=초록, untracked=회색) 을 **표시해야 한다** (shall display).

**[Event-Driven]** 사용자가 파일을 더블클릭하면 (When), 시스템은 파일 확장자에 따라 적절한 surface 에서 **열어야 한다** (shall open): `.md` -> Markdown surface, 이미지 -> Image surface, 기타 -> 기본 터미널.

**[State-Driven]** 파일 시스템에 변경이 발생하는 동안 (While fs events), moai-fs (notify) 와 연동하여 트리를 **실시간 갱신해야 한다** (shall refresh).

**[Ubiquitous]** FFI 는 Rust 측 디렉토리 리스팅 (moai-fs 연동) + git status 데이터를 Swift 로 **전달해야 한다** (shall deliver).

**산출물**: `SurfaceProtocol.swift`, `app/Sources/Surfaces/FileTree/`, `moai-ffi` filetree FFI

---

### RG-M2-5: Markdown Surface

**[Event-Driven]** Markdown surface 가 파일 경로와 함께 활성화되면 (When), 시스템은 Down (cmark wrapper) 을 사용하여 렌더링된 HTML 을 **표시해야 한다** (shall display).

**[Ubiquitous]** EARS SPEC 형식을 특수 포매팅 (requirement ID 강조, Given/When/Then 블록 시각화) 으로 **렌더링해야 한다** (shall render).

**[Event-Driven]** KaTeX 수식 블록 (`$$...$$` 또는 `$...$`) 이 감지되면 (When), 시스템은 WKWebView 임베드를 통해 수학 수식을 **렌더링해야 한다** (shall render).

**[Event-Driven]** Mermaid 코드 블록 (` ```mermaid `) 이 감지되면 (When), 시스템은 WKWebView 임베드를 통해 다이어그램을 **렌더링해야 한다** (shall render).

**[State-Driven]** 원본 파일이 변경되는 동안 (While file modified), 시스템은 자동으로 콘텐츠를 **리로드해야 한다** (shall reload). moai-fs (notify) 이벤트 연동.

**[Ubiquitous]** 다크/라이트 테마 전환을 macOS 시스템 설정에 따라 **지원해야 한다** (shall support).

**[Optional]** 목차 (Table of Contents) 사이드바를 **제공할 수 있다** (may provide).

**산출물**: `app/Sources/Surfaces/Markdown/`, Down 의존성

---

### RG-M2-6: Image Surface

**[Ubiquitous]** Image surface 는 PNG, JPEG, GIF, SVG, WebP 형식을 **지원해야 한다** (shall support).

**[Event-Driven]** 이미지가 로드되면 (When), 시스템은 "Fit to Window" 모드로 **표시해야 한다** (shall display).

**[Event-Driven]** 사용자가 핀치 또는 Cmd+/- 를 누르면 (When), 시스템은 줌 레벨을 **조정해야 한다** (shall adjust). 드래그로 패닝을 **지원해야 한다** (shall support).

**[Event-Driven]** 사용자가 diff 모드를 활성화하면 (When), 시스템은 두 이미지를 side-by-side 로 **표시해야 한다** (shall display).

**[Event-Driven]** diff 모드에서 (When in diff mode), 시스템은 Vision framework 를 사용하여 SSIM 점수를 **계산하고 표시해야 한다** (shall compute and display).

**산출물**: `app/Sources/Surfaces/Image/`

---

### RG-M2-7: Browser Surface

**[Ubiquitous]** Browser surface 는 WKWebView 를 감싸는 wrapper 로 **구현해야 한다** (shall implement).

**[Ubiquitous]** URL bar + 네비게이션 컨트롤 (뒤로, 앞으로, 새로고침) 을 **제공해야 한다** (shall provide).

**[Event-Driven]** Browser surface 가 활성화되면 (When), 시스템은 localhost 포트 (3000, 5173, 8080 등) 를 스캔하여 실행 중인 dev 서버를 **자동 감지해야 한다** (shall auto-detect).

**[Event-Driven]** 사용자가 URL bar 에 주소를 입력하고 Enter 를 누르면 (When), 시스템은 해당 URL 로 **이동해야 한다** (shall navigate).

**[Optional]** Web Inspector (DevTools) 패널을 **제공할 수 있다** (may provide).

**[Event-Driven]** 링크 클릭 시 (When), 같은 WKWebView 내에서 내부 네비게이션으로 **처리해야 한다** (shall handle). 외부 도메인 링크는 시스템 브라우저로 **열어야 한다** (shall open externally).

**산출물**: `app/Sources/Surfaces/Browser/`

---

### RG-M2-8: CI/CD Pipeline

**[Ubiquitous]** GitHub Actions 워크플로우는 Rust 검증 (cargo check, cargo test, cargo clippy, cargo fmt --check) 을 **실행해야 한다** (shall run).

**[Ubiquitous]** GitHub Actions 워크플로우는 Swift 빌드 검증 (xcodebuild build-for-testing, xcodebuild test) 을 **실행해야 한다** (shall run).

**[Ubiquitous]** GhosttyKit xcframework 빌드 결과를 캐싱하여 CI 시간을 **최적화해야 한다** (shall optimize).

**[Ubiquitous]** Rust xcframework 빌드를 `scripts/build-rust-xcframework.sh` 기반으로 **자동화해야 한다** (shall automate).

**[Ubiquitous]** CI 매트릭스는 macOS 14+ 와 Xcode 15+ 조합을 **포함해야 한다** (shall include).

**[Optional]** Branch protection rules (main 브랜치 PR 필수, CI 통과 필수) 를 **권장한다** (should recommend).

**[Unwanted]** CI 가 실패하면 (If CI fails), 시스템은 Slack 또는 GitHub 알림으로 실패 사유를 **통지해야 한다** (shall notify).

**산출물**: `.github/workflows/ci-rust.yml`, `.github/workflows/ci-swift.yml`, 캐싱 설정

---

### RG-M2-9: M1 Carry-over Resolution

**[Event-Driven]** CI 환경이 구성되면 (When CI configured), C-1 (Xcode UITest 서명) 을 CI 워크플로우에 **통합해야 한다** (shall integrate). `E2EWorkingShellTests` 가 CI 에서 실행 가능해야 한다.

**[Event-Driven]** Claude CLI 바이너리가 사용 가능한 환경에서 (When Claude CLI available), C-2 (AC-4.1 응답 수신 테스트) 를 **검증해야 한다** (shall verify). 스크립트 기반 자동화 또는 수동 체크리스트.

**[Ubiquitous]** C-3 (10min 4-ws stress + RSS <400MB) 를 자동화된 성능 테스트 스크립트로 **구현해야 한다** (shall implement).

**[Ubiquitous]** C-4 (GhosttyKit Metal 60fps) 를 런타임 GPU 벤치마크 XCTest 로 **측정해야 한다** (shall measure).

**[Event-Driven]** swift-bridge 가 Vectorizable 을 지원하는 버전으로 업데이트되면 (When swift-bridge updated), C-5 (workaround 제거) 를 **수행해야 한다** (shall perform). 현재 버전에서 미지원 시 이월 허용.

**[Ubiquitous]** C-6 (Auth token rotation) 을 moai-hook-http 에 토큰 만료 감지 + 갱신 로직으로 **구현해야 한다** (shall implement).

**[Ubiquitous]** C-7 (Swift FFI <1ms benchmark) 을 XCTest Performance 로 **측정해야 한다** (shall measure). P95 <1ms 기준.

**[Ubiquitous]** C-8 (force_paused 정식 API) 을 state machine 에 public method 로 **승격해야 한다** (shall promote). 문서화 포함.

**산출물**: CI 통합 테스트, 성능 스크립트, moai-hook-http 갱신, state machine API 개선

---

## 4. 비기능 요구사항 (NFR)

| 항목 | 목표 | 측정 방법 |
|------|------|-----------|
| Pane split 반응 | <100ms | 단축키 -> 화면 분할 완료 시간 |
| Tab 전환 | <50ms | 탭 클릭 -> surface 활성화 시간 |
| Command Palette 열기 | <200ms | Cmd+K -> 오버레이 표시 시간 |
| FileTree 초기 로드 | <500ms (1000 파일 기준) | 워크스페이스 활성화 -> 트리 렌더 완료 |
| FileTree 실시간 갱신 | <300ms | fs event -> 트리 UI 반영 |
| Markdown 렌더링 | <1s (100KB 파일) | 파일 열기 -> 렌더 완료 |
| Image 로드 | <500ms (10MB 이미지) | 파일 열기 -> 표시 완료 |
| Browser 초기 로드 | <2s (localhost) | surface 활성 -> 페이지 로드 |
| CI Rust 워크플로우 | <10min | GitHub Actions 실행 시간 |
| CI Swift 워크플로우 | <15min | GitHub Actions 실행 시간 |
| 메모리 (8 pane, 8 tab) | RSS <600MB | Instruments 또는 ps 샘플링 |
| 앱 재시작 후 레이아웃 복원 | 100% 일치 | pane tree + tab 순서 + surface kind 동일 |

---

## 5. Exclusions (What NOT to Build)

1. **Code Viewer surface** -- M3 범위. SwiftTreeSitter + LSP 진단 + @MX 거터 + tri-pane diff 는 별도 SPEC.
2. **Agent Run Viewer** -- M5 범위. hook event stream + cost tracking + live agent control.
3. **Kanban board** -- M5 범위. SPEC <-> worktree <-> `/moai run` 자동 연동.
4. **Memory Viewer** -- M5 범위. `~/.claude/projects/<root>/memory/` 렌더.
5. **InstructionsGraph** -- M5 범위. 세션 컨텍스트 디버거.
6. **LSP integration** -- M4 범위. `.lsp.json` 기반 LSP as plugin.
7. **Native Permission Dialog** -- M4 범위. TUI text prompt 대체.
8. **16+ 동시 워크스페이스** -- M6 범위. actor supervision 확장.
9. **Auto-update (Sparkle)** -- M6 범위. 자동 업데이트 프레임워크.
10. **Onboarding wizard** -- M4 범위. 첫 실행 안내.
11. **탭 간 drag-and-drop (cross-pane)** -- M3+ 범위. 같은 pane 내 reorder 만 M2.
12. **Surface 간 통신 프로토콜** -- M3 범위. surface-to-surface 이벤트 버스.

---

## 6. 용어 정의

| 용어 | 정의 |
|------|------|
| Pane | NSSplitView binary tree 의 노드. leaf 노드는 tab bar + surface 를 포함 |
| Surface | 하나의 뷰어 인스턴스. SurfaceProtocol 구현체 (terminal, filetree, markdown, image, browser 등 10종) |
| Tab | pane 내의 surface 선택 단위. 하나의 탭 = 하나의 surface |
| Command Palette | Cmd+K 로 열리는 fuzzy search 오버레이. 명령 실행 허브 |
| Slash Injection | Command Palette 에서 `/moai *` 명령을 Rust core 를 통해 Claude subprocess 에 전달하는 경로 |
| Binary Tree | pane 의 분할 구조. 각 non-leaf 노드는 horizontal 또는 vertical split, leaf 노드는 surface 포함 |
