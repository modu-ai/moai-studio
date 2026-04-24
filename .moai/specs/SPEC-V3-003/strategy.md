---
spec_id: SPEC-V3-003
title: Tab/Pane Split — Run Phase 1 Strategy Report
version: 1.0.0
created_at: 2026-04-24
parent_spec: SPEC-V3-003
parent_version: 1.0.0
author: MoAI (manager-strategy, Run Phase 1 Analysis)
mode: Full Pipeline + TDD (quality.yaml development_mode: tdd)
harness: thorough
ultrathink: true
---

# SPEC-V3-003 Run Phase 1 Strategy Report

## 1. Executive Summary

본 SPEC-V3-003 은 MoAI Studio v3 의 Tab / Pane split 레이어를 `crates/moai-studio-ui/src/{panes,tabs}` 신규 + `crates/moai-studio-workspace/src/persistence.rs` 신규로 구축한다. 범위는 **8 신규 파일 + 4 수정 지점 (lib.rs 4개 + workspace 1개) + 4 테스트 파일 + 2 bench 파일 + 1 CI workflow** 의 총 13 작업 단위이며, SPEC-V3-002 Terminal Core 의 74 tests 는 **완전 무수정** 으로 재사용된다 (RG-P-7, AC-P-16). 실행은 14 tasks × 3 milestones (MS-1 Pane core T1-T7 → MS-2 Tabs T8-T11 → MS-3 Persistence T12-T14) 순서로, 각 milestone 완료 시점에 이전 milestone AC 전체 regression 을 통과해야 한다. 4 개의 Plan Spike (S1 GPUI divider drag API, S2 gpui-component Resizable, S3 PaneId/TabId 생성 방식, S4 Linux shell 관례 UX) 중 S1·S3·S4 는 T1 과 **병렬 착수** 가능하고 S2 는 S1 이 실패한 조건부 실행이며, S4 의 결과는 사용자 확정 결정을 요하는 이원 경로를 가진다 (현행 유지 vs Ctrl+Shift escalation). Critical risks 는 R-1 (GPUI 0.2.2 divider drag API 미검증 — S1 으로 해소) 과 R-9 (Linux Ctrl+D/W/\\ shell 관례 충돌 — S4 + 사용자 결정 필요) 두 건이며, 나머지 R-2 ~ R-8 은 설계 수준에서 완화 경로가 확보되어 있다. TDD 경로로 Run Phase 2 에 진입하며, 각 task 는 RED (failing test) → GREEN (minimum impl) → REFACTOR 의 cycle 을 한 번 돌고, drift guard + re-planning gate 가 3 consecutive zero-progress iteration 에서 작동한다.

---

## 2. Codebase Reality Check

### 2.1 spec.md §9.2 수정 대상 파일의 실제 라인 번호 재확인

plan.md 및 spec.md 의 라인 참조 정확도를 실제 `crates/moai-studio-ui/src/lib.rs` (719 LOC) 와 대조한 결과:

| plan.md 참조 | 실제 라인 | 일치 여부 | 비고 |
|--------------|-----------|-----------|------|
| `lib.rs:75` (`terminal: Option<Entity<TerminalSurface>>`) | line 69-76 `RootView` struct 전체, line 75 에 해당 필드 | **일치 (필드 단위)** | plan 이 필드만 좁게 지목, 전체 struct 는 line 69-76 |
| `lib.rs:184` (`let terminal = self.terminal.clone()`) | line 184 동일 | **완전 일치** | 무수정 |
| `lib.rs:290-299` (`main_body` 시그니처) | **실제는 line 286-300** (`fn main_body(...)` 선언 + body) | **경미한 불일치** | plan 이 body 만 지목, 시그니처 시작은 line 286. downstream agent 는 실제 파일 재확인 필요. |
| `lib.rs:410-444` (`content_area` 분기) | line 410-444 동일 | **완전 일치** | 무수정 |

추가 발견:
- `crates/moai-studio-ui/src/lib.rs` 는 719 LOC 로 이미 단일 파일 문턱을 초과하는 큰 파일. T7 의 수정은 tab_container 필드 교체 + main_body / content_area 분기 재배선에 그쳐야 하며, **drive-by refactor 금지** (Agent Core Behavior 5). 이 파일의 분할은 별도 SPEC 으로 연기.
- `crates/moai-studio-workspace/src/lib.rs` 는 393 LOC 로 T12 의 `persistence.rs` 또는 `panes_schema.rs` 신규 파일 추가 대상 — 기존 `lib.rs` 의 `WorkspacesStore` 는 무수정.
- `crates/moai-studio-terminal/**` 는 5 파일 (`events.rs`, `lib.rs`, `libghostty_ffi.rs`, `pty/*`, `vt.rs`, `worker.rs`) 전체 무수정 — RG-P-7.

### 2.2 SPEC-V3-002 재사용 대상 공개 API (exact names — downstream agent 가 그대로 호출)

본 SPEC-V3-003 구현 중 아래 심볼은 **as-is 호출만 허용, 변경 금지** (REQ-P-060, AC-P-16):

