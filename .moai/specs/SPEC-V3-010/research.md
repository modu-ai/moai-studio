---
spec_id: SPEC-V3-010
artifact: research
version: 1.0.0
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
related: [SPEC-V3-004, SPEC-V3-009, SPEC-V3-002, SPEC-V3-003, SPEC-V3-006]
language: ko
---

# SPEC-V3-010 Research — Agent Progress Dashboard

## 1. 배경 및 동기 (Why now)

### 1.1 moai-studio 의 비전 갭

moai-studio 는 moai-adk Go CLI 의 GUI shell 로 출발했지만, agent runtime 의 **agentic essence** (실시간 hook 이벤트 + cost tracking + agent control) 를 시각화하는 화면이 아직 없다. SPEC-V3-009 가 `.moai/specs/` 디렉터리 자체의 시각화를 책임진다면, **본 SPEC (V3-010) 은 moai-adk 가 spawn 하는 agent process 의 라이브 진행 상황을 책임진다.**

차이점을 명확히 하면:

| 축 | SPEC-V3-009 (SPEC Management UI) | SPEC-V3-010 (Agent Progress Dashboard) |
|----|----------------------------------|----------------------------------------|
| 데이터 소스 | `.moai/specs/SPEC-*/` markdown 파일 | Claude Code subprocess stdout (stream-json) + moai-adk 27 hook events |
| 시간성 | quasi-static (파일 변경 시 갱신) | real-time stream (60fps throttled) |
| 주 사용자 가치 | "내 SPEC 들이 어디까지 왔지?" | "지금 도는 agent 가 무얼 하고 있고 얼마 들었지?" |
| 핵심 컴포넌트 | SpecListView / KanbanBoardView | EventTimelineView / CostPanel / InstructionsGraph |
| 수명 | quasi-permanent (SPEC lifetime) | session-scoped (agent run lifetime) |

### 1.2 moai-adk 27 hook event 의 미시각화 부채

moai-adk 의 `crates/moai-hook-http` 는 이미 27 개의 hook event 를 HTTP server 로 노출하고 있다. CLI 에서는 `moai hook` 서브커맨드로 stdout 출력만 가능하며, 어느 SPEC 의 어느 phase 에서 어떤 event 가 발생했는지 시각적 인덱싱이 부재하다. 27 개 event 를 시간축과 phase 축 두 dimension 으로 보는 timeline 이 본 SPEC 의 핵심.

알려진 27 events (research §6 에 enumerate):
- Lifecycle (4): `SessionStart`, `SessionEnd`, `Stop`, `SubagentStop`
- Pre/Post tool (8): `PreToolUse`, `PostToolUse`, `PreToolBatch`, `PostToolBatch`, `PrePromptInject`, `PostPromptInject`, `PreFileRead`, `PostFileRead`
- Notifications (4): `Notification`, `UserPromptSubmit`, `AssistantResponseEnd`, `ThinkingBlockEnd`
- Worktree (2): `WorktreeCreate`, `WorktreeRemove`
- Team (4): `TeammateIdle`, `TaskCompleted`, `TaskCreated`, `TaskUpdated`
- Compaction (2): `PreCompact`, `PostCompact`
- Custom (3): `MoaiHookCustom1`, `MoaiHookCustom2`, `MoaiHookCustom3`

(정확한 27 enumeration 은 `crates/moai-hook-http/src/event_kind.rs` 의 enum variant count 와 동기화한다 — 본 SPEC 의 RG-AD-1 이 이 enum 을 single source of truth 로 한다.)

### 1.3 cost tracking 의 시각적 부재

Claude Code stream-json 은 매 turn 마다 `usage.input_tokens` / `usage.output_tokens` / `usage.cache_creation_input_tokens` / `usage.cache_read_input_tokens` 를 노출한다. 그러나 사용자가 한 sprint 동안 누적 비용을 보려면 stream JSON 을 manual 로 grep 해야 한다. `/cost` 슬래시 커맨드만으로는 SPEC 별 / day 별 / week 별 aggregation 이 불가능하다.

