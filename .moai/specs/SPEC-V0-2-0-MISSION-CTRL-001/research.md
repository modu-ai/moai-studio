# SPEC-V0-2-0-MISSION-CTRL-001 — Research

**Audit reference**: RELEASE-V0.2.0/feature-audit.md §3 Tier E v0.2.0 critical gap (E-5) + §4 Top 8 Demo-Visible Wins #2.
**Predecessor**: SPEC-V3-010 (Agent Dashboard, MS-1/2/3 in-progress).
**v3 design spec**: §3 design-v3-spec.md 에는 Mission Control 전용 frame 없음. 본 SPEC 이 자체 정의한다.

---

## 1. Background

### 1.1 audit doc 의 정의 (feature-audit.md line 273~278)

> **E-5 Mission Control (parallel agents grid)** ⭐⭐⭐⭐⭐
> - **Why**: 다중 에이전트 동시 실행 가시화. moai-adk team mode 의 시각적 anchor. 4-cell grid + per-agent status pill.
> - **Demo visibility**: HIGH (별 surface 또는 Right Panel 에 mount).
> - **Scope**: hook event stream 의존 + grid layout + per-agent state machine. 추정 ~1.2K LOC.
> - **Risk**: 중간-높음 (hook stream 안정성 의존).

### 1.2 현재 구현 상태 (V3-010 carry)

V3-010 `AgentDashboardView` (115 LOC, dashboard_view.rs) 는 **단일 agent run** 의 5-pane 상세 뷰:
```
┌─────────────────── AgentControlBar (toolbar) ───────────────────┐
├─────────────────────────────┬──────────────────────────────────┤
│ EventTimelineView           │ CostPanelView                    │
├─────────────────────────────┼──────────────────────────────────┤
│ InstructionsGraphView       │ EventDetailView                  │
└─────────────────────────────┴──────────────────────────────────┘
```

기존 도메인 (V3-010 in `moai-studio-agent` crate):
- `AgentRunId` (events.rs) — 단일 소스 of truth (REQ-AD-001)
- `AgentRunStatus` (events.rs) — Running / Paused / Completed / Failed / Killed
- `AgentEvent` + `EventKind::{StreamJson, Hook, Unknown}` — 이벤트 도메인
- `RingBuffer<AgentEvent>` — capacity 1000 default
- `StreamIngest` (stream_ingest.rs) + `SseIngest` (sse_ingest.rs scaffold) — 이벤트 수신
- `CostTracker` — token 비용 집계
- `EventFilter` — kind/run 필터
- `ControlEnvelope` — pause/resume/kill stdin envelope

### 1.3 Mission Control vs AgentDashboardView 의 관계

| 측면 | AgentDashboardView (V3-010) | MissionControlView (본 SPEC) |
|------|----------------------------|------------------------------|
| 표시 단위 | 단일 agent run 1개 | N개 agent run (default 4-cell, max 9) |
| 데이터 깊이 | 모든 이벤트 + cost + instructions + detail | per-run summary card (status / 최근 이벤트 / cost roll-up) |
| 전환 | 사용자가 한 run 을 선택 후 진입 | 항상 모든 active run overview |
| 진입점 | tab 내부 mount (V3-010) | Right Panel 또는 별 surface (audit §3 line 275) |
| Kill 동작 | per-event detailed control | per-run kill button (placeholder) |

본 SPEC 은 V3-010 도메인 (AgentRunId / AgentEvent / SseIngest) 를 **재사용** 하며, 새 **`AgentRunRegistry`** 가 이 위에 per-run summary state 를 누적한다.

### 1.4 Hook stream subscribe 경로 (E-4 carry)

audit §3 Tier E line 207: "**E-4 Hook event stream 27 events 전수 wire** — moai-hook-http server 측 OK, GPUI 측 27 events 모두 wire 필요."

본 SPEC 은 E-4 의 wire-up 을 **부분적으로 driving** 한다 — Mission Control 이 SSE stream 의 첫 실 consumer 이기 때문. 단, 본 SPEC scope 는:
- MS-1: AgentRunRegistry 가 push() API 로 AgentEvent 를 받아 per-run summary 갱신 (logic-only)
- MS-2: MissionControlView 가 AgentRunRegistry snapshot 을 4-cell grid 로 렌더 (read-only)
- **MS-3 (carry)**: 실 SseIngest HTTP client 를 Registry 와 wire (SPEC-V0-2-0-HOOK-WIRE-001 또는 본 SPEC MS-3 으로 별 PR)