| Crate path | 심볼 | 종류 | 사용 시점 |
|------------|------|------|-----------|
| `moai_studio_terminal::events` | `PtyEvent` (Output / ProcessExit / Resize) | pub enum | T4 pane leaf 의 PTY 이벤트 수신 |
| `moai_studio_terminal::pty` | `Pty` | pub trait (feed / read_available / set_window_size / is_alive) | T4 pane 당 PTY 스폰 |
| `moai_studio_terminal::pty::unix` | `UnixPty` | pub struct | T4 macOS / Linux shell spawn |
| `moai_studio_terminal::pty::windows` | `ConPtyStub` | pub struct | non-goal (N10), 빌드만 유지 |
| `moai_studio_terminal::pty::mock` | `MockPty` | pub struct (test only) | T1-T4 단위 테스트 |
| `moai_studio_terminal::worker` | `PtyWorker` (`new()`, `run(pty, tx)`) | pub struct | T4 pane 당 worker spawn |
| `moai_studio_terminal::worker` | `AdaptiveBuffer`, `BufferSize` | pub struct/enum | 선택 — pane 단위 통계 수집 시 |
| `moai_studio_terminal::vt` | `VtTerminal` (`new(cols, rows)`, `feed`, `render_state`, `resize`) | pub struct | T4 pane 당 VT state |
| `moai_studio_terminal::libghostty_ffi` | `FfiError`, `TerminalHandle`, `RenderSnapshot` | pub struct/enum | T4/T13 render pipeline |
| `moai_studio_ui::terminal` | `TerminalSurface` (`new`, `set_font_metrics`, `on_output`, `on_process_exit`, `pixel_to_cell`, `begin_selection`, `update_selection`, `end_selection`, `clear_selection`, `selection_text`, `handle_key_down`, `drain_pending_input`) | pub struct | T1 leaf content 재사용 |
| `moai_studio_ui::terminal` | `FontMetrics`, `Selection`, `TerminalState` | pub struct | T10 `TabBar` 에서 font 참조 가능 |
| `moai_studio_workspace` | `Workspace`, `WorkspacesStore`, `WorkspaceError`, `default_storage_path`, `pick_project_folder`, `pick_and_save` | pub types/fn | T12/T13 workspace id 참조 전용 (파일 경로는 별도 분리) |

**중요**: Terminal Core 의 74 tests 에 대한 CI gate (AC-P-16) 는 매 PR 에서 실행되며, 이 API 중 하나라도 시그니처가 변경되면 본 SPEC PR 은 **자동 rejection**.

### 2.3 재사용 가능한 기존 패턴 (reuse, NOT reinvent)

downstream agent 는 아래 패턴을 신규 발명하지 말고 기존 코드베이스에서 **패턴 인용** 으로 차용해야 한다:

1. **`$schema` versioned JSON 패턴** (`workspace/src/lib.rs:81-91`)
   - `#[serde(rename = "$schema", default = "default_schema")]` + `fn default_schema() -> String`
   - T12 의 `panes-v1` schema 직렬화에 동일 패턴 적용 가능.
   - 단, atomic write 는 기존 `WorkspacesStore::save` 가 쓰지 않으므로 새 helper 가 필요 (REQ-P-052 atomic rename 강제).

2. **`~/.moai/studio/...` HOME/APPDATA 경로 분기 패턴** (`workspace/src/lib.rs:190-205`)
   - `default_storage_path()` 의 `#[cfg(windows)]` vs `#[cfg(not(windows))]` 분기.
   - T12 의 `panes-{ws-id}.json` 경로 구성에 동일 패턴 적용. `Windows` branch 는 non-goal 이지만 컴파일 유지.

3. **ID 생성 패턴** (`workspace/src/lib.rs:60-67`)
   - `fn generate_id() -> String { format!("ws-{:x}", SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()) }`
   - Spike 3 에서 `format!("pane-{:04x}", counter)` vs nanos-based id 선택 시 **기존 패턴 (nanos + prefix)** 을 우선 검토. `uuid` crate 추가는 YAGNI 위반 가능성.

4. **@MX 주석 5-종 패턴** (`terminal/src/worker.rs:108-114`, `terminal/src/pty/mod.rs:1-6`, `terminal/src/vt.rs:3-5`)
   - `@MX:ANCHOR(name)` + `@MX:REASON(...)` 연쇄 패턴 + fan_in 설명.
   - plan.md §8 의 9+ ANCHOR / 3+ WARN / 7+ NOTE / 2+ TODO 목표에 직접 대응.

5. **`#[cfg(test)] mod tests` 인라인 유닛 + `tests/integration_*.rs` 통합 분리 패턴**
   - SPEC-V3-002 에서 확립된 규칙: 단위는 파일 내 inline, 통합은 `tests/` 하위 개별 파일.
   - T1 (`tree.rs::tests`) / T8 (`container.rs::tests`) / T10 (`bar.rs::tests`) 에 동일 적용.

6. **Tracing 3-level 로그 패턴** (`terminal/src/worker.rs:65-82`)
   - `info!(buffer_size = "64KB", "...")` structured tracing 패턴.
   - REQ-P-056 의 `tracing::warn!("pane cwd fallback: ... (reason: ...)")` 에 동일 구조화 권장.

7. **GPUI `Entity<T>` 소유 패턴** (`ui/src/lib.rs:75`)
   - `Option<Entity<terminal::TerminalSurface>>` → `Option<Entity<TabContainer>>` 로 1:1 교체.
   - downstream agent 는 `cx.new_entity(...)` 생성 패턴을 그대로 유지.

---

## 3. Proportionality & Simplicity Audit

### 3.1 Over-engineering 의심 지점 검증

| 지점 | 과잉 여부 | 판정 근거 |
|------|-----------|-----------|
| **T3 PaneSplitter trait** | **과잉 아님** | REQ-P-061 이 SPEC 수준에서 trait 제공을 의무화 + REQ-P-063 이 gpui-component 결정 연기를 강제. 즉 추상화는 SPEC 요구이지 Agent 의 추가 발명이 아니다. AC-P-17 이 직접 검증. |
| **T3 ResizableDivider trait** | **과잉 아님** | REQ-P-062 가 SPEC 수준 의무화. 동일 근거. |
| **T3 MockPaneSplitter `#[cfg(test)]`** | **경미한 위험** | plan 이 doc test 예제 + AC-P-17 compile-only check 목적으로 열거. production impl 과 중복 여지 있음. 완화: `#[cfg(test)]` 엄격 유지, 통합 테스트가 production impl (T4) 만 사용. |
| **T12 `PanesFile`/`SerializedPaneTree` 별도 구조** | **과잉 아님** | REQ-P-050 / REQ-P-055 가 workspaces.json 과의 schema 분리 + scrollback 부재 negative assertion 을 의무화. 기존 `Workspace` serde 재사용은 불가 (RG-P-6 요구 필드 불일치). |
| **T14 `ci-v3-pane.yml` 신규 workflow** | **경미한 과잉 의심** | 기존 `.github/workflows/ci-rust.yml` (존재 가정, 실제 확인 필요) 확장으로도 matrix job 추가 가능. 단 spec.md §11.3 A-2 의 per-milestone regression gate 가 본 SPEC 고유 요건이므로 분리 운영이 합리적. 판정: 유지. |