### 1.4 instructions graph 의 인지 부담

CLAUDE.md / CLAUDE.local.md / `.claude/rules/**/*.md` / `.claude/skills/**/*.md` / `~/.claude/projects/{hash}/memory/MEMORY.md` 가 layered 로 적용되는데, 어느 시점에 어느 instruction 이 active 인지를 사용자가 직접 알기 어렵다. tree view 로 active instruction stack 을 보여주는 InstructionsGraph 가 transparency 를 제공한다.

### 1.5 agent control 의 부재

현재 사용자가 진행 중인 agent 를 멈추려면 (a) terminal 에서 `Ctrl+C` 누르거나 (b) Claude Code interface 에서 `Ctrl+X Ctrl+K` (background agent) 를 누르는 두 경로뿐이다. moai-studio 는 GUI 에서 `pause` / `resume` / `kill` 를 1-클릭으로 노출해야 한다.

---

## 2. 코드베이스 분석

### 2.1 기존 crate 자산

본 SPEC 은 신규 crate 를 최소화하고 **세 개의 기존 crate 를 재사용** 한다.

#### 2.1.1 `crates/moai-stream-json/`

- 책임: Claude Code subprocess stdout 의 stream-json 라인 단위 파싱
- 핵심 타입 추정: `StreamMessage` enum (Content / ToolUse / ToolResult / Usage / Error / Stop), `Decoder::decode_line(&str) -> Result<StreamMessage>`
- 본 SPEC 의 활용: RG-AD-1 의 Source-1 ingestion path. 이미 검증된 decoder 를 그대로 사용. 신규 wrapper `EventStreamSubscriber` 를 본 SPEC 에서만 추가.

#### 2.1.2 `crates/moai-hook-http/`

- 책임: moai-adk 27 hook event 를 HTTP webhook 으로 publish
- 핵심 타입 추정: `HookEvent` struct (kind / spec_id / phase / payload / timestamp), `EventKind` enum (27 variants)
- 본 SPEC 의 활용: RG-AD-1 의 Source-2 ingestion path. HTTP listener 를 dashboard process 가 subscribe 한다.
- Decision Point: pull (HTTP poll) vs push (HTTP server-sent events). research §3 USER-DECISION-AD-D 에서 결정.

#### 2.1.3 `moai-supervisor` (예정 또는 기존)

- 책임: agent subprocess lifecycle 관리 (spawn / pause / kill)
- 본 SPEC 의 활용: RG-AD-5 의 control IPC 백엔드. supervisor 가 이미 있다면 그대로, 없다면 본 SPEC 의 비목표 (별 SPEC 에서 작성).
- 가정: 현재 시점 (2026-04-25) 에서 `moai-supervisor` 는 아직 부재. 본 SPEC 은 minimal supervisor stub (`AgentHandle::pause/resume/kill` trait) 만 정의하고 실제 supervisor crate 작성은 별 SPEC 으로 분리 권고.

### 2.2 GPUI 0.2.2 + SPEC-V3-004 render layer 의존성

본 SPEC 의 모든 view 컴포넌트는 SPEC-V3-004 가 정의한 render trait 패턴을 따른다:

```rust
// SPEC-V3-004 가 확립한 패턴 (예시)
impl Render for AgentDashboardView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // ...
    }
}
```

본 SPEC 의 신규 view:
- `AgentDashboardView` — 최상위 split view (timeline + cost + instructions + control)
- `EventTimelineView` — 27-event timeline + filter
- `CostPanelView` — USD aggregation (session / day / week)
- `InstructionsGraphView` — active instruction tree
- `EventDetailView` — full JSON drilldown (RG-AD-6)
- `AgentControlBar` — pause / resume / kill buttons

### 2.3 SPEC-V3-009 와의 cross-link

