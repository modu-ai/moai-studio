---
spec_id: SPEC-V3-010
artifact: plan
version: 1.0.0
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
milestones: [MS-1, MS-2, MS-3]
language: ko
---

# SPEC-V3-010 Implementation Plan — Agent Progress Dashboard

본 plan 은 spec.md 의 RG-AD-1 ~ RG-AD-6 과 AC-AD-1 ~ AC-AD-12 를 3 milestone 으로 분할한다. priority 라벨은 사용하되 시간 추정은 사용하지 않는다 (CLAUDE.md / agent-common-protocol 의 Time Estimation HARD rule).

---

## 0. 사전 결정 (USER-DECISION default 가정)

| 게이트 | default | 영향 |
|--------|---------|------|
| AD-A (영속화) | A1 ring buffer 1000 | RG-AD-1 의 REQ-AD-004 |
| AD-B (cost source) | B1 self-report | RG-AD-3 의 REQ-AD-014/017 |
| AD-C (control IPC) | C1 stdin envelope | RG-AD-5 의 REQ-AD-025/026/027 |
| AD-D (hook ingestion) | D2 SSE | RG-AD-1 의 REQ-AD-003 |

사용자가 override 시 본 plan 의 milestone 분할을 재계산해야 한다.

---

## 1. Milestone 1 — Stream parser + 기본 timeline

### 1.1 목표

`moai-studio-agent` crate 의 ingestion path 를 동작시키고, `EventTimelineView` 가 ring buffer 의 events 를 60fps 로 렌더하도록 만든다. filter / cost / instructions / control / detail 은 MS-2/MS-3 에서 추가.

### 1.2 우선순위 작업 (Priority High)

**MS-1.A — Crate scaffold**

- `crates/moai-studio-agent/` workspace 멤버 추가
- `Cargo.toml` dependency: `moai-stream-json`, `moai-hook-http`, `uuid`, `chrono`, `serde`, `tokio` (async runtime), `gpui` (Render trait)
- `src/lib.rs` 모듈 선언 (`run`, `event`, `ring_buffer`, `cost`, `instructions`, `control` — 후 milestone 에서 일부 채움)

**MS-1.B — Domain types (RG-AD-1)**

- `AgentRunId(uuid::Uuid)` (REQ-AD-001) — single source of truth
- `AgentRun`, `AgentRunStatus` (research §5.1)
- `AgentEvent`, `AgentEventKind { Stream, Hook, Lifecycle, Unknown }` (REQ-AD-002/003/005)
- `EventKind::iter()` 동적 enumeration helper (REQ-AD-006)

**MS-1.C — Ring buffer (RG-AD-1)**

- `RingBuffer<AgentEvent>` capacity 1000 (REQ-AD-004)
- push / iter / freeze API
- overflow 시 oldest evict
- thread-safe (parking_lot::RwLock 또는 std::sync::Mutex)

**MS-1.D — Stream-json ingestion path (RG-AD-1)**

- `EventStreamSubscriber::from_subprocess(stdout: ChildStdout) -> impl Stream<Item = AgentEvent>`
- 1 line read → `moai_stream_json::Decoder::decode_line` → `AgentEventKind::Stream` (REQ-AD-002)
- error 시 `Unknown` fallback (REQ-AD-005)

**MS-1.E — SSE ingestion path (RG-AD-1)**

- `EventStreamSubscriber::from_sse(url: &str) -> impl Stream<Item = AgentEvent>`
- moai-hook-http `/events/sse` 가정 (USER-DECISION-AD-D default)
- 1 SSE message → `HookEventKind` 매핑 → `AgentEventKind::Hook` (REQ-AD-003)
- SSE endpoint 부재 시 graceful degradation: warning log + stream-json only ingestion (research §3 R1 의 D2 → polling fallback 은 MS-2 follow-up)

**MS-1.F — EventTimelineView 골격 (RG-AD-2 minimal)**

- `crates/moai-studio-ui/src/agent_ui/mod.rs` — 모듈 진입
- `timeline_view.rs` — `EventTimelineView` struct + `Render` impl
- ring buffer iter → reverse chronological list (REQ-AD-008)
- 16ms throttle window (REQ-AD-009) — `cx.notify()` debounce
- filter chip 은 MS-2 에서 추가