### 3.2 YAGNI 점검 — 불필요한 확장 포인트 제거

- **PaneId / TabId 생성 시 `uuid` crate 추가**: workspace 가 이미 `format!("ws-{:x}", nanos)` 를 쓰고 있으므로 uuid 의존은 YAGNI 위반. Spike 3 에서 기존 패턴 차용 경로를 우선 권장.
- **`PaneConstraints` 에 `pub const fn new(min_cols, min_rows)` 추가 유혹**: AC-P-21 이 negative API surface check 를 강제. 가변 API 추가 금지가 **SPEC 수준 계약** 임을 downstream agent 가 반드시 기억.
- **T5 `GpuiDivider` 의 `orientation` 필드**: plan 에서 `DividerOrientation` 을 언급하나, `PaneTree::SplitDirection` 과 1:1 대응되면 중복 enum 이다. 완화: `SplitDirection` 의 alias 또는 directly reuse 가 맞다. downstream agent 는 새 enum 도입 전에 SplitDirection 재사용 가능성 먼저 점검해야 한다.

### 3.3 Pattern reuse vs reinvention

- **Atomic write helper**: `WorkspacesStore::save` (`workspace/src/lib.rs:134-142`) 는 atomic write 를 쓰지 **않는다** (단순 `std::fs::write`). T12 는 tempfile + fsync + rename 을 새로 구현해야 하며 (REQ-P-052 강제), 이를 workspace crate 의 공용 helper 로 승격하는 것이 drive-by refactor 에 해당하여 **SPEC-V3-003 범위를 초과**. 별도 SPEC 에서 정리 권장. 본 SPEC 에서는 `crates/moai-studio-workspace/src/persistence.rs` 내부 helper 로 local 유지.
- **`default_storage_path()` HOME 경로 구성**: 재사용 가능. `panes-{ws-id}.json` 은 동일 디렉터리 `~/.moai/studio/` 에 위치하므로 `workspace/src/lib.rs:190-205` 의 pattern 을 import 해 사용 가능 (pub re-export 필요 시 `pub fn studio_config_dir()` 같은 helper 승격 고려 — 이 경우는 scope 내 최소 추가로 허용).

---

## 4. 4 Spike Placement Strategy

### 4.1 Spike 배치 개요

| Spike | 실행 시점 | 선행 조건 | Blocker 대상 | 병렬 가능성 |
|-------|-----------|-----------|--------------|-------------|
| **S1** GPUI divider drag API | Run Phase 2 **진입 직후** | 없음 | T4, T5 (구현체 경로 확정) | S3, S4 와 완전 병렬 |
| **S2** gpui-component Resizable | **S1 실패 조건부** | S1 FAIL | T4, T5 (대체 경로) | S1 후순위, 단독 실행 |
| **S3** PaneId / TabId 생성 방식 | Run Phase 2 **진입 직후** | 없음 | T1 의 `PaneId` 타입 정의 (경미한 재작업 범위) | S1, S4 와 완전 병렬 |
| **S4** Linux shell 관례 UX | Run Phase 2 **진입 직후** | 없음 | T6, T9 의 Linux 키 바인딩 확정 | S1, S3 와 완전 병렬. 사용자 결정 대기 단계 포함. |

### 4.2 Spike 별 termination criterion + fallback

#### S1: GPUI 0.2.2 divider drag API

- **산출물**: `crates/moai-studio-ui/examples/divider-spike.rs` (임시, mergeless branch)
- **성공 (PASS) criterion**:
  - 200 LOC 이하 예제 코드에서 2-pane horizontal split + divider drag 동작
  - drag 중 **60 fps 유지 (≤ 16.67 ms / frame)** — spec.md §6.1
  - GPUI 0.2.2 native API (mouse event + flex basis) 만 사용, 외부 crate 無
- **실패 (FAIL) criterion**:
  - 60 fps 미달 OR 코드 200 LOC 초과 OR 플랫폼별 (macOS/Linux) 동작 분기 필수
- **실패 시 경로**: **S2 자동 escalate** (gpui-component 검증).
- **산출 보고서 형식**:
  - spike 결론 (PASS/FAIL) + GPUI API 호출 스니펫 + fps 계측값 + macOS/Linux diff 기록
  - `docs/spikes/SPIKE-V3-003-01-gpui-divider.md` 로 보존 (run phase 산출물)
- **downstream 반영**: T4 의 `crates/moai-studio-ui/src/panes/splitter_gpui_native.rs` 파일명 확정 또는 `splitter_gpui_component.rs` 로 대체.

#### S2: longbridge/gpui-component Resizable / Dock 안정성

- **선행 조건**: S1 이 FAIL 일 때만 착수. S1 이 PASS 면 S2 는 skip.
- **산출물**: `crates/moai-studio-ui/examples/gpui-component-spike.rs` + `Cargo.toml` 에 일시적 `gpui-component = { git = "...", rev = "..." }` 추가 (branch 로 격리, merge 시 확정 rev pin)
- **성공 (PASS) criterion**:
  - `cargo build -p moai-studio-ui` 의존성 충돌 0
  - `Resizable::new([pane_a, pane_b])` 예제 60 fps 유지
  - upstream commit SHA pin 가능 (upstream 의 API churn 내성)
- **실패 (FAIL) criterion**:
  - GPUI 0.2.2 와 호환 불일치 OR 60 fps 미달 OR commit SHA pin 불가능 (moving target)
- **실패 시 경로**: **user escalation** — plan 전면 재조정 (manager-strategy 재위임 또는 GPUI 버전 업데이트 SPEC 신설).
- **downstream 반영**: T4 구체 구현체 경로 + `Cargo.toml` 의존성 pin.