SPEC-V3-009 (SPEC Management UI) 는 SPEC 카드에 "Run" 버튼을 두고 `moai run SPEC-XXX` 를 spawn 한다. **본 SPEC (V3-010) 은 V3-009 의 "Run" 클릭으로 spawn 된 subprocess 를 받아서 dashboard 로 보여준다.** 즉:

```
SPEC-V3-009 SpecListView
    └─ "Run" button click
         └─ MoaiCommandClient::spawn("moai run SPEC-V3-XXX")
              └─ stream-json output → SPEC-V3-010 EventTimelineView
              └─ hook events → SPEC-V3-010 EventTimelineView
              └─ usage data → SPEC-V3-010 CostPanelView
              └─ control messages ← SPEC-V3-010 AgentControlBar
```

V3-009 와 V3-010 은 같은 `AgentRunSession` ID 를 공유한다. 이 ID 는 본 SPEC 에서 정의 (`AgentRunId(uuid::Uuid)`).

### 2.4 markdown 렌더 의존 (SPEC-V3-006)

SPEC-V3-006 의 markdown viewer 는 본 SPEC 의 EventDetailView 에서 JSON pretty-print 와 함께 stack trace markdown 렌더에 재사용한다. (선택적, MS-3 default 비목표 — JSON-only 시작.)

### 2.5 V3-002/003 carry: terminal/panes/tabs 무변경 [HARD]

본 SPEC 은 `crates/moai-studio-terminal`, `crates/moai-studio-ui/src/panes/`, `crates/moai-studio-ui/src/tabs/` 의 코어 코드를 변경하지 않는다. RG-P-7 carry from SPEC-V3-002/003.

신규 코드는 다음 위치에만:
- `crates/moai-studio-agent/` (신규 crate, RG-AD-1/3/4 의 도메인 로직)
- `crates/moai-studio-ui/src/agent_ui/` (신규 모듈, 6 신규 view)

---

## 3. USER-DECISION 게이트

본 SPEC 은 다음 4 개 의사결정 gate 를 정의한다. SPEC body 의 RG/AC 는 default 결정을 가정하지만 사용자가 override 시 plan.md 가 갱신되어야 한다.

### USER-DECISION-AD-A: 이벤트 영속화 전략

| 옵션 | 설명 | 디스크 비용 | 복잡도 | 권장 |
|------|------|-------------|--------|------|
| A1: in-memory ring buffer (1000 events) | 메모리에만 보존, 앱 재시작 시 손실 | 0 | 낮음 | **default** |
| A2: SQLite log + ring buffer fallback | 디스크 영속, 세션 간 retrospective 가능 | 약 200KB / hour | 중간 | MS-3 follow-up |
| A3: in-memory only (unbounded) | 무제한, OOM 위험 | 0 | 낮음 | 거부 |
| A4: file-based JSONL log | append-only line 단위 | 약 100KB / hour | 낮음 | A2 대안 |

**Default**: A1. ring buffer capacity 1000 events. 사용자가 더 많은 history 를 원하면 plan.md MS-3 에서 A2 로 upgrade.

**Why A1 default**: 
- 디스크 lifecycle 관리 부담이 ring buffer 보다 큼
- 27 event × 평균 0.5KB ≈ 13.5KB peak memory (1000 events) — 무시 가능
- session 간 retrospective 는 v1.0.0 의 핵심 가치가 아님

**Risk if wrong**: 사용자가 retrospective 를 강하게 원하면 A2 로 upgrade 필요. plan.md MS-3 에 A2 path 미리 enumerate.

### USER-DECISION-AD-B: cost source 전략

| 옵션 | 설명 | 정확도 | 권장 |
|------|------|--------|------|
| B1: agent self-report (stream-json `usage` field) | Claude API 가 직접 보고 | 100% (API 신뢰) | **default** |
| B2: token × price 자체 계산 (model 별 pricing table) | local pricing table maintain | 95% (table staleness) | 거부 |
| B3: B1 + B2 cross-check | 양쪽 비교, mismatch 시 warning | 100% + audit | follow-up |