**MS-1.G — terminal/panes/tabs core 무변경 검증 (G8)**

- AC-AD-12 의 git diff 스크립트 작성
- CI hook 또는 사전 commit hook 으로 강제

### 1.3 우선순위 작업 (Priority Medium)

**MS-1.H — `AgentDashboardView` 컨테이너 골격**

- split layout 좌측: timeline / 우측: (placeholder for cost+instructions)
- MS-2/3 에서 placeholder 채움

**MS-1.I — Tests**

- `cargo test -p moai-studio-agent` — 단위 테스트
- AC-AD-1 (stream-json decode) / AC-AD-2 (SSE decode) / AC-AD-3 (60fps render bench) / AC-AD-12 (git diff)
- bench: `criterion` crate 으로 1000 events 의 throughput 측정 (NFR-AD-1)

### 1.4 MS-1 완료 조건 (Definition of Done)

- AC-AD-1, AC-AD-2, AC-AD-3, AC-AD-12 통과
- `cargo build --release` macOS arm64 + Ubuntu x86_64 OK
- `cargo clippy -- -D warnings` clean
- ring buffer push throughput ≥ 1000 events/sec (NFR-AD-1)
- timeline render p95 ≤ 16ms with 1000 events (NFR-AD-2)

### 1.5 MS-1 위험

- moai-hook-http SSE endpoint 미존재 가능성 → graceful degradation path 필수, 별 SPEC 으로 hook-http 측 enhance 권고 (본 SPEC scope 외)
- moai-stream-json 의 decoder API 가 line 단위가 아닌 chunk 단위라면 wrapper 추가 필요

---

## 2. Milestone 2 — Cost tracker + filter

### 2.1 목표

`CostPanelView` 와 EventTimelineView 의 filter chip 을 추가한다. RG-AD-3 전체 + RG-AD-2 의 filter 부분.

### 2.2 우선순위 작업 (Priority High)

**MS-2.A — Cost domain (RG-AD-3)**

- `cost.rs` — `CostSnapshot { input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, usd_total }` (research §5.2)
- `Usage::extract(StreamMessage) -> Option<CostSnapshot>` (REQ-AD-014)
- self-report only, 자체 계산 금지 (REQ-AD-017)

**MS-2.B — Cost aggregation**

- `CostAggregation { by_session, by_day, by_week }` (REQ-AD-016)
- `BTreeMap<NaiveDate, f64>` daily / `BTreeMap<(year, ISO week), f64>` weekly
- precision 4 decimal (NFR-AD-5)

**MS-2.C — CostPanelView (RG-AD-3)**

- `cost_panel_view.rs` — 3 metric (session / today / week) display (REQ-AD-015)
- `usage` 부재 session 의 warning badge "usage data unavailable" (REQ-AD-018)
- design token 재사용: `chart.{1..3}` for 3 metric

**MS-2.D — Timeline filter chip (RG-AD-2)**

- 2 group: stream messages / hook events (REQ-AD-011)
- chip toggle → filter state → render (REQ-AD-010)
- inactive chip kind 는 제거가 아닌 50% opacity dim

### 2.3 우선순위 작업 (Priority Medium)

**MS-2.E — Tests**

- AC-AD-4 (filter dim/full toggle)
- AC-AD-5 (session 누적 USD)
- AC-AD-6 (daily/weekly aggregation correctness)
- mock stream-json fixture 로 `usage` 시퀀스 검증

**MS-2.F — Persistence hook (forward compat for AD-A2 follow-up)**

- ring buffer 의 snapshot serialize 인터페이스 stub (`pub trait Persistable`)
- 본 SPEC v1.0.0 은 `NoOpPersister` 만, AD-A2 follow-up SPEC 에서 SQLite 구현체 추가

### 2.4 MS-2 완료 조건

- AC-AD-4, AC-AD-5, AC-AD-6 통과
- CostPanel 의 daily/weekly aggregation correctness unit test pass
- timeline filter UI usability 검증 (manual smoke)

### 2.5 MS-2 위험

- Claude / Anthropic API 의 `usage` schema 변동 → moai-stream-json 책임. 본 SPEC 의 `Usage::extract` 는 graceful `None` fallback.
- 사용자가 OpenAI / GLM 사용 시 `usage` schema 다름 → REQ-AD-018 의 warning badge 가 fallback. B2 self-calc 는 REJECT (N11).