#### S3: PaneId / TabId 생성 방식

- **산출물**: `docs/spikes/SPIKE-V3-003-03-id-generation.md` (측정 보고서)
- **비교 기준**:
  1. `uuid` crate (v4) — 16 byte, 충돌 확률 사실상 0, 외부 의존 추가
  2. `format!("pane-{:04x}", counter)` — 4-digit hex counter, 프로세스 로컬 고유, 순서 보장, 의존 제로
  3. `format!("pane-{:x}", SystemTime::now().as_nanos())` — 기존 workspace generate_id 패턴 차용, 의존 제로, 사람이 읽기 좋음
- **성공 criterion**:
  - 3 경로의 trade-off 표 작성 (의존 / 충돌 / 가독성 / serialize 크기)
  - 기존 workspace ID 와의 visual consistency 평가
- **권장 결과** (spike 전 예측): **경로 2 (counter) 또는 3 (nanos + prefix)** 이 YAGNI 및 기존 패턴 차용에 부합. uuid 는 저장 크기 + 의존 비용 대비 이익 부족.
- **downstream 반영**: T1 의 `PaneId` type alias + constructor 확정. T1 본 코드는 spike 완료 전에도 placeholder `PaneId(String)` 로 시작 가능 (경미한 타입 재작업 부담).

#### S4: Linux Ctrl+D / Ctrl+W / Ctrl+\\ shell 관례 UX 검증

- **산출물**: `docs/spikes/SPIKE-V3-003-04-linux-shell-conventions.md` (측정 보고서)
- **측정 항목** (v1.0.0 Nm-3 / R-9 해소):
  - (a) Ctrl+D 로 `exit` 대체 불가 → 사용자가 `exit\n` 타이핑 우회의 수용도
  - (b) Ctrl+W 의 `unix-word-rubout` 손실 → 실무 CLI 작업 빈도 (1 일 local 사용 로그 또는 개발자 인터뷰)
  - (c) Ctrl+\\ 의 SIGQUIT → 실무 사용 빈도 (거의 사용 안 됨 확인 / 반증)
- **이원 경로 결정** (`[USER-DECISION-REQUIRED: spike-4-linux-shell-path]`):
  - **경로 (a) 현행 유지**: host 바인딩 기본 활성, Shortcut Customization SPEC (Phase 5) 에서 사용자 override 제공. RG-P-4 Linux 컬럼 + AC-P-9b **무변경**.
  - **경로 (b) Shift-escalation**: Ctrl+D → Ctrl+Shift+D, Ctrl+W → Ctrl+Shift+W, Ctrl+\\ → Ctrl+Shift+\\ 로 이원. 이 경우 **annotation cycle 재개 필요** (spec.md §5 RG-P-4 표 + §6.4 + AC-P-9b 갱신), 플랜 일부 재작업 필요.
- **사용자 결정 시점**: Spike 4 측정 보고서 완성 후, downstream agent (manager-tdd) 가 T6 / T9 의 Linux 키 바인딩 구현에 진입하기 **직전**. 본 strategy.md 에서는 사전 확정 금지 (HARD 제약).
- **Fallback**: 사용자 결정이 지연되면 경로 (a) 를 기본값으로 진행하고, 이후 (b) 선택 시 T6/T9 의 키 바인딩 테이블만 minor patch 로 교체.

### 4.3 Spike 의존성 재확인 vs plan.md

plan.md §3 Spike 4건의 선행 관계:
- 원문: "S2 는 S1 결과에 따라 조건부 실행"
- 본 strategy 확인: 일치. 추가로 S2 실패 시 user escalation 경로 명시.
- plan.md §5 의존성 그래프: S1 / S3 / S4 를 "독립적으로 병렬 실행 가능" 으로 명시 — 본 strategy 동의.
- 수정 사항: **없음**. plan.md §5 의 그래프를 그대로 계승.

---

## 5. Task Execution Graph (TDD-oriented)

### 5.1 Task 의존성 + TDD cycle + AC mapping 표

각 task 는 TDD cycle 1회 (RED → GREEN → REFACTOR) + 해당 AC 전체 GREEN 을 완료 조건으로 한다. Complexity tier: S (< 100 LOC), M (100~300 LOC), L (300~500 LOC).