**Default**: B1. stream-json 의 `usage` field 를 그대로 신뢰.

**Why B1 default**:
- pricing table 은 staleness 위험이 큼 (Claude pricing 자주 변경)
- API 가 정확한 actual cost 를 보고하므로 redundant calculation 불필요
- model name → pricing 매핑 table 을 본 SPEC 에서 유지하지 않음

**Risk if wrong**: API 가 usage 를 누락하는 model (가설적) 에서 cost 가 0 으로 표시. fallback 으로 B2 가 follow-up SPEC.

### USER-DECISION-AD-C: agent control IPC 채널

| 옵션 | 설명 | 호환성 | 권장 |
|------|------|--------|------|
| C1: subprocess stdin command (JSON line) | dashboard → agent 의 stdin 으로 control message | Claude Code v2.1.97+ 검증 필요 | **default** |
| C2: POSIX signal (SIGSTOP / SIGCONT / SIGTERM) | OS-native, cross-platform unstable | macOS/Linux OK, Windows N/A | 거부 (Windows 비목표지만 future-proof 측면 거부) |
| C3: HTTP control endpoint | supervisor 가 별도 port 노출 | supervisor crate 의존 | C1 대안 |
| C4: Unix domain socket | local-only, low overhead | macOS/Linux OK | C1 대안 |

**Default**: C1. dashboard 에서 agent stdin 으로 `{"type":"control","action":"pause"}` JSON line 전달.

**Why C1 default**:
- agent stdin 은 이미 Claude Code session 입력 채널이라 추가 IPC layer 불필요
- C2 signal 은 process group 관리가 까다롭고 Claude Code 내부 동작과 충돌 가능
- C3 HTTP 는 supervisor crate 가 prerequisite — 본 SPEC scope 초과
- C4 socket 은 macOS/Linux OK 하지만 C1 대비 추가 가치 적음

**Risk if wrong**: Claude Code 가 stdin 을 직접 prompt 로 해석하면 control message 가 prompt injection 처럼 보일 수 있음 — 본 SPEC 은 `MOAI_CONTROL_PREFIX` envelope (`MOAI-CTRL: {json}`) 으로 isolation. 검증은 MS-3 spike.

### USER-DECISION-AD-D: hook event ingestion 모드

| 옵션 | 설명 | 지연 | 권장 |
|------|------|------|------|
| D1: HTTP poll (1초 간격) | dashboard → moai-hook-http GET | 1s avg | 거부 (real-time 위반) |
| D2: HTTP server-sent events (SSE) | moai-hook-http push to dashboard | <100ms | **default** |
| D3: WebSocket | bi-directional | <100ms | D2 대안 (overkill for one-way) |
| D4: file watch on hook log | dashboard ← fs notify | <500ms | D2 대안 |

**Default**: D2 SSE. moai-hook-http 가 SSE endpoint `/events/sse` 를 노출 (SPEC 본문 RG-AD-1 가정).

**Why D2 default**:
- 60fps real-time UX 요구사항과 부합
- HTTP 기반이라 firewall / sandbox 친화
- one-way push 면 충분 (control 은 별도 channel — USER-DECISION-AD-C)

**Risk if wrong**: moai-hook-http 가 SSE endpoint 를 미지원하면 본 SPEC 이 그쪽에 추가 요구사항 발생. 검증은 MS-1 첫 spike.

---

## 4. 위험 요약 (Risks)

### R1: stream-json schema drift
Claude Code stream-json schema 는 minor version 마다 미묘한 변경. moai-stream-json crate 가 schema 를 흡수하지만 신규 message variant 가 추가되면 EventTimelineView 의 filter 가 매핑 누락 가능.

**완화**: `EventKind::Unknown(serde_json::Value)` fallback variant 를 본 SPEC 의 RG-AD-1 에서 명시. 미매핑 event 는 "Unknown" lane 으로 표시.

