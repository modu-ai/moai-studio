---
id: SPEC-V3-010
version: 1.0.1
status: implemented
created_at: 2026-04-25
updated_at: 2026-04-26
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-004]
parallel_with: [SPEC-V3-005, SPEC-V3-006, SPEC-V3-009]
optional_integration: [SPEC-V3-006]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, gpui, agent-runtime, hook-events, cost-tracking, moai-adk-integration]
revision: v1.0.0 (initial draft, agentic essence 시각화 SPEC)
---

# SPEC-V3-010: Agent Progress Dashboard — moai-adk 27 hook 이벤트 + agent run viewer + cost tracking + instructions graph

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-studio 의 agentic essence 영역 정의. 데이터 소스 3 (stream-json + hook-http + supervisor) → UI 6 view (timeline + cost + instructions + control + detail + dashboard). RG-AD-1 ~ RG-AD-6, AC-AD-1 ~ AC-AD-12, MS-1/MS-2/MS-3, USER-DECISION 4 게이트 (A: 영속화 / B: cost source / C: control IPC / D: hook ingestion). SPEC-V3-004 선행, V3-009 병행 (AgentRunId 공유), V3-006 선택적 통합. terminal/panes/tabs core 무변경 (RG-P-7 carry from V3-002/003). |
| 1.0.1-implemented | 2026-04-26 | MS-1 (#11), MS-2 (#35), MS-3 구현 완료. status draft → implemented. AC-AD-1~11 통과, AC-AD-12 (terminal/panes/tabs core git diff = 0) 검증 PASS. 도메인 14 + UI 23 = 37개 신규 테스트. follow-up 보류: REQ-AD-032 markdown 렌더 (V3-006 통합), USER-DECISION-AD-A2 SQLite 영속화, multi-agent dashboard. |

---

## 1. 개요

### 1.1 목적

본 SPEC 은 moai-studio 가 moai-adk Go CLI 의 agent runtime 을 GUI 로 시각화하는 핵심 화면을 정의한다. 세 가지 데이터 소스 (Claude Code subprocess stdout / moai-adk 27 hook events / moai-supervisor lifecycle) 를 통합하여 단일 dashboard 에 다음 6 개 시각화를 제공한다:

1. **EventTimelineView** — 27 hook event + stream message 의 통합 timeline (RG-AD-2)
2. **CostPanelView** — session / day / week 단위 USD 누적 비용 (RG-AD-3)
3. **InstructionsGraphView** — active instruction 의 layered tree (RG-AD-4)
4. **AgentControlBar** — pause / resume / kill 1-클릭 (RG-AD-5)
5. **EventDetailView** — 선택된 event 의 full JSON 인스펙터 (RG-AD-6)
6. **AgentDashboardView** — 위 5 view 의 split layout container

본 SPEC 은 **agentic essence 영역** — 즉 "지금 도는 agent 가 무얼 하고 있고, 얼마 들었으며, 어떤 instruction 이 active 인가" 를 시각화하는 화면. SPEC-V3-009 (SPEC Management UI) 가 SPEC 카드의 "Run" 버튼으로 spawn 한 agent 가 본 SPEC 의 dashboard 에서 라이브 stream 된다.

### 1.2 SPEC-V3-009 와의 cross-link

V3-009 와 V3-010 은 같은 `AgentRunId(uuid::Uuid)` 를 공유한다. 본 SPEC 이 그 ID 의 single source of truth 다.

```
SPEC-V3-009 SpecListView "Run" button click
    └─ MoaiCommandClient::spawn("moai run SPEC-XXX")
         ↓ AgentRunId 발행
         ├─ stdout (stream-json) ──→ SPEC-V3-010 EventTimelineView
         ├─ hook events  (SSE) ────→ SPEC-V3-010 EventTimelineView
         ├─ usage data ────────────→ SPEC-V3-010 CostPanelView
         └─ control msgs ←─────────  SPEC-V3-010 AgentControlBar
```

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-010/research.md` — 코드베이스 분석, 4 USER-DECISION 게이트, 데이터 모델 초안, AC 후보, 위험 8 종.
- `.moai/specs/SPEC-V3-004/research.md` — GPUI 0.2.2 Render trait 패턴.
- `.moai/specs/SPEC-V3-009/spec.md` §RG-SU-5 — MoaiCommandClient + AgentRunId 발행.
- `crates/moai-stream-json/src/{lib,decoder,message}.rs` — stream message 디코더 재사용.
- `crates/moai-hook-http/src/event_kind.rs` — 27 hook event enum (single source of truth).
- `.claude/rules/moai/workflow/moai-memory.md` — instruction hierarchy 6 levels (RG-AD-4 가 트리화).
- `.claude/rules/moai/core/moai-constitution.md` — 60fps + parallel execution 원칙.

---

## 2. 배경 및 동기

상세는 `.moai/specs/SPEC-V3-010/research.md` §1 ~ §3 참조. 최소 맥락만 요약:

- **Agentic essence 갭** (research §1.1): moai-studio 가 moai-adk shell 임에도 agent 의 라이브 진행을 보는 화면 부재. SPEC-V3-009 가 SPEC-side 라면 본 SPEC 은 agent-side.
- **27 hook event 미시각화 부채** (research §1.2): moai-hook-http 가 27 event 를 publish 하지만 시각적 인덱스 부재.
- **cost tracking 의 시각적 부재** (research §1.3): `/cost` 만으로는 SPEC 별 / day 별 aggregation 불가.
- **instructions graph 의 인지 부담** (research §1.4): 6 layer instruction hierarchy 가 어느 시점에 어떤 stack 인지 사용자 불투명.
- **agent control 의 부재** (research §1.5): pause / resume / kill 의 1-클릭 GUI 부재.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- **G1.** 활성 agent run 의 이벤트가 EventTimelineView 에 60fps throttle 로 라이브 표시된다.
- **G2.** stream-json 의 ContentBlock / ToolUse / ToolResult / Usage / Stop 메시지가 모두 timeline 에 반영된다.
- **G3.** moai-hook-http 가 publish 하는 27 hook event 가 모두 timeline 에 반영되며, kind 별 filter 가 동작한다.
- **G4.** CostPanelView 가 현재 session 의 누적 USD 와 일별 / 주별 aggregation 을 표시한다 (cost source: B1 self-report).
- **G5.** InstructionsGraphView 가 `SessionStart` 시점에 active instruction stack (managed policy → project → rules → user → local → memory → skills) 을 layered tree 로 렌더한다.
- **G6.** AgentControlBar 의 pause / resume / kill 버튼이 stdin envelope (`MOAI-CTRL: {json}`) 을 통해 동작한다.
- **G7.** EventDetailView 가 선택된 event 의 full JSON 을 pretty-print 한다.
- **G8.** terminal/panes/tabs core (RG-P-7 carry from V3-002/003) 코드는 변경되지 않는다.
- **G9.** macOS 14+ / Ubuntu 22.04+ 동일 동작 (Windows 비목표).
- **G10.** ring buffer 1000 events 한도에서 메모리 사용 < 50MB peak.

### 3.2 비목표 (Non-Goals)

- **N1.** 이벤트 디스크 영속화 (USER-DECISION-AD-A 의 A2 SQLite) — v1.0.0 default 는 ring buffer only. A2 는 follow-up SPEC.
- **N2.** OpenAI / GLM 의 cost calculation — moai-stream-json 의 책임. 본 SPEC 은 Claude self-report (B1) 만.
- **N3.** instruction 본문 편집 — InstructionsGraphView 는 read-only, 클릭 시 editor 에 open 만.
- **N4.** instruction 의존성 graph (skill A → skill B reference) 시각화 — 별 SPEC.
- **N5.** multi-agent concurrent dashboard — v1.0.0 은 active agent 1 개만 표시. agent picker 는 V3-009 책임 (cross-link).
- **N6.** terminal / panes / tabs core 변경 — RG-P-7 carry. 본 SPEC 은 신규 `moai-studio-agent` crate + `moai-studio-ui/src/agent_ui/` 모듈만.
- **N7.** Windows 빌드 — V3-002/003/004/009 N carry.
- **N8.** 새 design token 추가 — V3-001 토큰 (`app.background`, `panel.background`, `text.primary`, `status.{success,warning,error,info}`, `chart.{1..8}` for cost graph) 재사용.
- **N9.** mouse drag-and-drop 으로 timeline 재배치 — keyboard / scroll 만.
- **N10.** moai-supervisor crate 작성 — 본 SPEC 은 minimal `AgentHandle` trait stub 만 정의. 실 supervisor 는 별 SPEC.
- **N11.** B2 token×price 자체 cost 계산 — pricing table staleness 위험으로 거부.
- **N12.** C2 POSIX signal control — Windows 비목표지만 future-proof 측면 stdin envelope 채택.

---

## 4. 사용자 스토리

- **US-AD-1**: 사용자가 V3-009 의 SPEC 카드에서 "Run" 을 클릭하면 본 SPEC 의 AgentDashboardView 가 split pane 의 우측 또는 하단에 활성화되며 첫 stream message 가 1초 이내 timeline 에 표시된다.
- **US-AD-2**: 사용자가 27 hook event 중 `PostToolUse` 만 보고 싶을 때 timeline 의 filter chip 에서 `PostToolUse` 만 활성화하면 다른 kind 는 회색 처리된다 (제거가 아닌 dim 이라 context 보존).
- **US-AD-3**: 사용자가 timeline 의 한 event 를 클릭하면 EventDetailView 에 full JSON payload 가 pretty-print 되어 표시된다.
- **US-AD-4**: 사용자가 CostPanelView 에서 "이번 session $0.42 / today $1.85 / this week $7.20" 의 누적을 한눈에 본다.
- **US-AD-5**: 사용자가 InstructionsGraphView 에서 "지금 CLAUDE.md (priority 2) + 5 개 rules + 3 개 skills 가 active" 라는 layered tree 를 본다. 클릭 시 해당 파일이 editor 에 open 된다.
- **US-AD-6**: 사용자가 AgentControlBar 의 pause 버튼을 누르면 agent 가 즉시 일시정지되며 상태 badge 가 "Paused" (warning color) 로 변한다.
- **US-AD-7**: 사용자가 kill 버튼을 누르면 confirm dialog ("Kill agent run? Unsaved progress will be lost.") 가 뜨고 OK 시에만 SIGTERM 이 stdin envelope 으로 전달된다.
- **US-AD-8**: 사용자가 1000+ event burst (대량 batch tool call) 가 들어와도 UI lag 없이 60fps 가 유지되며 timeline 이 자동 ring buffer eviction 된다.
- **US-AD-9**: 사용자가 dashboard 를 닫고 다시 열면 ring buffer 의 마지막 1000 events 가 복원된다 (in-memory 만, 앱 재시작 시는 손실 — A1 default).

---

## 5. 기능 요구사항 (EARS)

### RG-AD-1 — Event Stream Ingestion (stream-json + hook-http SSE)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-AD-001 | Ubiquitous | 시스템은 활성 agent run 마다 고유한 `AgentRunId(uuid::Uuid)` 를 발행하며, 이 ID 는 SPEC-V3-009 의 SpecListView 와 공유된다. | The system **shall** mint a unique `AgentRunId` per active agent run, shared with SPEC-V3-009. |
| REQ-AD-002 | Event-Driven | Claude Code subprocess 가 stream-json 1 line 을 stdout 에 출력하면, 시스템은 `moai_stream_json::Decoder::decode_line` 를 호출하여 `AgentEventKind::Stream(StreamMessage)` 로 변환하고 ring buffer 에 push 한다. | When the subprocess writes a stream-json line, the system **shall** decode it via `moai_stream_json::Decoder` and push as `AgentEventKind::Stream`. |
| REQ-AD-003 | Event-Driven | moai-hook-http SSE endpoint (`/events/sse`) 가 1 hook event 를 publish 하면, 시스템은 `AgentEventKind::Hook(HookEventKind)` 로 변환하여 ring buffer 에 push 한다. SSE 는 USER-DECISION-AD-D 의 default 결정. | When moai-hook-http SSE publishes a hook event, the system **shall** convert it to `AgentEventKind::Hook` and push to the ring buffer (default per USER-DECISION-AD-D). |
| REQ-AD-004 | Ubiquitous | 시스템은 ring buffer 의 capacity 를 1000 events 로 제한하며 (USER-DECISION-AD-A default), capacity 초과 시 oldest event 를 evict 한다. | The system **shall** cap the ring buffer at 1000 events (per USER-DECISION-AD-A default) and evict oldest on overflow. |
| REQ-AD-005 | Unwanted | 시스템은 알려지지 않은 stream message kind 또는 hook event kind 를 받아도 panic 하지 않는다. `AgentEventKind::Unknown(String)` 으로 fallback 한다. | The system **shall not** panic on unknown stream/hook kinds; falls back to `AgentEventKind::Unknown`. |
| REQ-AD-006 | Ubiquitous | 시스템은 `AgentEventKind::Hook` 의 kind enumeration 을 hard-code 하지 않고 `moai_hook_http::EventKind::iter()` 로 동적 추출한다. | The system **shall** derive hook event kind enumeration dynamically from `moai_hook_http::EventKind::iter()`, never hard-coded. |
| REQ-AD-007 | Event-Driven | agent run 이 종료되면 (status: Completed / Failed / Killed), 시스템은 `AgentRun.ended_at` 을 set 하고 ring buffer 를 freeze 한다 (이후 이벤트 push 거부). | When an agent run terminates, the system **shall** set `ended_at` and freeze the ring buffer (reject further pushes). |

### RG-AD-2 — 27-Event Timeline + Filter

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-AD-008 | Ubiquitous | 시스템은 EventTimelineView 에서 ring buffer 의 모든 event 를 시간 역순 (최신 위) 으로 GPUI list 로 렌더한다. | The system **shall** render ring buffer events in reverse chronological order (newest top) in EventTimelineView. |
| REQ-AD-009 | Ubiquitous | 시스템은 EventTimelineView 의 렌더 throttle window 를 16ms (60fps) 로 설정하며, 동일 window 내 들어오는 같은 kind 의 burst events 는 batch 로 1 frame 에 렌더한다. | The system **shall** throttle EventTimelineView rendering to 16ms (60fps); same-kind bursts within one window are batched. |
| REQ-AD-010 | State-Driven | 시스템은 사용자가 활성화한 filter chip 의 kind 만 정상 색상으로 표시하고, 비활성 chip 의 kind 는 dim (50% opacity) 으로 표시한다 (제거가 아닌 dim 이어서 context 보존). | The system **shall** render only filter-active kinds at full opacity; inactive kinds dim to 50% (preserve context, do not remove). |
| REQ-AD-011 | Ubiquitous | 시스템은 filter chip 을 두 그룹으로 분리한다: (a) Stream messages (Content / ToolUse / ToolResult / Usage / Stop / Error), (b) Hook events (27 enum from `EventKind::iter()`). | The system **shall** split filter chips into two groups: stream messages and hook events. |
| REQ-AD-012 | Event-Driven | 사용자가 timeline 의 1 event 를 클릭하면, 시스템은 EventDetailView 에 해당 event 의 full JSON payload 를 전달한다 (RG-AD-6 와의 인터페이스). | When the user clicks an event, the system **shall** route the full JSON payload to EventDetailView. |
| REQ-AD-013 | Ubiquitous | 시스템은 timeline 에서 1000 events 를 ring buffer 가득 채운 상태에서도 60fps 를 유지한다 (G10 의 메모리 < 50MB peak 제약과 함께). | The system **shall** maintain 60fps with 1000 events in the ring buffer (alongside G10's <50MB peak memory). |

### RG-AD-3 — Cost Tracker (USD per session, daily/weekly aggregation)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-AD-014 | Event-Driven | stream-json `usage` field 가 도착하면, 시스템은 `Usage::extract` 를 호출하여 `CostSnapshot` 을 생성하고 현재 session 의 누적에 add 한다. cost source 는 USER-DECISION-AD-B 의 B1 self-report default. | When a stream-json `usage` arrives, the system **shall** extract `CostSnapshot` and add to session total (B1 self-report per USER-DECISION-AD-B). |
| REQ-AD-015 | Ubiquitous | 시스템은 CostPanelView 에 (a) current session USD, (b) today USD (앱 시작 후), (c) this ISO week USD 의 세 metric 을 표시한다. | The system **shall** display three metrics in CostPanelView: session, today, this ISO week (USD). |
| REQ-AD-016 | Ubiquitous | 시스템은 daily aggregation 을 in-memory `BTreeMap<NaiveDate, f64>` 로 유지하며, week aggregation 은 `BTreeMap<(year, ISO week), f64>` 로 유지한다. | The system **shall** maintain daily and weekly aggregations as in-memory BTreeMaps. |
| REQ-AD-017 | Unwanted | 시스템은 자체적으로 token×price 를 계산하지 않는다 (USER-DECISION-AD-B 의 B2 거부). API self-report 만 사용. | The system **shall not** compute cost from local token×price tables (B2 rejected); only API self-report is used. |
| REQ-AD-018 | State-Driven | `usage` field 가 있는 stream message 가 0 인 session 에서, 시스템은 cost 를 0.00 USD 로 표시하며 warning badge ("usage data unavailable") 를 노출한다. | While a session has zero `usage` messages, the system **shall** display $0.00 with a "usage data unavailable" warning badge. |

### RG-AD-4 — Instructions Graph (memory + CLAUDE.md + skills tree)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-AD-019 | Event-Driven | `SessionStart` hook event 가 도착하면, 시스템은 active instruction stack 을 6 layer (managed policy / project / rules / user / local / memory + skills) 로 스캔하여 `InstructionNode` tree 를 빌드한다. | When a `SessionStart` hook arrives, the system **shall** scan and build the 6-layer `InstructionNode` tree. |
| REQ-AD-020 | Event-Driven | `PreCompact` 또는 `PrePromptInject` hook 이 도착하면, 시스템은 instruction tree 를 재빌드한다 (skill auto-load 또는 context 변경 반영). | When `PreCompact` or `PrePromptInject` arrives, the system **shall** rebuild the instruction tree. |
| REQ-AD-021 | Ubiquitous | 시스템은 InstructionsGraphView 에서 InstructionNode tree 를 indent 기반 layered tree 로 렌더하며, 각 node 는 source path 와 priority 를 표시한다. | The system **shall** render the tree in InstructionsGraphView with indentation, showing source path and priority. |
| REQ-AD-022 | Event-Driven | 사용자가 instruction node 를 클릭하면, 시스템은 해당 source path 를 OS 의 default editor 에 open 한다 (xdg-open / open). | When the user clicks a node, the system **shall** open the source path in the OS default editor. |
| REQ-AD-023 | Unwanted | 시스템은 InstructionsGraphView 에서 instruction 본문을 편집할 수 없게 한다 (read-only, N3). | The system **shall not** allow editing instruction content within InstructionsGraphView (read-only). |

### RG-AD-5 — Live Agent Control (pause / resume / kill)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-AD-024 | Ubiquitous | 시스템은 AgentControlBar 에 3 개 버튼 (pause / resume / kill) 을 두고, 현재 `AgentRunStatus` 에 따라 enabled / disabled 를 토글한다. | The system **shall** display three buttons (pause/resume/kill) toggled per current `AgentRunStatus`. |
| REQ-AD-025 | Event-Driven | 사용자가 pause 버튼을 누르면, 시스템은 agent stdin 으로 `MOAI-CTRL: {"action":"pause","run_id":"..."}` envelope (USER-DECISION-AD-C 의 C1 default) 을 전달한다. | When pause is clicked, the system **shall** write a `MOAI-CTRL: {...}` envelope to agent stdin (C1 default per USER-DECISION-AD-C). |
| REQ-AD-026 | Event-Driven | 사용자가 resume 버튼을 누르면, 시스템은 `MOAI-CTRL: {"action":"resume","run_id":"..."}` 를 전달한다. | When resume is clicked, the system **shall** write a resume envelope. |
| REQ-AD-027 | Event-Driven | 사용자가 kill 버튼을 누르면, 시스템은 GPUI native modal 로 confirm dialog 를 띄우고 OK 시에만 `MOAI-CTRL: {"action":"kill","run_id":"..."}` 를 전달한다. | When kill is clicked, the system **shall** show a confirm modal and only on OK send the kill envelope. |
| REQ-AD-028 | State-Driven | agent 가 control envelope 에 응답한 후 status 변경 hook (`Stop` for kill / custom for pause) 이 도착하면, 시스템은 `AgentRunStatus` 를 갱신하고 button enabled 상태를 재계산한다. | While the agent acks via status hook, the system **shall** update `AgentRunStatus` and recompute button enablement. |
| REQ-AD-029 | Unwanted | 시스템은 confirm dialog 없이 kill 을 즉시 실행하지 않는다. | The system **shall not** execute kill without a confirmation dialog. |

### RG-AD-6 — Event Drilldown (full JSON inspector)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-AD-030 | Event-Driven | 사용자가 timeline 에서 event 를 선택하면 (REQ-AD-012 의 routing), 시스템은 EventDetailView 에 해당 event 의 full JSON payload 를 2-space indent 로 pretty-print 한다. | When an event is selected, the system **shall** pretty-print the full JSON in EventDetailView (2-space indent). |
| REQ-AD-031 | Ubiquitous | 시스템은 EventDetailView 에서 JSON 의 nested object 를 collapse / expand 할 수 있는 disclosure 트라이앵글을 제공한다. | The system **shall** provide collapse/expand disclosure triangles for nested JSON objects. |
| REQ-AD-032 | Optional | 시스템은 가능한 경우 EventDetailView 의 markdown payload (예: `Notification` 의 message body) 를 SPEC-V3-006 markdown viewer 로 렌더한다. MS-3 default 는 JSON-only, markdown 은 follow-up. | Where possible, the system **shall** render markdown payloads via SPEC-V3-006 (MS-3 follow-up; JSON-only by default). |
| REQ-AD-033 | Ubiquitous | 시스템은 EventDetailView 에서 "Copy as JSON" 버튼을 제공하여 clipboard 에 raw JSON 을 복사한다. | The system **shall** provide a "Copy as JSON" button copying raw JSON to clipboard. |

---

## 6. 비기능 요구사항 (NFR)

- **NFR-AD-1**: ring buffer push throughput ≥ 1000 events/sec (G10 의 메모리 제약 하에).
- **NFR-AD-2**: EventTimelineView render latency ≤ 16ms p95 (60fps).
- **NFR-AD-3**: peak memory < 50MB with 1000 events in ring buffer (G10).
- **NFR-AD-4**: SSE reconnection 시 missed event 손실은 best-effort (영속화는 N1 비목표).
- **NFR-AD-5**: cost calculation 의 precision: 4 decimal USD (e.g., $0.0042).
- **NFR-AD-6**: instruction tree rebuild ≤ 200ms p95 (6 layer 스캔).
- **NFR-AD-7**: agent control envelope round-trip (click → status update) ≤ 500ms p95.
- **NFR-AD-8**: macOS 14+ / Ubuntu 22.04+ 동일 빌드 산출 (Windows 비목표).
- **NFR-AD-9**: code_comments=ko 정책 준수 (CLAUDE.local.md / language.yaml).
- **NFR-AD-10**: terminal/panes/tabs core git diff = 0 (G8, AC-AD-12 검증).

---

## 7. 인터페이스 (Interfaces)

### 7.1 Rust crate boundaries

- 신규 crate: `crates/moai-studio-agent/` — RG-AD-1/3/4 도메인 (ring buffer, cost aggregation, instruction tree)
- 신규 모듈: `crates/moai-studio-ui/src/agent_ui/` — 6 view (RG-AD-2/5/6 + dashboard container)
- 외부 crate dependency: `moai-stream-json`, `moai-hook-http`, (optional) `moai-supervisor`

### 7.2 SPEC-V3-009 와의 공유 타입

- `moai_studio_agent::AgentRunId` — single source of truth, V3-009 가 import.
- V3-009 의 `MoaiCommandClient::spawn` 이 본 SPEC 의 `EventStreamSubscriber` 에게 process handle 전달.

### 7.3 SPEC-V3-006 와의 optional 통합

- RG-AD-6 의 REQ-AD-032 가 markdown 렌더 지점. v1.0.0 default 는 JSON-only.

---

## 8. USER-DECISION 게이트 (research §3 요약)

본 SPEC body 는 다음 default 결정을 가정한다. override 시 plan.md 갱신 필수.

- **USER-DECISION-AD-A** (영속화): **A1** in-memory ring buffer 1000 events
- **USER-DECISION-AD-B** (cost source): **B1** agent self-report (stream-json `usage`)
- **USER-DECISION-AD-C** (control IPC): **C1** stdin envelope `MOAI-CTRL: {json}`
- **USER-DECISION-AD-D** (hook ingestion): **D2** HTTP server-sent events (SSE)

---

## 9. Acceptance Criteria

상세는 `.moai/specs/SPEC-V3-010/acceptance.md` 에 enumerate. spec.md 에는 ID 만 reference.

| AC ID | 영역 | RG | 우선 |
|-------|------|-----|------|
| AC-AD-1 | stream-json 1 line decode → AgentEvent 변환 | RG-AD-1 | P0 |
| AC-AD-2 | hook-http SSE 1 event → AgentEvent 변환 | RG-AD-1 | P0 |
| AC-AD-3 | EventTimelineView 1000 events 60fps render | RG-AD-2 | P0 |
| AC-AD-4 | event filter chip dim/full toggle | RG-AD-2 | P0 |
| AC-AD-5 | CostPanel session 누적 USD 표시 | RG-AD-3 | P0 |
| AC-AD-6 | CostPanel daily/weekly aggregation correct | RG-AD-3 | P1 |
| AC-AD-7 | InstructionsGraph 6 layer tree 표시 | RG-AD-4 | P1 |
| AC-AD-8 | InstructionsGraph SessionStart/PreCompact rebuild | RG-AD-4 | P1 |
| AC-AD-9 | AgentControlBar pause envelope 전달 | RG-AD-5 | P0 |
| AC-AD-10 | AgentControlBar kill confirm dialog + envelope | RG-AD-5 | P0 |
| AC-AD-11 | EventDetailView JSON pretty-print + collapse | RG-AD-6 | P1 |
| AC-AD-12 | terminal/panes/tabs core git diff = 0 | (RG-P-7 carry) | P0 |

---

## 10. Milestone 매핑

상세 milestone breakdown 은 `.moai/specs/SPEC-V3-010/plan.md` 참조.

- **MS-1** (Stream parser + 기본 timeline): RG-AD-1, RG-AD-2 (filter 제외 minimal), AC-AD-1/2/3/12
- **MS-2** (Cost tracker + filter): RG-AD-3, RG-AD-2 의 filter 부분, AC-AD-4/5/6
- **MS-3** (Instructions graph + agent control): RG-AD-4, RG-AD-5, RG-AD-6, AC-AD-7/8/9/10/11

---

## 11. 의존성 / 영향 범위

### 11.1 Upstream (선행)

- **SPEC-V3-004** (Render Layer) — Render trait pattern (필수)

### 11.2 Parallel

- **SPEC-V3-009** (SPEC Management UI) — `AgentRunId` schema 동기화 협업
- **SPEC-V3-005**, **SPEC-V3-006** — 무관 변경 영역

### 11.3 Optional integration

- **SPEC-V3-006** (Markdown Viewer) — RG-AD-6 의 markdown payload 렌더 (MS-3 follow-up, default 비목표)

### 11.4 Downstream (본 SPEC 이 unblock 하는)

- (가설) SPEC-V3-Future-Supervisor — 본 SPEC 의 `AgentHandle` trait stub 을 실 supervisor crate 로 확장
- (가설) SPEC-V3-Future-MultiAgent — N5 거부된 multi-agent dashboard 의 follow-up

---

## 12. 위험 (research §4 요약)

| 위험 | 완화 |
|------|------|
| R1: stream-json schema drift | `EventKind::Unknown` fallback (REQ-AD-005) |
| R2: 27 hook event enum staleness | dynamic `EventKind::iter()` (REQ-AD-006) |
| R3: 60fps event burst | 16ms coalesce (REQ-AD-009) |
| R4: cost API divergence | self-report only (REQ-AD-017) |
| R5: instruction graph 동적 변경 | hook trigger rebuild (REQ-AD-019/020) |
| R6: kill 우발 클릭 | confirm dialog (REQ-AD-027) |
| R7: SQLite 의존 회피 | A1 ring buffer default (REQ-AD-004) |
| R8: AgentRunId schema 충돌 | 본 SPEC 이 single source of truth (REQ-AD-001) |

---

## 13. 변경 영향 범위

### 13.1 신규 파일

- `crates/moai-studio-agent/Cargo.toml`
- `crates/moai-studio-agent/src/lib.rs`
- `crates/moai-studio-agent/src/run.rs` — `AgentRun`, `AgentRunId`, `AgentRunStatus`
- `crates/moai-studio-agent/src/event.rs` — `AgentEvent`, `AgentEventKind`
- `crates/moai-studio-agent/src/ring_buffer.rs`
- `crates/moai-studio-agent/src/cost.rs` — `CostSnapshot`, `CostAggregation`
- `crates/moai-studio-agent/src/instructions.rs` — `InstructionNode`, scanner
- `crates/moai-studio-agent/src/control.rs` — stdin envelope writer
- `crates/moai-studio-ui/src/agent_ui/mod.rs`
- `crates/moai-studio-ui/src/agent_ui/dashboard_view.rs`
- `crates/moai-studio-ui/src/agent_ui/timeline_view.rs`
- `crates/moai-studio-ui/src/agent_ui/cost_panel_view.rs`
- `crates/moai-studio-ui/src/agent_ui/instructions_graph_view.rs`
- `crates/moai-studio-ui/src/agent_ui/control_bar.rs`
- `crates/moai-studio-ui/src/agent_ui/detail_view.rs`

### 13.2 수정 파일

- `Cargo.toml` (workspace) — 신규 crate 추가
- `crates/moai-studio-ui/Cargo.toml` — `moai-studio-agent` dependency 추가
- `crates/moai-studio-ui/src/lib.rs` — `agent_ui` 모듈 export

### 13.3 무변경 (HARD)

- `crates/moai-studio-terminal/**` (RG-P-7 carry)
- `crates/moai-studio-ui/src/panes/**` (RG-P-7 carry)
- `crates/moai-studio-ui/src/tabs/**` (RG-P-7 carry)

AC-AD-12 가 git diff 검증.

---

Version: 1.0.0 (initial draft)
Last Updated: 2026-04-25
Author: MoAI (manager-spec)
Language: ko
