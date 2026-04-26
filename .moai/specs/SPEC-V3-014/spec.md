---
id: SPEC-V3-014
version: 1.0.0
status: implemented
created_at: 2026-04-26
updated_at: 2026-04-26
author: MoAI (manager-spec)
priority: Medium
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-004, SPEC-V3-006, SPEC-V3-012, SPEC-V3-013]
parallel_with: []
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-2, ui, banners, design-system, polish]
revision: v1.0.0 (initial draft, Banners Surface)
---

# SPEC-V3-014: Banners Surface — Top-of-Window Slim Banner Stack (5 variants) + Severity Priority + Auto-Dismiss

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-26 | 초안 작성. RG-V14-1 ~ RG-V14-7, REQ 28개, AC-V14-1 ~ AC-V14-13, MS-1/MS-2/MS-3 정의. IMPLEMENTATION-NOTES.md v1.1 §13.8 / §14 F (P1) carry. Round 2 시안 (`moai-revisions.jsx` 라인 552~635) 정합. SPEC-V3-001/004/006/012/013 의 design module + RootView + LspProvider mock 인프라 활용. v0.1.0 단계는 mock action 핸들러 한정 (실제 system event source 통합은 별도 SPEC). |

---

## 1. 개요

### 1.1 목적

moai-studio 가 사용자에게 비차단(non-blocking) 시스템 알림을 전달하기 위한 **상단 슬림 배너 스택(top-of-window slim banner stack)** 을 정의한다.

본 SPEC 의 산출은:

- **`crates/moai-studio-ui/src/banners/`** 신규 모듈 — Banner enum/trait, Severity, BannerView, BannerStack, 5 variants.
- **5 banner variants** — CrashBanner / UpdateBanner / LspBanner / PtyBanner / WorkspaceBanner.
- **Severity enum** (Critical > Error > Warning > Info > Success) 으로 priority queue.
- **BannerStack** — 최대 3개 동시 표시, FIFO + severity priority sort, dedup.
- **Auto-dismiss 정책** — Critical/Error/Warning manual, Info 8초, Success 5초.
- **RootView 통합** — `banner_stack: Entity<BannerStack>` slot, top-of-window mount.
- **Mock event source wiring** — push_crash / push_update / push_lsp / push_pty / push_workspace helper 5종 (실제 system 통합은 별도 SPEC).

### 1.2 IMPLEMENTATION-NOTES.md §13.8 / §14 F 정합

`.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` v1.1:

- **§13.8** — 5 banner 변종의 색상/위치 명시 (Crash=danger, Update=brand.primary, LSP=info, PTY=success, Workspace=brand.primary.dark).
- **§14 F** — `crates/moai-studio-ui/src/banners/` 신규 모듈, P1 우선도.

본 SPEC 은 §13.8 의 **색상 매핑은 시안 reference** 로만 활용하고, **severity 분류는 본 SPEC 의 5단계 enum** 을 정본으로 한다 (시안의 색상은 token 표현 — Crash 의 사용자 인식 severity 는 critical, Update 는 info, etc.).