---

## 3. Milestone 3 — Instructions graph + agent control + detail

### 3.1 목표

RG-AD-4 (instructions graph) + RG-AD-5 (control) + RG-AD-6 (event detail) 전체. dashboard 가 v1.0.0 spec 의 모든 view 를 갖춤.

### 3.2 우선순위 작업 (Priority High)

**MS-3.A — Instruction tree scanner (RG-AD-4)**

- `instructions.rs` — `InstructionNode`, `InstructionKind` (research §5.3)
- 6 layer scanner (managed policy / project / rules / user / local / memory + skills)
- priority 1~6 mapping per `.claude/rules/moai/workflow/moai-memory.md`
- `SessionStart` / `PreCompact` / `PrePromptInject` hook trigger rebuild (REQ-AD-019/020)

**MS-3.B — InstructionsGraphView (RG-AD-4)**

- `instructions_graph_view.rs` — indent layered tree render (REQ-AD-021)
- click → OS default editor open (REQ-AD-022) — `xdg-open` (Linux) / `open` (macOS)
- read-only enforcement (REQ-AD-023, N3)

**MS-3.C — Control envelope (RG-AD-5)**

- `control.rs` — stdin envelope writer with `MOAI-CTRL: ` prefix (USER-DECISION-AD-C C1)
- pause / resume / kill 3 action (REQ-AD-025/026/027)
- subprocess stdin handle injection from MS-1.D's `EventStreamSubscriber::from_subprocess`

**MS-3.D — AgentControlBar (RG-AD-5)**

- `control_bar.rs` — 3 button (pause/resume/kill) toggled per `AgentRunStatus` (REQ-AD-024)
- kill 버튼 클릭 시 GPUI native modal confirm (REQ-AD-027/029)
- status update on hook ack (REQ-AD-028)

**MS-3.E — EventDetailView (RG-AD-6)**

- `detail_view.rs` — JSON pretty-print (2-space indent, REQ-AD-030)
- collapse/expand disclosure triangles (REQ-AD-031)
- "Copy as JSON" 버튼 (REQ-AD-033)
- markdown payload 렌더 (REQ-AD-032) 는 SPEC-V3-006 통합 — **MS-3 default 비목표, follow-up**

### 3.3 우선순위 작업 (Priority Medium)

**MS-3.F — Tests**

- AC-AD-7 (6 layer tree rendering)
- AC-AD-8 (SessionStart/PreCompact rebuild)
- AC-AD-9 (pause envelope round-trip)
- AC-AD-10 (kill confirm + envelope)
- AC-AD-11 (JSON pretty-print + collapse)

**MS-3.G — Integration smoke test**

- end-to-end: V3-009 SpecListView "Run" click → V3-010 dashboard live update
- AgentRunId schema 동기화 검증 (V3-009 와 cross-link)

### 3.4 MS-3 완료 조건

- AC-AD-7 ~ AC-AD-11 통과
- AgentRunStatus 가 모든 hook event 에 대해 일관성 있게 갱신
- terminal/panes/tabs core git diff = 0 (NFR-AD-10, AC-AD-12 재검증)
- macOS 14+ / Ubuntu 22.04+ 동일 동작 (NFR-AD-8)

### 3.5 MS-3 위험

- stdin envelope 이 Claude Code session 의 prompt 로 해석될 가능성 → `MOAI-CTRL: ` prefix isolation, 그러나 v1.0.0 spike 에서 검증 필요. spike 결과에 따라 USER-DECISION-AD-C 재검토 가능.
- instruction tree rebuild 의 200ms p95 (NFR-AD-6) 는 6 layer × 평균 30 파일 스캔 가정. 사용자 환경에 따라 초과 가능 → 비동기 background scan + 부분 갱신 fallback.
- kill 의 process group SIGTERM 전파 동작이 OS 별 미묘하게 다름 → C1 stdin envelope default 라 본 SPEC 에서는 직접 SIGTERM 미사용. supervisor follow-up SPEC 의 책임.

---

## 4. 기술 접근 (Technical Approach)

### 4.1 Crate / 모듈 dependency direction