따라서 본 SPEC MS-1+MS-2 는 hook 서버 가용성과 **무관하게 단위 테스트 100% GREEN** 보장.

---

## 2. Risks

| ID | 위험 | 완화 |
|----|------|------|
| R-MC-1 | Hook stream 의존도 — server 미가동 시 화면 비어 보임 | MS-1+MS-2 는 logic-only / read-only. mock data injection helper 제공 (run_app 또는 통합 테스트). 실 wire 는 MS-3 carry. |
| R-MC-2 | 4-cell vs N-cell 결정 — 동시 9+ agent 가능성 | default 4-cell, max 9 까지 fixed grid 지원. 그 이상은 scroll. 본 SPEC 단계는 4-cell 만. |
| R-MC-3 | AgentRunRegistry 와 RingBuffer 중복 | Registry 는 per-run **summary** (last event timestamp / status / cost roll-up) 만 보유. event 본문은 RingBuffer 에 그대로 (V3-010 carry). |
| R-MC-4 | AgentDashboardView 와의 중복 컴포넌트 | MissionControlView 는 read-only summary 만. 사용자 클릭 → AgentDashboardView 진입 (별 PR carry). |
| R-MC-5 | GPUI render 복잡도 | MS-2 는 4-cell grid + per-card title/status/last-event-line 만. instruction graph / cost panel 재사용 X. |

---

## 3. Architecture Decision Records

### ADR-MC-1: Registry vs RingBuffer 분리

**Decision**: AgentRunRegistry 는 per-run summary (Map<AgentRunId, AgentCard>) 만 보유. RingBuffer (V3-010) 는 event stream 그대로 유지. push 시점에 두 자료구조 모두 업데이트.

**Rationale**:
- Registry 는 O(1) lookup + small memory
- RingBuffer 는 timeline 표시 / detail view 용 그대로
- 데이터 중복 없음 — Registry 는 "last seen" 만 캐시

### ADR-MC-2: AgentCard 의 cost 필드

**Decision**: AgentCard.cost = `Option<CostSnapshot>` — 마지막 갱신 cost (input/output tokens + cost_usd). 매번 재계산 없이 push 시 갱신.

**Rationale**: CostTracker (V3-010) 의 결과를 그대로 캐시. 4-cell grid 가 매 frame O(N) 비용 계산하지 않음.

### ADR-MC-3: 4-cell vs N-cell

**Decision**: MS-2 는 default 4-cell fixed grid. 5번째 이상 run 은 카드 표시 안 함 (Registry 에는 모두 보유, view 가 select_top_n).

**Rationale**: Demo 시나리오 가시성 우선. N-cell scroll 은 MS-3+ carry.

### ADR-MC-4: Status pill 색상 매핑

| AgentRunStatus | 색상 | 라벨 |
|----------------|------|------|
| Running | ACCENT (청록) | "Running" |
| Paused | FG_MUTED (회색) | "Paused" |
| Completed | success_green | "Completed" |
| Failed | destructive_red | "Failed" |
| Killed | destructive_red | "Killed" |

`design::tokens` 의 기존 토큰 재사용. 신규 토큰 0.

---

## 4. Out of Scope

- Hook server (moai-hook-http) HTTP client 실 연결 — MS-3 carry 또는 별 SPEC.
- Per-agent kill / pause / resume dispatch — MissionControlView 는 read-only.
- Agent run 그룹화 (workspace 별, team 별) — 별 SPEC.
- AgentRun start trigger UI (사용자가 ⌘+R 등으로 새 agent 시작) — 별 SPEC.
- Mission Control 진입 키 바인딩 / Command Palette entry — MS-2 후속 PR.
- Performance: 100+ concurrent runs — 본 SPEC 은 ≤9 cell 가정.

---

## 5. References

- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier E + §4 Top 8 #2
- `.moai/specs/SPEC-V3-010/spec.md` (Agent Dashboard, V3-010 carry)
- `crates/moai-studio-agent/src/events.rs` — AgentEvent 도메인
- `crates/moai-studio-agent/src/sse_ingest.rs` — SSE 파서 scaffold
- `crates/moai-studio-agent/src/cost.rs` — CostTracker
- `crates/moai-studio-ui/src/agent/dashboard_view.rs` — V3-010 5-pane 컨테이너 (별 surface)