| ID | 제목 | Blocker | Block | RED (failing test) | GREEN (최소 구현) | REFACTOR | Mapped AC | Tier | Platform |
|----|------|---------|-------|----------------------|--------------------|----------|-----------|------|----------|
| **T1** | PaneTree 자료구조 + unit test | 없음 | T2, T3 | `split_horizontal_from_leaf` (panics: PaneTree 미존재) | `enum PaneTree { Leaf, Split }` + `split_horizontal` 최소 구현 | in-order iterator + SplitError 분리 + `@MX:ANCHOR(pane-tree-invariant)` 주석화 | AC-P-1, AC-P-3, AC-P-20 (직접) + AC-P-2 (부분) | M | 양쪽 |
| **T2** | `PaneConstraints` associated const + API surface negative test | T1 | T3, T4 | `pane_constraints_has_no_mutable_api` (cargo public-api 출력이 `new`/`set_*` 포함) | `impl PaneConstraints { const MIN_COLS: u16 = 40; const MIN_ROWS: u16 = 10; }` | `@MX:ANCHOR(pane-constraints-immutable)` + doc `//! 이 상수는 런타임 변경 불가` | AC-P-21 (직접), AC-P-4 (간접 — 상수 참조), AC-P-14 (간접) | S | 양쪽 |
| **T3** | `PaneSplitter` / `ResizableDivider` trait + Mock | T1, T2 | T4, T5 | `abstract_traits_compile_without_impl` (trait 미정의로 cargo check FAIL) | trait 정의 + `#[cfg(test)] MockPaneSplitter` delegate-to-PaneTree | doc test 샘플 추가 + `@MX:ANCHOR(pane-splitter-contract)` | AC-P-17 (직접) | S | 양쪽 |
| **T4** | `PaneSplitter` 구체 구현 (S1 or S2 결과) | T3, **S1** (+ 조건부 S2) | T5, T6, T7 | `split_integration_spawns_new_terminal_surface` (Entity 생성 없음) | `GpuiNativeSplitter` OR `GpuiComponentSplitter` 최소 구현 + `PtyWorker::spawn` 연동 | external dep 호출 지점에 `@MX:WARN(external-dep-api-churn)` + `@MX:REASON(upstream-alpha)` | AC-P-1 (통합), AC-P-2 (drop), AC-P-5, AC-P-16 (Terminal Core 무변경 확인), AC-P-18 (bench) | L | 양쪽 |
| **T5** | `ResizableDivider` 구체 구현 + drag clamping | T3, T4, **S1** | T6 | `drag_clamps_ratio_within_min_size` (ratio 가 `< MIN_COLS/total` 이 되려는 drag 가 거부되지 않음) | `GpuiDivider::on_drag` 의 `raw_ratio.clamp(min, max)` 로직 | `@MX:NOTE(ratio-clamp-enforces-min-size)` + SplitDirection reuse 고려 | AC-P-6 (직접), AC-P-4 (경계 판정 연계) | M | 양쪽 |
| **T6** | Focus routing + MS-1 키 바인딩 | T4 | T7 | `next_pane_in_order` (focus cycle 미작동) + `single_focus_invariant` (다중 focused 동시 허용) | `FocusRouter` 최소 구현 + `platform_mod!` 매크로 (`#[cfg(target_os = ...)]`) | `@MX:ANCHOR(focus-routing)` + `@MX:NOTE(cmd-ctrl-platform-dispatch)` | AC-P-7 (직접), AC-P-22 (직접), AC-P-23 (tmux Ctrl+B passthrough), AC-P-9a / AC-P-9b (MS-1 부분) | L | **macOS + Linux 각각 검증** |
| **T7** | RootView 통합 + content_area 분기 재설계 | T4, T5, T6 | T8 | `empty_state_cta_shown_when_no_tabs` (lib.rs content_area 가 TabContainer 를 모름) | `terminal` 필드를 `tab_container: Option<Entity<TabContainer>>` 로 교체 + `content_area` 분기 재배선 | lib.rs 내 drive-by refactor **금지** (최소 diff) + `@MX:ANCHOR(root-view-content-binding)` | AC-P-16 (Terminal Core regression), AC-P-24 (부분 — 탭 바 미구현 상태) | M | 양쪽 |
| **T8** | `TabContainer` 자료구조 + 전환 로직 | T7 | T9, T10, T11 | `tab_switch_restores_focus` (last_focused_pane 복원 미작동) | `TabContainer` + `Tab` struct + `new_tab` / `switch_tab` / `close_tab` | `@MX:ANCHOR(tab-switch-invariant)` + `@MX:ANCHOR(tab-create-api)` | AC-P-8 (직접), AC-P-10, AC-P-11, AC-P-24, AC-P-25 (직접) | M | 양쪽 |
| **T9** | MS-2 키 바인딩 + tmux 중첩 통합 테스트 | T8 | T11 | `nested_tmux_does_not_receive_host_keystroke` (PTY master 에 Ctrl+T byte 기록됨) | 탭 키 dispatcher 확장 + `integration_tmux_nested.rs` | `@MX:ANCHOR(tab-key-dispatch)` + `@MX:NOTE(tmux-nested-os-priority-test)` | AC-P-9a (전체), AC-P-9b (전체, **S4 결정 전까지 경로 (a) 가정**), AC-P-26 (직접, v1.0.0 Nm-1) | M | **macOS + Linux 각각** + tmux 바이너리 필요 |
| **T10** | 탭 바 UI + design token `toolbar.tab.active.background` | T8 | T11 | `active_tab_has_bold_and_background_token` (font weight 와 bg color 모두 불일치) | `TabBar::render` + `.moai/design/v3/system.md` 에 `toolbar.tab.active.background` 토큰 추가 | `@MX:ANCHOR(active-tab-styling)` + design token diff 주석 | AC-P-27 (직접, v1.0.0 Nm-2), AC-P-24 (탭 바 렌더 경로) | M | 양쪽 |
| **T11** | 탭 전환 성능 bench | T8, T9 | T12 | `tab_switch_1_to_9_50_cycles` criterion bench 가 50ms threshold 위반 | 9 탭 × 2 pane fixture + criterion 측정 | fixture 재사용 가능성 평가 + `@MX:NOTE(tab-switch-performance-guard)` | AC-P-19 (직접) | S | Linux CI smoke only |
| **T12** | Persistence schema + atomic write + cwd fallback | T8 (MS-2 완료) | T13 | `save_tabs_atomic_without_vt_state` (`.tmp` 파일 잔재 + scrollback 키 존재) + `cwd_fallback_to_home_on_missing_dir` | `PanesFile` / `SerializedPaneTree` serde 타입 + `save_panes` (tempfile + fsync + rename) + `restore_panes` + `apply_cwd_fallback` | `@MX:WARN(race-condition-on-concurrent-write)` + `@MX:REASON(multiple-windows-may-write-same-file)` + `@MX:ANCHOR(persistence-restore-entry)` | AC-P-12, AC-P-13, AC-P-13a (v1.0.0 NM-1), AC-P-14, AC-P-15 (모두 직접) | L | **macOS + Linux 각각** |
| **T13** | Persistence 통합 (shutdown / startup hook) | T12 | T14 | `end_to_end_save_and_restore` (정상 shutdown 후 재시작이 pane tree 복원 못함) | GPUI `WindowCloseEvent` hook + app main 시작 시 `restore_panes` 호출 + RootView 초기화 | `@MX:WARN(shutdown-race-window)` + `@MX:REASON(crash-before-save-is-non-recoverable)` | AC-P-12 (end-to-end), AC-P-13 (end-to-end) | M | 양쪽 |
| **T14** | CI regression gate (`.github/workflows/ci-v3-pane.yml`) | T1 ~ T13 | — | CI 가 MS-1 / MS-2 / MS-3 matrix 없이 실패 | 5 job (`unit-tests`, `integration-tests`, `snapshot-tests`, `benches`, `terminal-core-regression`) × matrix (`macos-14`, `ubuntu-22.04`) | tmux 바이너리 install step + Zig setup step 재사용 (SPEC-V3-002 상속) | AC-P-16 (CI gate), 전체 29 AC 의 실행 경로 | S | CI only |