`.moai/design/from-claude-design/project/moai-revisions.jsx` 라인 552~635 의 5 컴포넌트 prototype 을 GPUI 시각 구현의 reference (icon + body + meta + actions 패턴).

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-014/research.md` — 코드베이스 분석, UX 패턴, severity priority, GPUI render, 위험.
- `.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` v1.1 §13.8 / §14 F.
- `.moai/design/from-claude-design/project/moai-revisions.jsx` 라인 552~635 — 5 banner 시안.
- `.moai/design/tokens.json` v2.0.0 — semantic 토큰 (DANGER/WARNING/INFO/SUCCESS).
- `crates/moai-studio-ui/src/design/{tokens,layout,runtime}.rs` — 기존 design module.
- `crates/moai-studio-ui/src/palette/mod.rs` — overlay mount 패턴 reference.
- `crates/moai-studio-ui/src/lib.rs` (RootView struct) — 통합 지점.

---

## 2. 배경 및 동기

본 섹션의 상세는 `research.md` §1 참조. 최소 맥락만 요약한다.

- moai-studio 는 IDE 형 데스크톱 앱이지만 **비차단 시스템 알림 채널이 부재**. agent crash / update / LSP/PTY 실패 / workspace 손상이 모두 silent.
- modal 은 workflow 차단, toast 는 위치 일관성 / multi-stack 불리 — IDE 표준 패턴인 **top-of-window slim banner stack** 을 채택 (VS Code, Zed, Sublime Merge 모두 동일).
- IMPLEMENTATION-NOTES.md §14 F 가 P1 후속 작업으로 명시.
- v0.1.0 release-blocker 는 아니지만, **moai-supervisor crash 채널** 부재는 사용자 경험 저하 주요 원인 — v0.1.0 직전/직후 정렬.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `crates/moai-studio-ui/src/banners/` 신규 모듈 도입.
- G2. `Severity` enum (Critical > Error > Warning > Info > Success) 정의 + Ord impl.
- G3. `BannerKind` enum (Crash / Update / Lsp / Pty / Workspace) 5 variants 정의.
- G4. `BannerView` Entity — 개별 banner UI (icon + strong + meta + actions + dismiss).
- G5. `BannerStack` Entity — 최대 3개 동시, priority queue + FIFO + dedup.
- G6. Auto-dismiss 정책 — Critical/Error/Warning manual, Info 8초, Success 5초.
- G7. Banner 의 dismiss 경로 — × 버튼, 우선 action 클릭, auto-dismiss timer 만료.
- G8. RootView 에 `banner_stack: Entity<BannerStack>` 필드 추가, top-of-window mount, TabContainer 위.
- G9. Mock event source helpers — `push_crash` / `push_update` / `push_lsp` / `push_pty` / `push_workspace` (BannerStack public API).
- G10. design::tokens 활용 — semantic.{DANGER/WARNING/INFO/SUCCESS} + brand.primary (action button), 신규 BANNER_HEIGHT_PX/BANNER_PADDING_X 등 보조 토큰 추가 가능 (선택).
- G11. ActiveTheme (V3-013 MS-3) 정합 — dark/light 모드 자동 감지, severity color 는 mode-agnostic.
- G12. 단위 테스트 coverage 85%+ 가능 구조 (severity ordering, stack policy, auto-dismiss state machine, 5 variant defaults, RootView 통합).
- G13. Local 5 quality gates 통과 가능: cargo test PASS, clippy 0 warning, fmt PASS, bench 회귀 없음, cargo check --release PASS.

### 3.2 비목표 (Non-Goals)

- N1. **실제 system event source 통합** — moai-supervisor crash event subscribe, moai-adk update check, LSP/PTY/Workspace 실제 error pipe 통합은 본 SPEC 비목표. mock helper 만 제공. v0.2.0+ 별 SPEC.
- N2. **Action 핸들러 실제 dispatch** — "Reopen" / "Update" / "Configure" / "Restart Terminal" / "Reset Workspace" 등 action 의 실제 동작은 mock (log::info! 만). 실제 wire-up 은 별도 SPEC.
- N3. **추가 variant** — 본 SPEC 은 5 variant 고정. 6번째+ variant 는 별도 SPEC.
- N4. **Banner 의 토스트 / 코너 모드** — 본 SPEC 은 top-of-window slim banner 만. 토스트 / 우하단 corner notification 은 비목표.
- N5. **Multi-line body / rich content** — 본 SPEC 은 strong + meta sub-line 1행만. multi-paragraph / 인라인 progress bar / 인라인 input 은 비목표.
- N6. **사용자 환경설정 통합** — banner 의 사용자 토글 / sound / 다크/라이트 무시 등 settings 통합은 본 SPEC 비목표 (V3-013 SettingsModal 의 advanced pane 에 향후 추가 가능).
- N7. **i18n / 다국어 텍스트** — 본 SPEC 은 영어 default text 만. 다국어는 v0.2.0+ i18n SPEC.
- N8. **Animation / transition** — banner mount/dismiss 의 fade/slide animation 은 v0.2.0+ polish SPEC.

### 3.3 비목표 명시 (Exclusions)

[HARD] 본 SPEC 의 Exclusion 목록 (manager-spec 규칙 준수):

- **EX-1**. 실제 supervisor IPC / channel subscribe 코드 (mock helper API 만)
- **EX-2**. moai-adk update 서버 polling 로직 (mock helper 만)
- **EX-3**. LSP / PTY 의 실제 error stream 구독 (V3-006 LspProvider mock 결과를 helper 로 호출)
- **EX-4**. workspace persistence corruption 자동 감지 (mock helper 만)
- **EX-5**. Banner 클릭 시 실제 시스템 동작 (모든 action handler 는 log::info! 또는 println!)
- **EX-6**. CSS animation / GPUI transition (instant mount / dismiss)
- **EX-7**. i18n / locale 별 텍스트
- **EX-8**. Banner 우선순위 사용자 커스터마이징 / Settings 통합

---

## 4. Stakeholder 및 사용자 시나리오

### 4.1 1차 사용자

- **moai-studio end user** — moai-studio 를 IDE 로 사용하는 개발자. 비차단 시스템 알림 인지가 1차 가치.

### 4.2 시나리오

**S-1 (Crash detection)**: 사용자가 `/moai run SPEC-V3-014` 실행 중. agent process 가 sigsegv 로 crash. moai-supervisor 가 crash event 발행 (mock 으로 가정). BannerStack 이 CrashBanner (Critical) 를 top 에 표시. 사용자가 "Reopen" 클릭 → log::info!("crash:reopen") 출력 (mock). 사용자가 × 클릭 → banner dismiss.

**S-2 (Update available)**: 앱 시작 시 update check (mock). v0.2.0 가용. UpdateBanner (Info) 가 표시. 8초 후 자동 dismiss. 사용자가 그 사이 "Update" 클릭 → log::info!("update:install") (mock).

**S-3 (LSP failure)**: 사용자가 .rs 파일 open. rust-analyzer spawn 실패 (mock LspProvider error). LspBanner (Warning) 표시. 수동 dismiss 까지 유지.

**S-4 (Multiple banners stacking)**: 동시에 Crash + Update + LSP 발생. Stack 이 priority 정렬: Crash(top) > LSP > Update(bottom). 모두 표시 (3개 cap). 추가 PtyBanner 발생 → Update 가 evict (PTY=Error > Update=Info), stack: Crash > PTY > LSP.

**S-5 (Auto-dismiss timer)**: UpdateBanner 표시 중. 사용자 무반응. 8초 경과 후 자동 dismiss. stack 에서 제거, 다른 banner 가 자동 위로 shift.

---

## 5. 핵심 요구사항 그룹 (Requirement Groups)

| ID | Group | 핵심 |
|----|-------|------|
| RG-V14-1 | banners 모듈 구조 | 신규 모듈 + 9 파일 + Severity/BannerKind enum |
| RG-V14-2 | BannerView | 개별 banner UI (icon/body/meta/actions/dismiss) |
| RG-V14-3 | BannerStack | 최대 3개, priority + FIFO + dedup, push/dismiss/tick |
| RG-V14-4 | Auto-dismiss | severity 별 정책, state machine 순수 함수 |
| RG-V14-5 | 5 Variants | Crash/Update/Lsp/Pty/Workspace default text/icon/actions |
| RG-V14-6 | RootView 통합 | top-of-window mount, TabContainer 위, Scrim/Settings 아래 |
| RG-V14-7 | Mock helper API | push_crash / push_update / push_lsp / push_pty / push_workspace |

---

## 6. EARS 요구사항 (Requirements)

본 섹션은 EARS (Easy Approach to Requirements Syntax) 5 패턴 (Ubiquitous / Event-driven / State-driven / Optional / Unwanted) 을 사용한다.

### 6.1 RG-V14-1 — banners 모듈 구조 (REQ-V14-001 ~ REQ-V14-005)

**REQ-V14-001 (Ubiquitous)**: The system shall provide a `crates/moai-studio-ui/src/banners/` module exposing `Banner`, `Severity`, `BannerKind`, `BannerStack`, `BannerView`, `BannerId`, `ActionButton`.

**REQ-V14-002 (Ubiquitous)**: The system shall define `Severity` as an enum with five variants ordered `Critical > Error > Warning > Info > Success`, implementing `Ord` and `PartialOrd`.

**REQ-V14-003 (Ubiquitous)**: The system shall define `BannerKind` as an enum with five variants `{ Crash, Update, Lsp, Pty, Workspace }`, each carrying variant-specific payload data.

**REQ-V14-004 (Ubiquitous)**: The system shall define `BannerId` as an opaque, `Eq + Hash` identifier used for dedup and dismiss targeting.

**REQ-V14-005 (Ubiquitous)**: The system shall define `ActionButton { label: String, primary: bool, handler: Box<dyn Fn(&mut Context)> }` for banner actions.

### 6.2 RG-V14-2 — BannerView (REQ-V14-006 ~ REQ-V14-010)

**REQ-V14-006 (Ubiquitous)**: The system shall render each banner with a fixed height of 36 pixels (BANNER_HEIGHT_PX), full window width, no border radius.

**REQ-V14-007 (Ubiquitous)**: The system shall render each banner left-to-right as `[icon (16px)] [strong + meta (flex)] [action buttons] [dismiss × button]`, with horizontal padding 12px (= spacing::S_3) and inter-element gap 8px (= spacing::S_2).

**REQ-V14-008 (Ubiquitous)**: The system shall map severity to background color using semantic tokens: `Critical → semantic::DANGER`, `Error → semantic::DANGER`, `Warning → semantic::WARNING`, `Info → semantic::INFO`, `Success → semantic::SUCCESS`.

**REQ-V14-009 (Ubiquitous)**: The system shall render the primary action button with `brand::PRIMARY` (light) or `brand::PRIMARY_DARK` (dark) background, dispatched via `design::runtime::ActiveTheme::theme`.

**REQ-V14-010 (Event-driven)**: When the user clicks the dismiss × button on a banner, the system shall remove that banner from the stack and trigger `cx.notify()` for re-render.

### 6.3 RG-V14-3 — BannerStack (REQ-V14-011 ~ REQ-V14-016)

**REQ-V14-011 (Ubiquitous)**: The system shall enforce a maximum of 3 simultaneously displayed banners in `BannerStack`.

**REQ-V14-012 (Event-driven)**: When `BannerStack::push(banner)` is called and the stack has fewer than 3 banners, the system shall insert the new banner sorted by severity (highest priority first), with FIFO order within the same severity tier.

**REQ-V14-013 (Event-driven)**: When `BannerStack::push(banner)` is called while the stack is full and the new banner's severity is strictly greater than the lowest-priority banner currently in the stack, the system shall evict the oldest banner of that lowest priority and insert the new banner sorted by severity.

**REQ-V14-014 (Event-driven)**: When `BannerStack::push(banner)` is called while the stack is full and the new banner's severity is less than or equal to the lowest priority in the stack, the system shall drop the new banner without modifying the stack.

**REQ-V14-015 (Event-driven)**: When `BannerStack::push(banner)` is called and a banner with the same `BannerId` is already in the stack, the system shall ignore the push (dedup).

**REQ-V14-016 (Event-driven)**: When `BannerStack::dismiss(id)` is called with an id present in the stack, the system shall remove the matching banner and shift remaining banners up.

### 6.4 RG-V14-4 — Auto-dismiss (REQ-V14-017 ~ REQ-V14-020)

**REQ-V14-017 (Ubiquitous)**: The system shall define auto-dismiss durations per severity: `Critical → None`, `Error → None`, `Warning → None`, `Info → 8s`, `Success → 5s`.

**REQ-V14-018 (Ubiquitous)**: The system shall expose a pure function `should_dismiss(mounted_at: Instant, auto_dismiss_after: Option<Duration>, now: Instant) -> bool` returning `true` iff `auto_dismiss_after.is_some() && now.duration_since(mounted_at) >= auto_dismiss_after.unwrap()`.

**REQ-V14-019 (State-driven)**: While a banner is mounted with `auto_dismiss_after = Some(d)`, the system shall periodically (poll interval ≤ 250ms) evaluate `should_dismiss` and dismiss the banner when the function returns `true`.

**REQ-V14-020 (Unwanted)**: If `auto_dismiss_after = None`, then the system shall not auto-dismiss the banner regardless of elapsed time; only manual dismiss (× button or action click) is permitted.

### 6.5 RG-V14-5 — 5 Variants (REQ-V14-021 ~ REQ-V14-025)

**REQ-V14-021 (Ubiquitous)**: The system shall define `CrashBanner` with `severity = Critical`, default strong text `"Agent crashed"`, optional meta `"<log path> · last alive <duration>"`, and actions `[ "Reopen" (primary), "Dismiss" ]`. `auto_dismiss_after = None`.

**REQ-V14-022 (Ubiquitous)**: The system shall define `UpdateBanner` with `severity = Info`, default strong text `"Update v{x.y.z} available"`, optional meta `"<size> · changelog →"`, and actions `[ "Update" (primary), "Later" ]`. `auto_dismiss_after = Some(8s)`.

**REQ-V14-023 (Ubiquitous)**: The system shall define `LspBanner` with `severity = Warning`, default strong text `"<server> failed to start"`, optional meta `"<error reason>"`, and actions `[ "Configure" (primary), "Dismiss" ]`. `auto_dismiss_after = None`.

**REQ-V14-024 (Ubiquitous)**: The system shall define `PtyBanner` with `severity = Error`, default strong text `"Terminal failed to spawn"`, optional meta `"<error code> · cwd <path>"`, and actions `[ "Restart Terminal" (primary), "Dismiss" ]`. `auto_dismiss_after = None`.

**REQ-V14-025 (Ubiquitous)**: The system shall define `WorkspaceBanner` with `severity = Warning`, default strong text `"Workspace state corrupted"`, optional meta `"<bak path>"`, and actions `[ "Reset Workspace" (primary), "Continue" ]`. `auto_dismiss_after = None`.

### 6.6 RG-V14-6 — RootView 통합 (REQ-V14-026 ~ REQ-V14-027)

**REQ-V14-026 (Ubiquitous)**: The system shall add `banner_stack: Entity<BannerStack>` field to `RootView` (in `crates/moai-studio-ui/src/lib.rs`), instantiated in `RootView::new` with empty stack.

**REQ-V14-027 (Ubiquitous)**: The system shall render `banner_stack` as the topmost child in `RootView::render` (above `TabContainer`, below palette/settings overlays). Banner stack height pushes `TabContainer` down (normal flow, not overlay).

### 6.7 RG-V14-7 — Mock helper API (REQ-V14-028)

**REQ-V14-028 (Ubiquitous)**: The system shall expose 5 public helper methods on `BannerStack`:
  - `push_crash(log_path: PathBuf, last_alive: Duration, cx: &mut Context<Self>)`
  - `push_update(version: String, size: String, cx: &mut Context<Self>)`
  - `push_lsp(server: String, error: String, cx: &mut Context<Self>)`
  - `push_pty(error_code: i32, cwd: PathBuf, cx: &mut Context<Self>)`
  - `push_workspace(bak_path: Option<PathBuf>, cx: &mut Context<Self>)`

Each helper constructs the corresponding `BannerKind` variant with mock `ActionButton` handlers (each handler invokes `log::info!("<kind>:<action>")` only — no real system dispatch) and calls `BannerStack::push` internally.

---

## 7. Acceptance Criteria (AC)

[HARD] 모든 AC 는 unit test 또는 통합 테스트로 검증 가능해야 한다 (observable evidence).

### AC-V14-1 (RG-V14-1) — Module structure
Given moai-studio-ui crate compiles, When the crate is built with `cargo build -p moai-studio-ui`, Then `banners` module is publicly exposed with `Banner`, `Severity`, `BannerKind`, `BannerStack`, `BannerView`, `BannerId`, `ActionButton` symbols.

**Evidence**: `cargo doc -p moai-studio-ui` succeeds; `cargo test -p moai-studio-ui banners::` finds tests under the module.

### AC-V14-2 (RG-V14-1) — Severity ordering
Given `Severity` enum, When `vec![Info, Critical, Warning, Success, Error].sort_by(|a, b| b.cmp(a))` is executed, Then the result is `[Critical, Error, Warning, Info, Success]`.

**Evidence**: Unit test `severity_ordering_descending` passes.

### AC-V14-3 (RG-V14-3) — Stack push under capacity
Given an empty `BannerStack`, When `push_crash`, `push_lsp`, `push_update` are called in order, Then `stack.banners().len() == 3` and the order is `[Crash(Critical), Lsp(Warning), Update(Info)]`.

**Evidence**: Unit test `stack_push_three_priority_ordered` passes.

### AC-V14-4 (RG-V14-3) — Stack evict on priority increase
Given a stack already full with `[Update(Info), Update2(Info), Lsp(Warning)]`, When `push_crash` is called (Critical), Then the oldest Info-tier banner (Update) is evicted and the resulting stack is `[Crash(Critical), Lsp(Warning), Update2(Info)]`.

**Evidence**: Unit test `stack_evict_lowest_priority_oldest` passes.

### AC-V14-5 (RG-V14-3) — Stack drop on equal priority full
Given a stack full with `[Crash(Critical), Pty(Error), Lsp(Warning)]`, When `push_workspace` is called (Warning, equal to lowest), Then the new banner is dropped and the stack is unchanged.

**Evidence**: Unit test `stack_drop_on_equal_priority_when_full` passes.

### AC-V14-6 (RG-V14-3) — Stack dedup by id
Given a stack containing one `LspBanner` with `id = "lsp:rust-analyzer"`, When a second `LspBanner` with the same id is pushed, Then the stack length remains 1 and no change occurs.

**Evidence**: Unit test `stack_dedup_same_id` passes.

### AC-V14-7 (RG-V14-3) — Dismiss by id
Given a stack with `[Crash(c1), Lsp(l1), Update(u1)]`, When `dismiss(l1)` is called, Then the stack is `[Crash(c1), Update(u1)]` and `cx.notify()` was triggered.

**Evidence**: Unit test `dismiss_removes_target_and_notifies` passes.

### AC-V14-8 (RG-V14-4) — Auto-dismiss state machine
Given `should_dismiss(mounted_at, auto_dismiss_after, now)`, When called with `mounted_at = T0`, `auto_dismiss_after = Some(8s)`, `now = T0 + 7.999s`, Then the result is `false`. When called with `now = T0 + 8.001s`, Then the result is `true`. When called with `auto_dismiss_after = None` and `now = T0 + 1h`, Then the result is `false`.

**Evidence**: Unit test `should_dismiss_truth_table` covers all 4 cases above.

### AC-V14-9 (RG-V14-5) — 5 variants default specs
Given each of `CrashBanner`, `UpdateBanner`, `LspBanner`, `PtyBanner`, `WorkspaceBanner`, When constructed with default mock data, Then severity, action count (= 2), primary action label, and `auto_dismiss_after` match REQ-V14-021 through REQ-V14-025 verbatim.

**Evidence**: Unit tests `<variant>_default_spec` (5 tests, one per variant) pass.

### AC-V14-10 (RG-V14-2) — BannerView render contract
Given a `BannerView` Entity, When rendered, Then the output element tree contains:
  - 1 leftmost icon element of size 16×16
  - 1 strong text element followed by 0 or 1 meta sub-element
  - N action button elements (N = banner.actions().len())
  - 1 rightmost dismiss × button element

**Evidence**: Render snapshot test or DOM-equivalent test asserts element count and ordering.

### AC-V14-11 (RG-V14-6) — RootView slot integration
Given the modified RootView struct, When `RootView::new` is invoked, Then `banner_stack` is initialized with empty stack. When `RootView::render` is invoked, Then `banner_stack` is the first child in the v_flex tree, above TabContainer.

**Evidence**: Unit test `rootview_has_banner_stack_topmost` passes.

### AC-V14-12 (RG-V14-7) — Mock helper API end-to-end
Given a `BannerStack` Entity owned by RootView, When `banner_stack.update(cx, |s, cx| s.push_crash("/tmp/log".into(), Duration::from_secs(12), cx))` is called, Then the stack contains exactly 1 CrashBanner with `severity = Critical`, default strong text, and actions `[Reopen (primary), Dismiss]`.

**Evidence**: Integration test `push_crash_helper_constructs_crash_banner` passes.

### AC-V14-13 (Quality gates) — Local 5 gates
Given the implementation merged into a feature branch, When the local quality script runs:
  - `cargo test -p moai-studio-ui` — all tests pass
  - `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` — 0 warnings
  - `cargo fmt -- --check` — PASS
  - `cargo bench -p moai-studio-ui` (smoke) — no regression > 5% on existing benches
  - `cargo check --release -p moai-studio-ui` — PASS

Then all 5 gates report success.

**Evidence**: CI run (`.github/workflows/ci-rust.yml`) GREEN on the feature branch PR.

---

## 8. 마일스톤 (Milestones — TDD 3-MS breakdown)

[HARD] 모든 milestone 은 RED → GREEN → REFACTOR 사이클 + Local 5 gates 통과 후 commit. 시간 추정 금지 (priority 만).

### MS-1 — Banner core (Severity + BannerKind + BannerView + BannerStack)

**우선도**: P1 (필수, 차순위 milestone 의 prerequisite)

**Scope**:

- `banners/mod.rs` — Severity enum (Ord) + BannerKind enum + BannerId + ActionButton struct + Banner trait/extension methods + module re-exports.
- `banners/banner_view.rs` — BannerView Entity + Render impl (icon/body/meta/actions/dismiss layout).
- `banners/banner_stack.rs` — BannerStack Entity + push/dismiss/tick/should_dismiss + dedup HashSet + Vec<BannerView Entity> management.

**Coverage**:
- REQ-V14-001 ~ REQ-V14-016, REQ-V14-018, REQ-V14-020
- AC-V14-1 ~ AC-V14-8, AC-V14-10

**RED phase tests** (작성 후 빨간색 확인):
- `severity_ordering_descending`
- `severity_partial_ord`
- `banner_id_eq_hash`
- `stack_push_under_capacity_ordered`
- `stack_push_three_priority_ordered`
- `stack_evict_lowest_priority_oldest`
- `stack_drop_on_equal_priority_when_full`
- `stack_dedup_same_id`
- `dismiss_removes_target_and_notifies`
- `should_dismiss_truth_table` (4 cases)
- `bannerview_renders_icon_body_actions_dismiss`

**GREEN phase**: 위 테스트 모두 통과시키는 최소 구현.

**REFACTOR phase**: dedup HashSet vs HashMap, Vec sort 알고리즘 (BinaryHeap 검토), tick 의 GPUI timer 패턴 검증.

**Local 5 gates**: PASS 필수.

### MS-2 — 5 Variants (Crash / Update / Lsp / Pty / Workspace)

**우선도**: P1 (MS-1 통과 후)

**Scope**:

- `banners/variants/mod.rs` — 5 variant 모듈 re-export.
- `banners/variants/crash.rs` — CrashBanner 구조체 + BannerKind::Crash payload + default text/icon/actions + handler mock (log::info!).
- `banners/variants/update.rs` — UpdateBanner 동일 패턴.
- `banners/variants/lsp.rs` — LspBanner.
- `banners/variants/pty.rs` — PtyBanner.
- `banners/variants/workspace.rs` — WorkspaceBanner.

각 variant 파일은 200~300 LOC 예상.

**Coverage**:
- REQ-V14-021 ~ REQ-V14-025
- AC-V14-9

**RED phase tests**:
- `crash_banner_default_spec` — severity = Critical, action count = 2, primary label = "Reopen", auto_dismiss_after = None.
- `update_banner_default_spec` — severity = Info, primary label = "Update", auto_dismiss_after = Some(8s).
- `lsp_banner_default_spec` — severity = Warning, primary label = "Configure", auto_dismiss_after = None.
- `pty_banner_default_spec` — severity = Error, primary label = "Restart Terminal", auto_dismiss_after = None.
- `workspace_banner_default_spec` — severity = Warning, primary label = "Reset Workspace", auto_dismiss_after = None.
- `each_variant_action_handler_emits_log` — mock handler 가 log::info! 호출 (test_log crate 또는 mock).

**GREEN phase**: 5 variant 의 minimal 구현.

**REFACTOR phase**: variant 간 공통 helper 추출 (e.g. `make_dismiss_action()` shared by all variants).

**Local 5 gates**: PASS 필수.

### MS-3 — RootView 통합 + Mock helper API + 통합 테스트

**우선도**: P1 (MS-2 통과 후)

**Scope**:

- `crates/moai-studio-ui/src/lib.rs` — RootView struct 에 `banner_stack: Entity<BannerStack>` 추가, `RootView::new` 에서 instantiate, `RootView::render` 에서 v_flex 의 첫 child 로 mount (TabContainer 위).
- `banners/banner_stack.rs` 확장 — public helper 5종 (`push_crash` / `push_update` / `push_lsp` / `push_pty` / `push_workspace`) 추가. 각 helper 는 BannerKind variant 생성 + mock action handler + push 호출.
- 통합 테스트 — `crates/moai-studio-ui/tests/banner_integration.rs` (또는 lib.rs 의 #[cfg(test)] 모듈) 에 RootView + BannerStack 통합 시나리오.

**Coverage**:
- REQ-V14-017 (auto-dismiss durations 정합), REQ-V14-019 (poll interval ≤ 250ms), REQ-V14-026 ~ REQ-V14-028
- AC-V14-11, AC-V14-12, AC-V14-13

**RED phase tests**:
- `rootview_has_banner_stack_topmost` — RootView 에 banner_stack 필드 + render 시 topmost child 검증.
- `push_crash_helper_constructs_crash_banner` — helper 호출 → stack 에 CrashBanner 1개.
- `push_update_helper_constructs_update_banner` — 동일 패턴.
- `push_lsp_helper_constructs_lsp_banner`.
- `push_pty_helper_constructs_pty_banner`.
- `push_workspace_helper_constructs_workspace_banner`.
- `multiple_banners_render_in_priority_order` — Crash + Update + LSP push → render 순서 검증.
- `auto_dismiss_info_after_8s_smoke` — UpdateBanner mount, mock clock 8.001s 진행, dismiss 검증 (test 용 가짜 clock 또는 should_dismiss 직접 호출).

**GREEN phase**: RootView 수정 + 5 helper + 통합 테스트 통과.

**REFACTOR phase**: BannerStack 의 tick 메서드 위치 (RootView 가 호출 vs 자체 spawn), helper API 의 인자 단순화, dedup id 생성 규칙 (e.g. "lsp:<server-name>", "update:<version>") 정합.

**Local 5 gates**: PASS 필수. 특히 cargo bench smoke check (RootView render path 회귀 < 5%).

---

## 9. 파일 인벤토리 (File Inventory)

### 9.1 신규 파일

```
crates/moai-studio-ui/src/banners/
├── mod.rs                  // ~150 LOC — Severity, BannerKind, BannerId, ActionButton, re-exports
├── banner_view.rs          // ~250 LOC — BannerView Entity, Render impl
├── banner_stack.rs         // ~400 LOC — BannerStack Entity, push/dismiss/tick, 5 helpers (MS-3)
└── variants/
    ├── mod.rs              // ~30 LOC — 5 variant re-exports
    ├── crash.rs            // ~180 LOC — CrashBanner default + mock handler
    ├── update.rs           // ~180 LOC — UpdateBanner
    ├── lsp.rs              // ~180 LOC — LspBanner
    ├── pty.rs              // ~180 LOC — PtyBanner
    └── workspace.rs        // ~180 LOC — WorkspaceBanner
