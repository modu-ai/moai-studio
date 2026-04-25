# Agent Dashboard Surface

---
title: Agent Progress & Cost Tracking
version: 1.0.0
source: SPEC-V3-010
last_updated: 2026-04-25
---

## 개요

**Agent Dashboard** 는 Claude Code 에이전트 실행 진행 상황을 시각화한다. Hook event timeline, cost tracking, instructions graph, agent control 을 제공한다.

상태: 설계 완료 (구현 미완료, SPEC-V3-010)

---

## 3-Pane 레이아웃

```
┌──────────────────────────────────────────────────────┐
│ Agent Dashboard                                       │
├──────────────────────────────────────────────────────┤
│ [Filter]  [Timeline] [Cost] [Instructions] [Control] │
├──────────┬──────────────────┬───────────────────────┤
│          │                  │                        │
│ Filters  │   Event          │   Instructions        │
│          │   Timeline       │   Graph               │
│          │                  │                        │
│ Stream   │                  │                        │
│ - JSON   │                  │                        │
│ - Hook   │                  │                        │
│ - Tool   │                  │                        │
│ - Result │                  │                        │
│          │                  │                        │
└──────────┴──────────────────┴───────────────────────┘
```

---

## 1. Event Timeline (중앙)

### 이벤트 타입 (27개)

| Hook | Color | Icon | 설명 |
|------|-------|------|------|
| session_start | info | ⚙ | Claude subprocess 시작 |
| session_end | neutral | ✓ | 세션 종료 |
| message_start | primary | 💬 | 메시지 수신 |
| message_delta | primary | ↓ | 메시지 스트리밍 |
| tool_use_start | secondary | 🔧 | Tool 호출 시작 |
| tool_use_result | success | ✓ | Tool 결과 반환 |
| error | error | ❌ | 에러 발생 |
| (20+ more) | varied | — | 기타 hook event |

### 타임라인 렌더

```
[13:45:30] ⚙ session_start
[13:45:31] 💬 message_start (user: "run SPEC-...")
[13:45:32] ↓ message_delta (chunk 1)
[13:45:33] ↓ message_delta (chunk 2)
[13:45:34] 🔧 tool_use_start (Bash)
[13:45:35] ✓ tool_use_result (exit 0)
[13:45:36] ↓ message_delta (final)
[13:45:37] ✓ message_end

▼ [클릭해서 drill down]
```

### 스타일

- 각 event: 1 라인
- Timestamp: neutral.400 12px (sm)
- Icon: 16px, 색별 (primary/secondary/success/error)
- Event name: neutral.50 14px (base)
- Gap: 4px
- 호버: 배경 highlight (neutral.800)
- 클릭: detail panel 열기

---

## 2. Filter Chip (좌상단)

```
[✓] stream-json
[✓] hook
[✓] Tool
[  ] Result  ← 체크 해제 시 필터링
[✓] Message
```

- 배경: primary.500 (selected), neutral.700 (unselected)
- 텍스트: neutral.0 (selected), neutral.50 (unselected)
- 클릭: toggle

---

## 3. Cost Panel (상단 우측)

### 비용 요약

```
┌──────────────────────────┐
│ Cost This Session        │
│ $0.042 (Opus 4.7)        │
│                          │
│ Daily: $1.23             │
│ Weekly: $8.47            │
└──────────────────────────┘
```

- 배경: neutral.800
- 테두리: 1px primary.500
- Radius: 8px (lg)
- Font: 12px (sm) secondary text

### Chart (Daily / Weekly)

```
┌─────────────────────────┐
│ Daily Cost Trend        │
│ $2.0 |      ╱╲          │
│ $1.5 |  ╱╲  ╱  ╲        │
│ $1.0 |╱╱╲╱╱    ╲       │
│ $0.5 |           ╲      │
│      └─────────────────│
│ Mon Tue Wed Thu Fri Sat│
└─────────────────────────┘
```

- Chart height: 180px
- Bar/line color: chart.1 ~ chart.5 (cycling)
- Axis labels: 12px (sm)
- Grid lines: subtle (neutral.700)

---

## 4. Instructions Graph (우측)

### 트리 구조

```
┌─────────────────────────────────┐
│ Instructions Loaded (89.2 KB)   │
├─────────────────────────────────┤
│ ├─ CLAUDE.md (62.4 KB)          │
│ │  ├─ moai-constitution.md      │
│ │  └─ agent-common-protocol.md  │
│ ├─ Skill tree (16.8 KB)         │
│ │  ├─ moai-foundation-core      │
│ │  └─ moai-workflow-project     │
│ └─ Memory (10 KB)               │
│    ├─ MEMORY.md                 │
│    ├─ user_profile.md           │
│    └─ lessons.md                │
└─────────────────────────────────┘
```

- 노드: clickable
- 호버: 배경 highlight + tooltip (reason)
- 색: 로드 방식별 (session_start=blue, include=green, nested=orange)
- Indent: 16px per level

### Hover Tooltip

```
moai-foundation-core
Reason: session_start
Loaded: 2026-04-25 13:45:30
Size: 12.4 KB
Status: ✓ loaded
```

---

## 5. Control Bar (우측 하단)

```
┌──────────────────────────┐
│ [⏸ Pause] [▶ Resume]    │
│ [⏹ Kill]   [↻ Restart]  │
└──────────────────────────┘
```

- 버튼: secondary style (outlined)
- 비활성 상태: 회색, disabled cursor

### Confirm Dialog (Kill 클릭)

```
┌──────────────────────────┐
│ Kill agent?              │
│                          │
│ 진행 중인 작업이 중단됨  │
│                          │
│ [Cancel] [Kill]          │
└──────────────────────────┘
```

---

## 상태 시각화

### Running 상태

```
⚙ Agent Running...
  step 1 of 3: Planning
  
[⏸ Pause] [⏹ Kill]
```

- Status: "Running", "Paused", "Completed", "Failed"
- Progress: "step N of M"
- 색: secondary.500 (running)

### Idle 상태

```
✓ Agent Idle
  
(last run: 2 hours ago)

[Start new session...]
```

---

## Event Detail Drilldown

클릭 시 expand:

```
[13:45:34] 🔧 tool_use_start (Bash)
  ▼ [Details]
  
  Command: cargo test --workspace
  Duration: 1.2s
  Exit code: 0
  Output: 
    running 103 tests
    test result: ok
```

- 배경: neutral.750 (dark), neutral.100 (light)
- 코드블록: JetBrains Mono 12px

---

## JSON Inspector (선택사항)

Event 클릭 시 우측 패널:

```
{
  "type": "tool_use_result",
  "tool_name": "Bash",
  "exit_code": 0,
  "output": "test results...",
  "duration_ms": 1234
}
```

- 문법 강조
- Collapse/expand 지원
- Copy JSON 버튼

---

## 성능 특성

### Throttling
- Event rate: 16ms throttle (60fps)
- Timeline update: burst 가능, but 16ms window 내 한 번만 render
- Chart update: 100ms debounce

### Rendering
- Virtual scroll: 1000+ events
- Chart: canvas-based (smooth 60fps)

---

## 접근성

- **Tab order**: Filter → Timeline → Cost → Instructions → Control
- **Keyboard nav**: arrows 로 event traverse
- **Color contrast**: 모든 text ≥ 4.5:1
- **Screen reader**: semantic heading & list structure

---

**마지막 수정**: 2026-04-25  
**상태**: 설계 완료 — SPEC-V3-010 plan