### 5.2 29 AC 완전 커버리지 검증

본 strategy 는 29 AC (AC-P-1 ~ AC-P-25 + AC-P-9a / 9b + AC-P-13a + AC-P-26 + AC-P-27) 전체를 위 Task 표에서 적어도 한 번 이상 매핑한다. 매핑 요약:

| AC | Task(s) | 실행 단계 |
|----|---------|-----------|
| AC-P-1 | T1 (unit), T4 (integration) | MS-1 |
| AC-P-2 | T1 (unit), T4 (FD count integration) | MS-1 |
| AC-P-3 | T1 (unit) | MS-1 |
| AC-P-4 | T2 (constants) + T4 (split rejection) | MS-1 |
| AC-P-5 | T4 + T5 (window resize headless) | MS-1 |
| AC-P-6 | T5 (divider clamp unit + manual) | MS-1 |
| AC-P-7 | T6 (focus routing unit) | MS-1 |
| AC-P-8 | T8 (tab switch focus restore unit) | MS-2 |
| AC-P-9a | T6 (MS-1 부분) + T9 (전체, macOS CI) | MS-1 + MS-2 |
| AC-P-9b | T6 (MS-1 부분) + T9 (전체, Linux CI) | MS-1 + MS-2, **S4 의존** |
| AC-P-10 | T8 (9 탭 생성 unit) | MS-2 |
| AC-P-11 | T8 + T9 (tab switch preserves integration) | MS-2 |
| AC-P-12 | T12 (atomic write + negative assertion) + T13 (end-to-end) | MS-3 |
| AC-P-13 | T12 + T13 (restore valid cwd) | MS-3 |
| AC-P-13a | T12 (cwd fallback, REQ-P-056) | MS-3 |
| AC-P-14 | T12 (schema mismatch) | MS-3 |
| AC-P-15 | T12 (parse failure + .corrupt rename) | MS-3 |
| AC-P-16 | T7 (초기 확인) + T14 (CI gate, 매 PR) | MS-1 + 전체 |
| AC-P-17 | T3 (abstract traits + doc test) | MS-1 |
| AC-P-18 | T4 (split perf benchmark) | MS-1 |
| AC-P-19 | T11 (tab switch benchmark) | MS-2 |
| AC-P-20 | T1 (ratio boundary unit) | MS-1 |
| AC-P-21 | T2 (negative API surface) | MS-1 |
| AC-P-22 | T6 (single focus invariant unit) | MS-1 |
| AC-P-23 | T6 (tmux prefix passthrough integration) | MS-1 |
| AC-P-24 | T7 (초기) + T8 (탭 바 활성) | MS-1 + MS-2 |
| AC-P-25 | T8 + T9 (10+ 탭 접근 정책) | MS-2 |
| AC-P-26 | T9 (tmux nested integration, v1.0.0 Nm-1) | MS-2 |
| AC-P-27 | T10 (탭 바 active styling, v1.0.0 Nm-2) | MS-2 |

**결론**: **29 AC 모두 적어도 한 Task 에 매핑되며, 누락 없음**.

### 5.3 병렬 실행 기회

downstream agent (manager-tdd) 는 아래 조합에 한해 병렬 (`Agent()` 2개 동시 호출) 가능:

- **Spike S1 / S3 / S4**: T1 착수 직후 또는 병렬 (3 개 모두 독립)
- **T2 와 T3**: T1 완료 후 병렬 착수 (서로 간섭 없음)
- **T9 / T10 / T11**: T8 완료 후 병렬 착수 가능하나 T11 의 fixture 가 T9 의 9 tabs 설정에 의존 → T9 선행 권장 (직렬 권고)
- **그 외 T4 → T5 → T6 → T7, T12 → T13 → T14**: 엄격 순차

---

## 6. Milestone Transition Gates

### 6.1 MS-1 → MS-2 진입 Gate

다음 AC 전체가 **macOS 14 runner + Ubuntu 22.04 runner 양쪽** CI 에서 GREEN 이어야 MS-2 (T8) 착수 가능:

- **Unit / Integration AC** (14 건): AC-P-1, AC-P-2, AC-P-3, AC-P-4, AC-P-5, AC-P-6, AC-P-7, AC-P-16, AC-P-17, AC-P-18, AC-P-20, AC-P-21, AC-P-22, AC-P-23
- **MS-1 에 해당하는 키 바인딩 부분 AC**: AC-P-9a (MS-1 5 건 중 3 건: split / close / focus prev), AC-P-9b (동일)
- **CI regression 확인**: `cargo test -p moai-studio-terminal` 74 tests exit 0 (AC-P-16 전제)

Gate 실패 시: MS-2 진입 차단, `re-planning gate` 트리거 (Phase 2.7).

### 6.2 MS-2 → MS-3 진입 Gate

MS-1 AC 전체 + 아래 9 AC 전체 GREEN:

- AC-P-8, AC-P-9a (전체), AC-P-9b (전체), AC-P-10, AC-P-11, AC-P-19, AC-P-24, AC-P-25, AC-P-26 (v1.0.0 Nm-1), AC-P-27 (v1.0.0 Nm-2)

특이 조건:
- AC-P-9b 는 **S4 사용자 결정이 확정된 이후**에 최종 통과 판정. 경로 (a) 현행 유지면 Gate 통과. 경로 (b) shift-escalation 면 annotation cycle 재개 후 키 바인딩 테이블 갱신 → Gate 재평가.
- AC-P-26 은 tmux 바이너리가 설치된 runner 에서만 실행 (T14 의 workflow yml 에 `apt install tmux` / `brew install tmux` step 필수).