```

총 9 파일, ~1730 LOC 예상.

### 9.2 수정 파일

| 파일 | 수정 내용 | 영향 |
|------|----------|------|
| `crates/moai-studio-ui/src/lib.rs` | RootView 에 `banner_stack: Entity<BannerStack>` 필드 추가, `RootView::new` instantiate, `RootView::render` 에서 v_flex topmost child mount, `pub mod banners;` 선언 | MS-3, ~30 LOC 추가 |

### 9.3 선택 (별도 PR 가능)

| 파일 | 수정 내용 | 영향 |
|------|----------|------|
| `crates/moai-studio-ui/src/design/tokens.rs` | `pub mod banner { BANNER_HEIGHT_PX, BANNER_PADDING_X_PX, BANNER_ICON_SIZE_PX, BANNER_ACTION_GAP_PX }` 추가 | 선택 — banner 가 spacing/layout const 직접 사용해도 무방. 신규 module 추가는 가독성 향상 목적. |
| `.moai/design/tokens.json` | `round2_component.banner` 섹션 추가 (Round 2 시안 정렬) | 선택 — 별도 PR 권장 |

### 9.4 신규 테스트 파일

```
crates/moai-studio-ui/src/banners/banner_stack.rs  // #[cfg(test)] mod tests — MS-1 stack 테스트
crates/moai-studio-ui/src/banners/banner_view.rs   // #[cfg(test)] mod tests — MS-1 render 테스트
crates/moai-studio-ui/src/banners/mod.rs           // #[cfg(test)] mod tests — MS-1 severity/id 테스트
crates/moai-studio-ui/src/banners/variants/*.rs    // #[cfg(test)] mod tests — MS-2 variant 별 default 테스트
crates/moai-studio-ui/tests/banner_integration.rs  // MS-3 통합 테스트 (RootView + helpers)
```

---

## 10. 의존성 (Dependencies)

### 10.1 SPEC 의존

| SPEC | 사용 범위 | 본 SPEC 가 변경 |
|------|----------|----------------|
| SPEC-V3-001 (RootView/TabContainer/design module) | RootView 통합 지점, design::tokens / layout / runtime 활용 | RootView 에 1 필드 + render 1 child 추가 (additive) |
| SPEC-V3-004 (Keymap dispatcher) | Action 핸들러 dispatch 인프라 (mock 단계는 미사용) | 없음 |
| SPEC-V3-006 MS-3a (LspProvider mock) | LspBanner 의 source 시뮬레이션 | 없음 (helper API 만 사용) |
| SPEC-V3-012 (palette overlay 패턴) | overlay vs banner stack 의 z-index 구분 reference | 없음 |
| SPEC-V3-013 (ActiveTheme global) | dark/light 분기 + accent 색상 | 없음 (read-only 사용) |

### 10.2 외부 crate 의존

추가 외부 의존 **없음** — 모든 기능을 기존 stack 으로 구현:

- `gpui` 0.1 — Entity / Render / Context
- `std::time::{Instant, Duration}` — auto-dismiss state
- `std::collections::HashSet` — dedup
- `log` (이미 workspace 의존) — mock action handler

---

## 11. 비기능 요구사항 (Non-functional)

- **성능**: BannerStack push/dismiss 는 O(stack_size) ≤ O(3) — 무시 가능. RootView render path 에 추가 v_flex 1 노드 — bench 회귀 < 5% 보장.
- **메모리**: 최대 3 BannerView Entity (각 ~수 KB) — 무시 가능.
- **접근성**: 모든 banner 는 키보드 dismiss 가능해야 한다 (Esc 단축키 binding 은 본 SPEC 비목표 — Round 2 polish SPEC 으로 분리. × 버튼은 클릭 가능).
- **국제화**: default text 는 영어. i18n 비목표 (N7).
- **테스트 커버리지**: 단위 테스트 85%+, banners/ 모듈 라인 기준.
- **clippy**: 0 warning (allow_unused 등 마스킹 금지).
- **fmt**: rustfmt PASS.

---

## 12. 위험 요소 및 완화

본 섹션의 상세는 `research.md` §5 참조. 요약:

| ID | Risk | 완화 |
|----|------|------|
| R-1 | GPUI timer 정확성 | should_dismiss 순수 함수로 분리, real timer 은 통합 테스트만 |
| R-2 | 동시 push race | GPUI Entity 의 single-threaded UI context 보장, 외부 source 는 channel forwarding |
| R-3 | Severity drop 정책의 UX 임팩트 | 동일 priority drop 명시 + 단위 테스트 |
| R-4 | Mock action 의 사용자 혼란 | 비목표 (N1, N2) 에 명시, log::info! 출력으로 디버깅 가능 |
| R-5 | bench 회귀 | RootView render path 에 1 v_flex child 추가만 — 미미. MS-3 종료 시 smoke check |
| R-6 | Brand 정합성 위반 | severity 색상은 semantic 토큰 (brand 와 독립), primary action 만 brand.primary 사용. [FROZEN] 위반 없음 |

---

## 13. 검증 체크리스트 (Verification)

[HARD] 본 SPEC 구현 완료 시 모든 항목 체크:

- [ ] `crates/moai-studio-ui/src/banners/` 모듈 9 파일 생성, 모두 cargo build 성공
- [ ] Severity enum Ord impl + 단위 테스트 PASS
- [ ] BannerKind 5 variant + payload struct 정의
- [ ] BannerStack max=3 + priority + FIFO + dedup + dismiss 단위 테스트 ALL PASS
- [ ] `should_dismiss` 순수 함수 truth table 4 case PASS
- [ ] 5 variant 의 default spec (severity / actions / auto_dismiss) 단위 테스트 PASS
- [ ] BannerView render 시 element 트리 4 영역 (icon/body/actions/dismiss) 검증
- [ ] RootView 에 banner_stack 필드 추가, render 시 topmost child
- [ ] 5 mock helper API 통합 테스트 PASS
- [ ] Local 5 gates GREEN: cargo test, clippy 0 warning, fmt, bench, cargo check --release
- [ ] cargo doc -p moai-studio-ui 성공 (publict API 문서화)
- [ ] [FROZEN] 모두의AI 청록 정합 — primary action button 만 brand.primary 사용, severity 는 semantic

---

## 14. 변경 이력 / Future SPECs

본 SPEC 완료 후 후속 SPEC 후보:

- **SPEC-V3-015 (가칭) — Banner system event source integration**: 실제 moai-supervisor crash event subscribe, moai-adk update check polling, LSP/PTY/Workspace 실제 error stream 통합. Mock helper 를 real source 로 교체.
- **SPEC-V3-XXX — Banner action dispatch**: 각 action ("Reopen", "Update", "Configure", "Restart Terminal", "Reset Workspace") 의 실제 시스템 동작 구현.
- **SPEC-V3-XXX — Banner animation polish**: mount/dismiss fade/slide transition.
- **SPEC-V3-XXX — Banner i18n**: 다국어 default text (ko/en).
- **SPEC-V3-XXX — Banner Settings integration**: SettingsModal Advanced pane 에 banner toggle / sound / 다크/라이트 제어.

---

Version: 1.0.0
Last Updated: 2026-04-26
Author: MoAI (manager-spec)
Source: IMPLEMENTATION-NOTES.md v1.1 §13.8 / §14 F (P1 후속작업)