```
moai-studio-ui (binary host) 
   └── agent_ui/* (6 view)
          ↓ depends on
   moai-studio-agent (domain)
          ↓ depends on
   { moai-stream-json, moai-hook-http, (future) moai-supervisor }
```

본 SPEC 은 supervisor 의 trait stub (`AgentHandle`) 만 정의. 실 supervisor crate 는 별 SPEC.

### 4.2 비동기 모델

- ingestion: `tokio` async stream (subprocess stdout + SSE)
- UI thread: GPUI 의 single-threaded model. ingestion → channel → UI thread `cx.notify()`.
- channel: `tokio::sync::mpsc::UnboundedSender<AgentEvent>` (back-pressure 는 ring buffer eviction 에 위임).

### 4.3 에러 처리

- ingestion error: `tracing::warn!` + `AgentEventKind::Unknown` push
- ring buffer freeze 후 push 시도: `Result::Err` 반환, ingest task 종료
- control envelope write error: dashboard 에 toast notification

### 4.4 60fps throttle 구현

- `cx.notify()` 호출을 16ms window 로 coalesce
- 기존 GPUI `Subscription` 패턴 (SPEC-V3-004 가 확립한 pattern) 재사용
- burst event 가 16ms 내 다수 도착 시 마지막 1 회만 notify, ring buffer 는 이미 모두 push 됨

### 4.5 design token 재사용 (N8)

| 용도 | 토큰 |
|------|------|
| dashboard background | `app.background` |
| 각 view panel | `panel.background` |
| event 텍스트 | `text.primary` |
| Hook event status (success) | `status.success` |
| Hook event status (error) | `status.error` |
| Hook event status (warning) | `status.warning` |
| Hook event status (info) | `status.info` |
| Cost chart bars | `chart.1`, `chart.2`, `chart.3` |

---

## 5. 마일스톤별 RG / AC 매핑 요약

| MS | RG 커버 | AC 커버 |
|----|---------|---------|
| MS-1 | RG-AD-1 (full), RG-AD-2 (minimal, no filter) | AC-AD-1, AC-AD-2, AC-AD-3, AC-AD-12 |
| MS-2 | RG-AD-3 (full), RG-AD-2 (filter) | AC-AD-4, AC-AD-5, AC-AD-6 |
| MS-3 | RG-AD-4 (full), RG-AD-5 (full), RG-AD-6 (full) | AC-AD-7, AC-AD-8, AC-AD-9, AC-AD-10, AC-AD-11 |

---

## 6. 검증 / 품질 게이트

- **TRUST 5** (CLAUDE.md):
  - Tested: AC 12 종 모두 자동화. integration smoke MS-3 에서.
  - Readable: code_comments=ko (CLAUDE.local.md), 변수/함수명 영문.
  - Unified: `rustfmt + clippy -D warnings` 강제.
  - Secured: control envelope 의 prompt injection 회피 검증 (MS-3 spike).
  - Trackable: SPEC-V3-010 → AC-AD-* → 파일별 commit subject 의 Conventional Commits scope 일치.
- **CI / LSP gate** (constitution §6):
  - run phase: 0 errors / 0 type errors / 0 lint errors
  - sync phase: 0 errors / max 10 warnings

---

## 7. 단일 commit 정책 (본 SPEC authoring 단계)

본 plan / spec / research 작성은 단일 commit 에 포함:
```
docs(spec): SPEC-V3-010 Agent Progress Dashboard v1.0.0 (research/plan/spec)
```
sign-off: `🗿 MoAI <email@mo.ai.kr>`. push 또는 PR 은 본 작업에서 수행하지 않음 (사용자 명시 지시 부재).

---

## 8. Out-of-scope follow-up SPEC 후보

| 후보 SPEC | 사유 |
|-----------|------|
| SPEC-V3-Future-AgentDash-Persist | USER-DECISION-AD-A2 (SQLite 영속화) |
| SPEC-V3-Future-AgentDash-MarkdownDetail | REQ-AD-032 (V3-006 통합) |
| SPEC-V3-Future-MultiAgent | N5 (multi-agent dashboard) |
| SPEC-V3-Future-Supervisor | `AgentHandle` trait 의 실 구현 (kill 의 process group SIGTERM 등) |

---

Version: 1.0.0
Last Updated: 2026-04-25
Author: MoAI (manager-spec)
Language: ko