### 6.3 Post-MS-3 완료 Gate (Sync 진입 전)

전체 29 AC GREEN + 아래 추가 조건:

- **SPEC-V3-002 의 74 tests + SPEC-V3-001 의 60 tests = 총 134 기존 tests 의 regression 0** (spec.md §6.4)
- **신규 test ≥ 20** (TDD cycle 의 RED → GREEN 으로 누적)
- **Coverage ≥ 85%** (`cargo tarpaulin` 또는 동등 도구, spec.md TRUST 5 Tested)
- **`cargo clippy --workspace --all-targets -- -D warnings` 0 warnings**
- **`cargo fmt --all -- --check` 통과**
- **`.github/workflows/ci-v3-pane.yml` 5 job matrix (macOS + Linux × unit/integration/snapshot/benches/terminal-core) 모두 GREEN, wall-clock ≤ 10 분**
- **MX tag 총 20+ 추가** (ANCHOR 9, WARN 3, NOTE 7, TODO 2 이상 — plan.md §8)

### 6.4 Commit 전략 (git-strategy.yaml: mode=manual, auto_branch=false, auto_commit=true)

- **Branch**: `feat/v3-scaffold` (현재 체크아웃된 branch) 에 직접 MS 별 commit 누적. 신규 branch 생성 금지 (spec.md §11.4).
- **Commit 단위**: 각 milestone 완료 시점에 per-milestone 분리 commit. 중간 task 완료에는 per-task commit 허용 (auto_commit: true).
- **`main` 직접 merge 금지** (spec.md §11.4).
- **PR**: `auto_pr: false` 이므로 downstream agent 가 자동 PR 생성 금지. 사용자 지시 후 수동 `gh pr create`.

---

## 7. Risk & Mitigation Snapshot

### 7.1 spec.md §12 R-1 ~ R-9 replay

| ID | 위험 | 현재 상태 | 완화 경로 |
|----|------|-----------|-----------|
| **R-1** | GPUI 0.2.2 divider drag API 미확인 | **Active mitigation needed** | **Spike 1** 로 해소. 실패 시 S2 escalate. |
| **R-2** | 다중 pane FD 압박 / 메모리 spike | Monitor during impl | T4 의 FD count 테스트 (AC-P-2) + pane 당 60MB 상한 (SPEC-V3-002 계승) |
| **R-3** | SPEC-V3-002 API 변경 유혹 | Active monitoring | T7 / T14 의 CI gate (AC-P-16), 매 PR 자동 rejection |
| **R-4** | GPUI FocusHandle vs active pane 괴리 | Active mitigation needed | T6 에서 Zed 의 `FocusHandle + last_focus_handle_by_item` 패턴 차용 + AC-P-22 single_focus_invariant 검증 |
| **R-5** | 탭 바 design token 부재 | Post-MS delivery + T10 | REQ-P-044 + AC-P-27 이 "bold + background" 최소 스펙을 SPEC 본문에 직접 규정. T10 에서 `.moai/design/v3/system.md` 에 `toolbar.tab.active.background` 토큰 값 추가 — **[USER-DECISION-REQUIRED: design-token-color-value]** |
| **R-6** | Persistence schema 역호환 | Deferred to T12 | `panes-{ws-id}.json` + `"moai-studio/panes-v1"` schema 분리 (RG-P-6). AC-P-14 / AC-P-15 검증. |
| **R-7** | gpui-component 의존 유지비 | **Deferred to S2** | S1 PASS 면 회피. S1 FAIL 이고 S2 PASS 면 commit SHA pin. S2 도 FAIL 이면 user escalation. |
| **R-8** | Linux Super 키 기대 충돌 | Deferred to Phase 5 SPEC | 본 SPEC 은 Ctrl 고정 (design 원천). Shortcut Customization SPEC 에서 해소. |
| **R-9** | Linux Ctrl+D/W/\\ shell 관례 충돌 | **Active, USER DECISION NEEDED** | **Spike 4** 측정 후 경로 (a)/(b) 중 사용자 확정 — `[USER-DECISION-REQUIRED: spike-4-linux-shell-path]` |

### 7.2 본 strategy 단계에서 신규 식별된 위험

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| **R-P1** | GPUI 0.2.2 headless 테스트 infeasibility | AC-P-5 (window resize), AC-P-11 (tab switch identity), AC-P-27 옵션 B (element 속성 추출) 가 GPUI headless 지원 미확정 | T6/T8/T10 의 RED phase 에서 GPUI `TestAppContext` 의 실제 지원 수준을 먼저 spike 측정. 미지원 시 snapshot test 옵션 (AC-P-27 옵션 A) 또는 PaneTestHarness mock 으로 대체. 사용자 통지 사항 → Section 8 에 추가 `[USER-DECISION-REQUIRED: gpui-headless-test-strategy]` 등록 여부 검토 대상. 본 strategy 는 **경로 A (snapshot + mock) 를 기본값** 으로 채택 권장 — spike 없이도 진행 가능. |
| **R-P2** | tmux 바이너리 CI 의존 | AC-P-26 실행 불가 | T14 의 `ci-v3-pane.yml` 에 `apt install tmux` / `brew install tmux` step 추가 (R-P2 는 완화 방법이 명확하므로 SPEC-level 이슈 아님) |
| **R-P3** | plan.md 의 `lib.rs:290-299` vs 실제 `:286-300` 경미한 line reference drift | downstream agent 가 line 참조를 곧이곧대로 믿으면 miss | T7 의 RED phase 에서 **반드시 실제 파일을 Read 로 재확인** (Tool Selection Priority 준수). 본 strategy Section 2.1 에서 이미 명기. |
| **R-P4** | `gpui-component` (S2 채택 시) 와 GPUI 0.2.2 의 버전 호환성 | 빌드 깨짐 | S2 spike 가 commit SHA pin 가능성을 명시적 성공 criterion 으로 포함. S2 단계에서 호환 검증 실패 시 S2 FAIL 판정. |
| **R-P5** | Drift guard 에서 3 consecutive zero-progress iteration 시 re-planning gate 트리거 | 구현 중단 | drift guard + re-planning gate (spec-workflow.md) 가 자동 감지. manager-tdd 가 stagnation report 를 MoAI 에 반환 → 사용자 AskUserQuestion 으로 이어짐. 사전 완화 무 (detection-only). |