### R2: 27 hook event enum 의 staleness
moai-adk 가 새 hook event 를 추가하면 본 SPEC 이 hard-coded "27" 으로 표시한 부분이 잘못됨.

**완화**: dashboard 는 `EventKind::iter()` 로 dynamic count. spec.md 본문에서 "27" 표기는 informative 만, normative 는 enum 자체.

### R3: 60fps event stream throttling
초당 100+ events (e.g., 대량 PreToolUse) 가 들어오면 GPUI 렌더가 백로그.

**완화**: RG-AD-2 에서 16ms (60fps) coalesce window 를 명시. 같은 event_kind 의 burst 는 1 frame 내 batch.

### R4: cost calculation 의 Claude vs other-API divergence
사용자가 Claude 가 아닌 OpenAI/GLM 모델을 사용하면 stream-json `usage` schema 가 다름.

**완화**: B1 default 가 self-report 이므로 API 별 schema 차이는 moai-stream-json 의 책임. 본 SPEC 은 `Usage::extract(StreamMessage) -> Option<Cost>` trait 만 정의.

### R5: instruction graph 의 동적 변경 추적
session 중간에 `/clear` 하거나 새 skill 이 auto-load 되면 active instruction 이 변함. 정적 tree 로는 부족.

**완화**: RG-AD-4 가 `SessionStart` / `PreCompact` / `PrePromptInject` hook 을 trigger 로 instruction graph 재빌드. tree 는 reactive.

### R6: agent control 의 destructive action 안전성
"kill" 버튼이 우발 클릭되면 작업 손실.

**완화**: RG-AD-5 가 destructive action (kill) 에 대해 explicit confirm dialog (GPUI native modal) 강제. AC-AD-12 에서 검증.

### R7: SQLite 의존 회피 (USER-DECISION-AD-A 의 follow-up)
A1 ring buffer default 채택으로 sqlite 의존 회피. 그러나 A2 로 upgrade 시 `rusqlite` 추가 시 cross-platform 빌드 검증 필요 (macOS arm64 / Ubuntu x86_64).

**완화**: 본 SPEC v1.0.0 은 A1 만. A2 follow-up 시 별 SPEC 으로 분리.

### R8: SPEC-V3-009 의 AgentRunId 미정
본 SPEC 이 `AgentRunId(uuid::Uuid)` 를 정의하면 V3-009 의 SpecListView 와 ID 공유 필요. V3-009 가 아직 draft 라 schema 충돌 가능.

**완화**: 본 SPEC 의 RG-AD-1 가 `AgentRunId` 를 single source of truth. V3-009 는 본 SPEC 을 import.

---

## 5. 데이터 모델 초안 (Pre-spec)

### 5.1 핵심 타입 (Rust)

```rust
// crates/moai-studio-agent/src/lib.rs 추정
pub struct AgentRunId(pub uuid::Uuid);

pub struct AgentRun {
    pub id: AgentRunId,
    pub spec_id: Option<String>,        // e.g., "SPEC-V3-010"
    pub command: String,                 // e.g., "moai run SPEC-V3-010"
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: AgentRunStatus,
}

pub enum AgentRunStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
    Killed,
}

pub struct AgentEvent {
    pub run_id: AgentRunId,
    pub kind: AgentEventKind,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

pub enum AgentEventKind {
    Stream(moai_stream_json::StreamMessage),
    Hook(moai_hook_http::HookEventKind),  // 27 variants
    Lifecycle(AgentRunStatus),             // status transition
    Unknown(String),                        // R1 fallback
}
```

### 5.2 cost aggregation

```rust
pub struct CostSnapshot {
    pub run_id: AgentRunId,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub usd_total: f64,                  // self-reported (B1)
}

pub struct CostAggregation {
    pub by_session: HashMap<AgentRunId, CostSnapshot>,
    pub by_day: BTreeMap<chrono::NaiveDate, f64>,
    pub by_week: BTreeMap<(i32, u32), f64>,  // (year, ISO week)
}
```

