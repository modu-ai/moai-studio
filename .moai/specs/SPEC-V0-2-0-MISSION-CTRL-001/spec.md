---
id: SPEC-V0-2-0-MISSION-CTRL-001
version: 1.0.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: High
issue_number: 0
depends_on: [SPEC-V3-010]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [v0.2.0, agent, mission-control, ui, hook-stream, audit-top-8]
revision: v1.0.0 (initial draft, full SPEC — Lightweight 부적격 사유: cross-component, 신규 hook subscribe API 표면, ≥3 milestones)
---

# SPEC-V0-2-0-MISSION-CTRL-001: Mission Control — parallel agents 4-cell grid + per-run summary registry

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 8 (audit Top 8 #2 ⭐⭐⭐⭐⭐). V3-010 AgentDashboardView (단일 run 5-pane 상세) 의 보완 surface — N개 agent run 의 4-cell grid summary. AgentRunRegistry (per-run summary cache) + MissionControlView (read-only render) + SseIngest hook wire (MS-3 carry). |

---

## 1. 개요

### 1.1 목적

`moai-adk team mode` 또는 다중 SDK session 실행 시 사용자가 N개의 agent run 을 **동시에 모니터링** 할 수 있는 "Mission Control" surface 도입. 단일 run 상세 보기 (V3-010 `AgentDashboardView`) 와 별개로, 4-cell grid 형태의 per-run **summary card** (status pill / 최근 이벤트 / cost roll-up) 를 한 화면에 노출한다.

audit feature-audit.md §3 Tier E v0.2.0 critical gap line 208 + §4 Top 8 Demo-Visible Wins #2 (⭐⭐⭐⭐⭐) 의 정식 해소.

### 1.2 근거 문서

- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier E + §4 Top 8 #2 (audit baseline).
- `.moai/specs/SPEC-V0-2-0-MISSION-CTRL-001/research.md` — 코드베이스 분석 + ADR 4건 + risk register.
- `.moai/specs/SPEC-V3-010/spec.md` — 단일 agent dashboard predecessor.
- `crates/moai-studio-agent/src/events.rs` — AgentRunId / AgentRunStatus / AgentEvent 도메인 (V3-010).
- `crates/moai-studio-agent/src/cost.rs` — CostSnapshot.
- `crates/moai-studio-agent/src/sse_ingest.rs` — SSE 파서 scaffold (E-4 carry).

---

## 2. 배경 및 동기

상세 분석은 research.md §1 참조. 요지:

- V3-010 의 `AgentDashboardView` 는 **단일** agent run 의 5-pane 상세 뷰. team mode / 다중 SDK session 시 사용자는 4+ agent 를 **한눈에** 보고 싶어 하지만 현재 surface 가 없다.
- audit §3 Tier E line 208: "E-5 Mission Control (parallel agents grid) — 멀티 에이전트 동시 모니터링. 추정 ~1.2K LOC, hook stream 의존, 중간-높은 위험."
- 본 SPEC 의 demo-visibility 가치: "다중 에이전트 동시 실행 가시화. moai-adk team mode 의 시각적 anchor" (audit §4 #2).

---

## 3. 목표 및 비목표

### 3.1 목표 (Goals)

- G1. 신규 도메인 `AgentRunRegistry` — per-run summary (`AgentCard`) 캐시. push(AgentEvent) → 해당 run 의 카드 갱신.
- G2. `AgentCard` — id / label / status / last_event_summary / last_event_at / cost_snapshot / event_count.
- G3. AgentRunStatus 의 transition 규칙 — hook event_name 별 (SessionStart → Running, Stop → Completed, ...).
- G4. 신규 GPUI Entity `MissionControlView` — Registry snapshot 을 4-cell grid 로 read-only 렌더 (default 4-cell, max 9, top-N 선택).
- G5. `MissionControlView` 가 RootView 의 신규 옵셔널 필드로 보유될 수 있는 인터페이스 (mount/dismiss API). 실 mount 결정은 MS-2 마지막 단계.
- G6. SseIngest 가 AgentRunRegistry 와 wire 되어 실 hook server 이벤트 → 카드 갱신 (MS-3 carry — 별 PR).
- G7. V3-010 의 기존 모듈 (events / cost / ring_buffer / sse_ingest / view) 의 공개 API 무변경.

### 3.2 비목표 (Non-Goals)

- N1. **Per-run kill / pause / resume dispatch** — MissionControlView 는 read-only.
- N2. **Hook server (moai-hook-http) HTTP client 실 연결** — MS-3 carry 또는 별 SPEC (HOOK-WIRE-001 후보).
- N3. **Agent run start trigger UI** — 사용자가 새 agent 를 시작하는 진입점은 별 SPEC.
- N4. **Per-cell click → AgentDashboardView 전환 wire** — MS-2 후속 PR.
- N5. **Mission Control 진입 키 바인딩 / Command Palette entry** — MS-2 후속 PR.
- N6. **N-cell scroll (10+ runs)** — default 4-cell, max 9 fixed.
- N7. **Run 그룹화 (workspace / team 별)** — 별 SPEC.

---

## 4. Requirements (EARS)

### RG-MC-1: AgentCard 도메인 (MS-1)

- **REQ-MC-001**: `AgentCard` 는 다음 필드를 보유한다 — `run_id: AgentRunId`, `label: String`, `status: AgentRunStatus`, `last_event_summary: String`, `last_event_at: Option<u128>` (Unix ns), `cost: Option<CostSnapshot>`, `event_count: u64`.
- **REQ-MC-002**: `AgentCard::new(run_id, label)` 는 status=Running, event_count=0, last_event_at=None, last_event_summary 빈 문자열로 초기화한다.
- **REQ-MC-003**: `AgentCard` 는 `Clone + Debug` 트레이트를 구현한다. (PartialEq 는 cost field 의 `CostSnapshot` 이 V3-010 에서 PartialEq 미구현이므로 본 SPEC scope 에서는 제외한다 — REQ-MC-040 frozen zone 보호.)

### RG-MC-2: AgentRunRegistry 도메인 (MS-1)

- **REQ-MC-010**: `AgentRunRegistry` 는 `HashMap<AgentRunId, AgentCard>` 를 내부 자료구조로 보유한다. Default 는 빈 map.
- **REQ-MC-011**: `register(run_id, label)` 는 새 카드를 등록한다. 동일 id 재등록 시 기존 카드의 label/status 만 갱신, event_count 등 history 보존.
- **REQ-MC-012**: `push_event(run_id, event)` 는 해당 카드의 last_event_summary / last_event_at / event_count 를 갱신한다. 카드가 없으면 자동 register (label = run_id 의 String 표현).
- **REQ-MC-013**: `push_event` 는 hook event_name 에 따라 status 를 자동 transition 한다:
  - `SessionStart` → status=Running
  - `Stop` → status=Completed
  - `Notification` (severity=error) → status=Failed
  - 다른 event_name → 기존 status 유지
- **REQ-MC-014**: `set_status(run_id, status)` 는 명시적 status 갱신 API (SDK self-report 또는 supervisor 측 명시 신호용).
- **REQ-MC-015**: `set_cost(run_id, cost)` 는 카드의 cost snapshot 을 갱신한다.
- **REQ-MC-016**: `cards()` 는 모든 카드의 immutable iterator 를 반환한다.
- **REQ-MC-017**: `top_n_active(n)` 는 status=Running/Paused 인 카드를 last_event_at 내림차순으로 정렬해 상위 n 개를 반환한다 (4-cell grid 의 source).
- **REQ-MC-018**: `clear_terminal()` 은 status.is_terminal() 인 카드를 제거한다 (사용자가 명시적으로 청소할 때 호출).

### RG-MC-3: MissionControlView GPUI Entity (MS-2)

- **REQ-MC-020**: `MissionControlView` 는 `AgentRunRegistry` 의 read-only snapshot 을 보유한다 (`Arc<RwLock<...>>` 또는 `Entity<...>` 를 통한 공유, MS-2 시점 결정).
- **REQ-MC-021**: `Render for MissionControlView` 는 4-cell grid (2x2) 를 렌더한다. 셀 수가 active run 보다 많으면 빈 셀 placeholder ("no active run").
- **REQ-MC-022**: 각 셀은 (a) 상단 status pill (color per status — research §3 ADR-MC-4), (b) label / run_id 짧은 라벨, (c) 최근 event summary 1-line preview, (d) cost roll-up (input/output tokens + cost_usd 가 있으면) 4 element 를 표시.
- **REQ-MC-023**: 셀 클릭 listener 는 placeholder (MS-2 단계는 logic-only) — 실 dispatch 는 별 PR carry.
- **REQ-MC-024**: `MissionControlView` 는 RootView 의 신규 옵셔널 필드 (`mission_control: Option<Entity<MissionControlView>>`) 로 mount 가능하다. mount 진입 트리거는 MS-2 마지막 PR 또는 별 PR 결정.

### RG-MC-4: Hook wire (MS-3 — 본 SPEC scope 에 포함되나 별 PR 로 carry)

- **REQ-MC-030**: SseIngest 는 새 helper `pump_into_registry(registry: &mut AgentRunRegistry, chunk: &str)` 를 노출한다 — parse_sse_chunk 의 결과 AgentEvent 들을 registry.push_event 로 전달.
- **REQ-MC-031**: HTTP client (reqwest 등) 가 SSE endpoint 에 연결되어 chunk 를 실시간 수신 — 본 REQ 는 별 PR 또는 별 SPEC 으로 carry.

### RG-MC-5: V3-010 무변경 (전체)

- **REQ-MC-040**: `crates/moai-studio-agent/src/{events.rs, ring_buffer.rs, cost.rs, filter.rs, instructions.rs, control.rs, view.rs, stream_ingest.rs, sse_ingest.rs}` 의 공개 API 무변경. (sse_ingest 는 helper 함수 추가만 허용, 기존 export 무변경.)
- **REQ-MC-041**: `crates/moai-studio-ui/src/agent/{dashboard_view.rs, control_bar.rs, cost_panel_view.rs, detail_view.rs, instructions_graph_view.rs, timeline_view.rs}` 무변경.
- **REQ-MC-042**: `crates/moai-studio-terminal/**` 무변경.

---

## 5. Acceptance Criteria

| AC ID | Requirement | Milestone | Given | When | Then | 검증 수단 |
|-------|-------------|-----------|-------|------|------|-----------|
| AC-MC-1 | REQ-MC-001/002/003 | MS-1 | `AgentCard::new(id, "Demo")` | 직후 필드 검사 | status=Running, event_count=0, last_event_at=None, last_event_summary="" | unit test |
| AC-MC-2 | REQ-MC-010/011 | MS-1 | empty Registry | `register(id, "A")` 두 번 호출 (label "A" → "B") | cards 1개, label="B", event_count 보존 | unit test |
| AC-MC-3 | REQ-MC-012 | MS-1 | empty Registry | `push_event(id, hook_event)` (SessionStart) | 자동 register + 카드 1개, event_count=1, last_event_summary 비어있지 않음, last_event_at != None | unit test |
| AC-MC-4 | REQ-MC-013 | MS-1 | Registry 에 status=Running 카드 | `push_event` Stop 이벤트 | status=Completed | unit test |
| AC-MC-5 | REQ-MC-013 | MS-1 | Registry 에 status=Running 카드 | `push_event` Notification (severity=error) 이벤트 | status=Failed | unit test |
| AC-MC-6 | REQ-MC-014 | MS-1 | Registry 에 카드 | `set_status(id, Killed)` | status=Killed (transition 규칙 무관 직접 갱신) | unit test |
| AC-MC-7 | REQ-MC-015 | MS-1 | Registry 에 카드 | `set_cost(id, snapshot)` | cost == Some(snapshot) | unit test |
| AC-MC-8 | REQ-MC-017 | MS-1 | Registry 에 5 카드 (3 Running + 1 Completed + 1 Failed) | `top_n_active(4)` | 3 카드 반환 (Running 만), last_event_at 내림차순 | unit test |
| AC-MC-9 | REQ-MC-018 | MS-1 | Registry 에 4 카드 (2 Running + 2 Completed) | `clear_terminal()` | cards 2개만 남음 | unit test |
| AC-MC-10 | REQ-MC-016 | MS-1 | Registry 에 3 카드 push 후 | `cards().count()` | 3 반환, 각 id 로 lookup 가능 | unit test |
| AC-MC-11 | REQ-MC-020/021 | MS-2 | MissionControlView 가 빈 Registry snapshot 으로 생성 | Render | 4 셀 모두 placeholder ("no active run") | unit test (logic-level) + 수동 smoke |
| AC-MC-12 | REQ-MC-021/022 | MS-2 | Registry 에 2 active card | Render | 2 셀에 카드 정보, 2 셀 placeholder | unit test (logic-level) |
| AC-MC-13 | REQ-MC-024 | MS-2 | RootView 에 mission_control field 추가 | RootView::new 직후 | mission_control == None (lazy) | unit test |
| AC-MC-14 | REQ-MC-030 | MS-3 | SSE chunk 1개 (PostToolUse) | `pump_into_registry(&mut reg, chunk)` | reg.cards() 1개, event_count=1 | unit test |
| AC-MC-15 | REQ-MC-040/041/042 | 전체 | 본 SPEC 모든 milestone | `cargo test -p moai-studio-agent` + `-p moai-studio-ui` + `-p moai-studio-terminal` | 기존 V3-010 + V3-009 + 모든 회귀 0 | CI gate |

---

## 6. Milestones

### MS-1: AgentRunRegistry 도메인 (본 PR scope)

- 범위:
  - `crates/moai-studio-agent/src/mission_control.rs` 신규 — `AgentCard` + `AgentRunRegistry` + transition 규칙 + 단위 테스트.
  - `crates/moai-studio-agent/src/lib.rs` re-export 추가.
- 포함 REQ: REQ-MC-001 ~ REQ-MC-018 (RG-MC-1 + RG-MC-2 전체).
- AC: AC-MC-1 ~ AC-MC-10 (10건).
- Test count target: ≥12 신규.
- 시연: logic-only — 단위 테스트 GREEN. UI 변화 없음.

### MS-2: MissionControlView GPUI Entity (별 PR carry)

- 범위:
  - `crates/moai-studio-ui/src/agent/mission_control_view.rs` 신규.
  - `crates/moai-studio-ui/src/agent/mod.rs` re-export.
  - `crates/moai-studio-ui/src/lib.rs` RootView 에 `mission_control: Option<Entity<MissionControlView>>` 필드 추가 (R3 새 필드만).
  - mount API + Render 분기.
- 포함 REQ: REQ-MC-020 ~ REQ-MC-024.
- AC: AC-MC-11 ~ AC-MC-13.
- Test count target: ≥6 신규.

### MS-3: Hook stream wire (별 PR carry, 또는 별 SPEC HOOK-WIRE-001 으로 분리)

- 범위:
  - SseIngest 에 `pump_into_registry` helper 추가.
  - HTTP client (reqwest stream) 로 endpoint subscribe — 본 REQ 는 USER-DECISION 이후 결정.
- 포함 REQ: REQ-MC-030 (+선택 REQ-MC-031).
- AC: AC-MC-14.
- USER-DECISION 게이트: HTTP client 채택 시점에 reqwest vs ureq 선택, polling 주기 결정.

---

## 7. File Layout

### 7.1 신규 (MS-1)

- `crates/moai-studio-agent/src/mission_control.rs` — `AgentCard` + `AgentRunRegistry` + transition rules + 단위 테스트.

### 7.2 수정 (MS-1)

- `crates/moai-studio-agent/src/lib.rs` — `pub mod mission_control;` + re-export.

### 7.3 신규 (MS-2 carry)

- `crates/moai-studio-ui/src/agent/mission_control_view.rs`.

### 7.4 수정 (MS-2 carry)

- `crates/moai-studio-ui/src/agent/mod.rs` — `pub mod mission_control_view;` + re-export.
- `crates/moai-studio-ui/src/lib.rs` — RootView 의 `mission_control` 옵셔널 field + Render 분기.

### 7.5 변경 금지 (FROZEN — REQ-MC-040 ~ 042)

- `crates/moai-studio-terminal/**` 전체.
- `crates/moai-studio-workspace/**` 전체.
- `crates/moai-studio-agent/src/{events.rs, ring_buffer.rs, cost.rs, filter.rs, instructions.rs, control.rs, view.rs, stream_ingest.rs}` 의 공개 API.
- `crates/moai-studio-ui/src/agent/{dashboard_view.rs, control_bar.rs, cost_panel_view.rs, detail_view.rs, instructions_graph_view.rs, timeline_view.rs}` 전체.

---

## 8. 의존성 및 제약

### 8.1 외부 의존성

| Crate | 버전 | 변경 |
|-------|------|------|
| `serde` / `serde_json` | workspace | 변경 없음 |
| `tracing` | workspace | 변경 없음 |
| `gpui` | 0.2.2 (V3-001 carry) | 변경 없음 (MS-2 때만 사용) |
| `reqwest` 또는 `ureq` | TBD | MS-3 USER-DECISION |

### 8.2 USER-DECISION 게이트 (MS-3 진입 시)

- **[USER-DECISION-MC-A: hook-http-client]** — MS-3 진입 직전 발동.
  - Option (a): `reqwest` (async, tokio 의존) — moai-adk-go 와 정렬.
  - Option (b): `ureq` (sync, 의존 가벼움) — GPUI 외부 thread 에서만 사용.
  - Default: option (a). 비채택 시 progress.md 명시.

### 8.3 Git / Branch 제약

- MS-1: `feature/SPEC-V0-2-0-MISSION-CTRL-001-ms1` (본 PR).
- MS-2: `feature/SPEC-V0-2-0-MISSION-CTRL-001-ms2` (carry).
- MS-3: `feature/SPEC-V0-2-0-MISSION-CTRL-001-ms3` 또는 `feature/SPEC-V0-2-0-HOOK-WIRE-001-ms1` (별 SPEC 분리 시).
- main 직접 커밋 금지 (CLAUDE.local.md §1).
- 각 MS squash merge.

---

## 9. 위험 및 완화 (research §2 mirror)

| ID | 위험 | 완화 |
|----|------|------|
| R-MC-1 | Hook stream 의존도 — server 미가동 시 빈 화면 | MS-1+MS-2 logic-only / read-only. mock injection helper. 실 wire 는 MS-3. |
| R-MC-2 | 4-cell vs N-cell 결정 | default 4, max 9 fixed. scroll 은 carry. |
| R-MC-3 | Registry 와 RingBuffer 중복 | Registry 는 summary 만 (last_event 캐시). RingBuffer 는 V3-010 그대로. |
| R-MC-4 | AgentDashboardView 와의 컴포넌트 중복 | MissionControlView 는 read-only summary. 클릭 → DashboardView 진입은 carry. |
| R-MC-5 | GPUI render 복잡도 | MS-2 는 4-cell + per-card 4 element 만. instruction graph / cost panel 재사용 X. |

---

## 10. 참조 문서

- `.moai/specs/SPEC-V0-2-0-MISSION-CTRL-001/research.md` — 본 SPEC 의 코드베이스 분석 + ADR.
- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier E + §4 Top 8 #2.
- `.moai/specs/SPEC-V3-010/spec.md` — V3-010 (Agent Dashboard, predecessor).
- `crates/moai-studio-agent/src/events.rs` — AgentRunId / AgentRunStatus / AgentEvent / EventKind / HookEvent / TokenUsage.
- `crates/moai-studio-agent/src/cost.rs` — CostSnapshot, CostTracker.
- `crates/moai-studio-agent/src/sse_ingest.rs` — parse_sse_chunk + SseIngestor scaffold.

---

## 11. Exclusions

본 SPEC 이 명시적으로 다루지 않는 항목 (별 SPEC 으로 분리):

- E1. **moai-hook-http server 측 변경** — V3-010 이후 변경 없음. 본 SPEC 은 SSE chunk 만 소비.
- E2. **Agent run start trigger UI** — 사용자가 새 agent 시작하는 진입점.
- E3. **Per-cell click → AgentDashboardView 전환 wire** — MS-2 후속.
- E4. **Mission Control 진입 키 바인딩 / Command Palette entry** — MS-2 후속.
- E5. **N-cell scroll / virtualization** — 10+ run 시.
- E6. **Run 그룹화 (workspace / team 별)** — 별 SPEC.
- E7. **Status pill custom theming** — design::tokens 의 기존 토큰만 사용.
- E8. **Cost aggregation across runs** — 본 SPEC 은 per-run cost 만.

---

## 12. 용어 정의

- **Mission Control**: 다중 agent run 의 4-cell grid summary surface. AgentDashboardView (단일 run 5-pane 상세) 의 보완.
- **AgentCard**: per-run summary 캐시 (status / last event / cost roll-up). RingBuffer 와 별개의 in-memory state.
- **AgentRunRegistry**: AgentCard 의 HashMap 컨테이너. push_event API 로 갱신.
- **active card**: status 가 Running / Paused 인 카드 (terminal 상태 제외).
- **top-N**: active card 를 last_event_at 내림차순 정렬한 상위 N개. 4-cell grid 의 source.

---

Version: 1.0.0
Source: SPEC-V0-2-0-MISSION-CTRL-001
Last Updated: 2026-05-04