---

## 8. Open Questions for User (USER-DECISION-REQUIRED 총람)

본 strategy 에서 마킹한 사용자 확정 필요 사항. 오케스트레이터 (MoAI) 가 AskUserQuestion 으로 순차 제시 예정.

| Marker | 결정 내용 | 결정 시점 | 제시 옵션 | 기본값 (사용자 결정 지연 시) |
|--------|-----------|-----------|-----------|----------------------------|
| **`[USER-DECISION-REQUIRED: spike-4-linux-shell-path]`** | Spike 4 의 Linux shell 관례 경로 선택 | S4 측정 보고서 완성 직후, T6 / T9 Linux 구현 직전 | (a) 현행 유지 + Customization SPEC 으로 연기 vs (b) Ctrl+Shift escalation + annotation cycle 재개 | (a) 현행 유지 |
| **`[USER-DECISION-REQUIRED: gpui-component-adoption]`** | S1 FAIL + S2 결과에 따른 gpui-component 채택 여부 | S2 완료 직후, T4 구현 직전 | (a) S1 PASS → 자체 구현 (default) / (b) S2 PASS → gpui-component 도입 + SHA pin / (c) 양쪽 FAIL → user escalation (plan 재조정) | S1 결과에 따라 자동 — 단독 (a) 는 MoAI 자동 확정, (b)/(c) 는 사용자 확인 필요 |
| **`[USER-DECISION-REQUIRED: design-token-color-value]`** | `.moai/design/v3/system.md` Toolbar 섹션에 추가할 `toolbar.tab.active.background` 의 정확한 RGB/OKLCH 값 | T10 RED phase 직전 | (a) 기존 `BG_SURFACE_3` 계열에서 한 단계 밝은 값 / (b) `BG_SURFACE_2` (중간) / (c) 사용자 커스텀 지정 | (a) 기존 토큰 계열 extrapolation |

각 marker 는 orchestrator 가 AskUserQuestion 에 제시할 때 (1) 옵션 간 trade-off, (2) 본 strategy 의 추천 근거, (3) 결정 지연 시 기본값을 함께 전달한다. 사용자 결정이 strategy 확정 경로와 다를 경우 manager-tdd 는 재위임 받아 해당 Task 의 RED phase 부터 재착수.

---

## 9. Effort Estimate (Priority 기반, 시간 추정 금지)

| Milestone | Tier (복잡도) | Task 수 | 우선순위 분포 | 핵심 blocker 위험 |
|-----------|---------------|---------|----------------|-------------------|
| **MS-1 (Pane core)** | **L** | T1 ~ T7 (7 tasks) | High 6 / Medium 1 | Spike 1 결과 (R-1) + GPUI headless 테스트 feasibility (R-P1) |
| **MS-2 (Tabs)** | **M** | T8 ~ T11 (4 tasks) | High 3 / Medium 1 | Spike 4 사용자 결정 (R-9) + tmux CI 바이너리 (R-P2) |
| **MS-3 (Persistence)** | **M** | T12 ~ T14 (3 tasks) | High 3 | Persistence race 조건 (R-P5 류) + CI workflow 신설 |

**전체 SPEC-V3-003 규모**: **XL** (14 tasks × 3 milestones × 양 플랫폼 CI, 29 AC, 4 Spikes, 2 사용자 결정 포인트). Harness level thorough (progress.md 기록) 에 부합.

**우선순위 순차**:
1. **High (9건)**: T1, T2, T3, T4, T5, T6, T7 (MS-1) + T8, T9, T10, T12, T13, T14
2. **Medium (4건)**: T11 (탭 bench, smoke only), Spike 3/4 (사용자 결정 대기)
3. **Low (1건)**: 해당 없음 — plan.md 의 Low 1건은 Spike 분류와 관련 없이 T11 bench 로 해석되어 Medium 으로 격상 가능

---

## 10. Handover Summary (manager-tdd / downstream 에게)

downstream agent (manager-tdd) 는 본 strategy.md 를 Run Phase 2 Progress Input 으로 수령하며, 아래 규칙을 준수한다:

1. **SPEC-V3-002 공개 API 는 as-is 호출** — `crates/moai-studio-terminal/**` 수정 절대 금지 (AC-P-16)
2. **TDD cycle 엄격 준수** — 각 Task 마다 RED → GREEN → REFACTOR 1회 완주. RED 없이 GREEN 코드 작성 금지 (quality.yaml development_mode=tdd).
3. **Spike 우선** — T4/T5 착수 전에 S1 완료. T6/T9 의 Linux 바인딩 확정 전에 S4 사용자 결정. T1 의 PaneId type 정의는 S3 결과 반영 전에 placeholder 허용.
4. **AC-ID 를 commit body 에 명시** — `feat(panes): T1 PaneTree (AC-P-1, AC-P-3, AC-P-20)` 등 형식.
5. **drive-by refactor 금지** — `crates/moai-studio-ui/src/lib.rs` 는 전체 719 LOC 이지만 T7 은 필드 교체 + 분기 재배선 최소 범위.
6. **사용자 결정 필요 3 건은 AskUserQuestion 대상** — manager-tdd 는 결정 없이 경로 fork 금지, orchestrator 에 blocker report 반환.
7. **Drift guard / re-planning gate 자동 감지** — 3 consecutive zero-progress iteration 시 stagnation 보고.
8. **@MX tag 계획 plan.md §8 준수** — ANCHOR 9+ / WARN 3+ / NOTE 7+ / TODO 2+ 목표. 한국어 주석 (language.yaml code_comments=ko).

---

Version: 1.0.0 · 2026-04-24 · Run Phase 1 Analysis Complete