### 5.3 instruction node

```rust
pub struct InstructionNode {
    pub kind: InstructionKind,
    pub source_path: PathBuf,
    pub priority: u8,                    // moai-memory hierarchy 1~6
    pub active: bool,
    pub children: Vec<InstructionNode>,
}

pub enum InstructionKind {
    ManagedPolicy,
    ProjectInstructions,                 // CLAUDE.md
    ProjectRules,                        // .claude/rules/**/*.md
    UserInstructions,                    // ~/.claude/CLAUDE.md
    LocalInstructions,                   // CLAUDE.local.md
    AutoMemory,                          // ~/.claude/projects/{hash}/memory/
    SkillBody,                           // .claude/skills/**/*.md
}
```

---

## 6. 기존 SPEC 과의 dependency graph

```
SPEC-V3-004 (Render Layer)
    ↓ provides Render trait pattern
SPEC-V3-010 (Agent Dashboard)  ←─ SPEC-V3-006 (Markdown Viewer, optional MS-3)
    ↑ provides AgentRunId
SPEC-V3-009 (SPEC Management UI)
```

본 SPEC 은:
- SPEC-V3-004 **선행 필수** (Render trait)
- SPEC-V3-006 **선택적** (markdown 렌더 — MS-3 default 비목표)
- SPEC-V3-009 **병행** (AgentRunId 공유, schema 동기화 협업)

---

## 7. AC 후보 (spec.md 의 AC-AD-* 사전 enumeration)

본 research 단계에서 14 개 AC 후보를 식별했고 spec.md 에서 최종 12 개로 정리한다 (2 개는 acceptance.md 의 nice-to-have 로 demote).

| ID | 영역 | RG | 우선 |
|----|------|-----|------|
| AC-AD-1 | stream-json 1 line decode → AgentEvent | RG-AD-1 | P0 |
| AC-AD-2 | hook-http SSE 1 event → AgentEvent | RG-AD-1 | P0 |
| AC-AD-3 | EventTimelineView 1000 events 60fps render | RG-AD-2 | P0 |
| AC-AD-4 | event filter (kind, time range) 동작 | RG-AD-2 | P0 |
| AC-AD-5 | CostPanel session 누적 USD 표시 | RG-AD-3 | P0 |
| AC-AD-6 | CostPanel daily aggregation correct | RG-AD-3 | P1 |
| AC-AD-7 | InstructionsGraph CLAUDE.md / rules / skills 트리 표시 | RG-AD-4 | P1 |
| AC-AD-8 | InstructionsGraph SessionStart 시 rebuild | RG-AD-4 | P1 |
| AC-AD-9 | AgentControlBar pause → status: Paused 전이 | RG-AD-5 | P0 |
| AC-AD-10 | AgentControlBar kill → status: Killed + confirm dialog | RG-AD-5 | P0 |
| AC-AD-11 | EventDetailView JSON pretty-print | RG-AD-6 | P1 |
| AC-AD-12 | terminal/panes/tabs core 파일 무변경 git diff 검증 | (RG-P-7 carry) | P0 |

---

## 8. 결론

본 SPEC 은 moai-studio 의 agentic essence 영역으로, 기존 3 crate (`moai-stream-json`, `moai-hook-http`, `moai-supervisor` stub) 위에 신규 `moai-studio-agent` crate + `moai-studio-ui/src/agent_ui/` 모듈을 얹는다.

핵심 결정:
- **A1 ring buffer 1000** (영속화 default)
- **B1 self-report** (cost source default)
- **C1 stdin envelope** (control IPC default)
- **D2 SSE** (hook ingestion default)

총 6 RG, 12 AC, 3 MS. SPEC-V3-004 선행, V3-009 와 병행, V3-006 와 선택적 통합.
