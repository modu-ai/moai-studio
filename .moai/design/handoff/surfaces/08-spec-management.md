# SPEC Management Surface

---
title: SPEC Tracking & Kanban Board
version: 1.0.0
source: SPEC-V3-009
last_updated: 2026-04-25
---

## 개요

**SPEC Management** 는 `.moai/specs/` 를 시각화한다. SPEC list, detail view, Kanban board, AC state tracker 를 제공.

상태: 설계 완료 (구현 미완료, SPEC-V3-009)

---

## Tab 네비게이션

```
┌────────────────────────────────────────┐
│ SPEC Management  [List] [Detail] [Board]
├────────────────────────────────────────┤
│ ... content changes by tab ...         │
└────────────────────────────────────────┘
```

---

## Tab 1: List View

### SPEC Tree

```
📁 SPEC-V3 (15)
 ├─ ✓ SPEC-V3-001 GPUI Scaffold
 ├─ ✓ SPEC-V3-002 Terminal Core
 ├─ ✓ SPEC-V3-003 Panes + Tabs
 ├─ 🟡 SPEC-V3-004 Render Layer (draft)
 ├─ 📅 SPEC-V3-005 File Explorer
 ├─ 📅 SPEC-V3-006 Markdown/Code Viewer
 └─ 🔵 SPEC-V3-010 Agent Dashboard

📁 SPEC-AUTH (8)
 ├─ 📅 SPEC-AUTH-001 JWT Implementation
 └─ (7 more)
```

### 상태 표시

| 아이콘 | 상태 | 색 |
|--------|------|-----|
| ✓ | Complete (AC GREEN) | success.green |
| 🟡 | Draft | warning.yellow |
| 📅 | Planned (Next) | info.blue |
| 🔵 | In Progress | primary.blue |
| ⚠️ | Blocked | error.red |

### Folder 정렬

- 도메인별 (V3, AUTH, etc.)
- Alphabetical within domain

---

## Tab 2: Detail View

### 헤더

```
SPEC-V3-003 — Panes + Tabs (3 Milestones × 29 AC)

Status: ✓ Complete (29/29 AC GREEN)
Priority: P1 (High)
Branch: feature/SPEC-V3-003-ms3
```

- Status badge: 색별 (green/yellow/blue/red)
- AC progress: 진행률 bar (29/29 = 100%)
- Branch link: 클릭 시 git checkout

### 탭

```
[spec.md] [plan.md] [contract.md] [progress.md]
```

#### spec.md

```
## Requirements (EARS format)

Ubiquitous:
- PaneTree 는 binary tree 구조를 지원한다
- Divider 는 마우스 drag 로 크기 조정 가능하다

Event-Driven:
- When user presses Cmd+\\, split current pane horizontally

...
```

#### plan.md

```
## Implementation Plan

### Milestone 1: Core PaneTree
- Subtask 1: Implement binary tree data structure
- Subtask 2: Render split dividers
- Estimated: 2 days

...
```

#### contract.md

```
## Sprint Contract (Rev 10)

Acceptance Criteria:
1. PaneTree 생성 시 min size 240×120 유지
2. Divider drag 시 ratio clamp 0.3~0.7
3. Persistence save/load 테스트 ≥ 85%

Test Scenarios:
- [x] Test 1: Split horizontal
- [ ] Test 2: Split vertical
- [ ] Test 3: Restore on app reopen

...
```

#### progress.md

```
## Current Progress

Completed:
- [x] Core PaneTree implementation (2026-04-20)
- [x] Divider drag interaction (2026-04-21)
- [x] Tab container integration (2026-04-23)

In Progress:
- [ ] Persistence layer

...
```

---

## Tab 3: Kanban Board

### 컬럼 구조

```
┌─────────────┬─────────────┬──────────────┬────────┐
│   Backlog   │   TODO      │  In Progress │  Done  │
├─────────────┼─────────────┼──────────────┼────────┤
│             │             │              │        │
│ SPEC-V3-004 │ SPEC-V3-006 │ SPEC-V3-007 │SPEC-V3-│
│ Render      │ Markdown    │ Web Browser │ 001 ✓  │
│ [P1]        │ [P1]        │ [P2]        │        │
│             │             │              │SPEC-V3-│
│ SPEC-V3-008 │ SPEC-V3-008 │              │ 002 ✓  │
│ Git         │ Git         │              │        │
│ [P1]        │ [P1]        │              │SPEC-V3-│
│             │             │              │ 003 ✓  │
└─────────────┴─────────────┴──────────────┴────────┘
```

### Card Design

```
┌──────────────────────┐
│ SPEC-V3-006          │
│ Markdown/Code        │  ← Title
│ Viewer               │
│                      │
│ [P1] [29 AC]         │  ← Badge
│                      │
│ Assigned: @user      │  ← Metadata
│ Due: 2026-05-10      │
│                      │
│ AC: 15/29 ✓          │  ← Progress
└──────────────────────┘
```

### 드래그 & 드롭

- Card 선택 및 drag → 다른 칼럼으로 move
- Drag animation: spring 200ms
- Drop feedback: target column highlight

### Filter & Search

```
[Status: All] [Priority: P1+] [Search: ___________]

[Apply] [Reset]
```

---

## AC State Tracker (우측 사이드패널)

### State 매트릭스

```
AC-P-001 ✓ (GREEN)
  When user splits pane, min size 240×120 maintained
  ✓ Verified: 2026-04-21

AC-P-002 🟡 (PARTIAL)
  Divider ratio clamp 0.3~0.7
  ✓ Implemented
  ✗ Not tested on Windows
  ↗ Deferred to MS-3

AC-P-003 ✗ (FAIL)
  Persistence roundtrip in < 100ms
  Result: 250ms on large workspace (>50 panes)
  → Optimize or defer?
```

### Color Coding

| 상태 | 색 | 의미 |
|------|-----|------|
| ✓ | GREEN | 완료, 검증됨 |
| 🟡 | YELLOW | 부분 구현 (일부 실패) |
| ✗ | RED | 실패, 요청됨 |
| 🔵 | BLUE | 연기됨 (deferred) |
| ⏳ | GRAY | 미처리 (pending) |

---

## Slash Command Bar (좌상단)

```
[/moai plan] [/moai run] [/moai sync] [+ More]
```

- 각 버튼: 해당 command 1-click 실행
- Tooltip: "Run /moai plan SPEC-V3-006"

---

## 상태

### Empty (no SPEC)
```
"SPEC을 생성하세요"
[+ New SPEC]
```

### Loading
```
⟳ SPEC 목록 로드 중...
```

### Populated
```
SPEC-V3-003 ✓
SPEC-V3-004 📅
(list 또는 board view)
```

---

## 접근성

- Kanban card: keyboard navigate (arrow keys)
- Tab order: Filter → Search → Board/List
- Color contrast: ≥ 4.5:1

---

**마지막 수정**: 2026-04-25  
**상태**: 설계 완료 — SPEC-V3-009 plan

