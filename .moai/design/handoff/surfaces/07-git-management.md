# Git Management Surface

---
title: Git Status, Diff, Commit UI
version: 1.0.0
source: SPEC-V3-008
last_updated: 2026-04-25
---

## 개요

**Git Management** 는 git status, diff, commit, branch 관리 UI 를 제공한다. 상태 패널 + diff 뷰어 + commit composer 로 구성.

상태: 설계 완료 (구현 미완료, SPEC-V3-008)

---

## 2-Pane 레이아웃

```
┌──────────────────────────────────────────┐
│ Git Management                            │
├──────────────────────────────────────────┤
│ [Branch] [Status] [Log]  [Stash]         │
├─────────────────┬──────────────────────┤
│                 │                      │
│ File Status     │  Diff Viewer        │
│ Panel           │  (side-by-side)    │
│ (staged/        │                      │
│  unstaged/      │                      │
│  untracked)     │                      │
│                 │                      │
└─────────────────┴──────────────────────┘
```

---

## Status Panel (좌측)

### Staged Files

```
📁 Staged (2)
 ├─ ✓ main.rs
 └─ ✓ lib.rs
```

- 아이콘: ✓ (초록)
- 색: success.green
- 클릭: 파일 diff 보기
- 우클릭: unstage, revert, delete

### Unstaged Files

```
📁 Unstaged (3)
 ├─ M src/main.rs        (수정)
 ├─ M tests/lib.rs       (수정)
 └─ A new_file.rs        (추가)
```

- M 아이콘: warning.yellow
- A 아이콘: success.green
- D 아이콘: error.red
- 우클릭: stage, discard, add to gitignore

### Untracked Files

```
📁 Untracked (1)
 └─ ? .env
```

- ? 아이콘: neutral.400
- 우클릭: add, git ignore

---

## Diff Viewer (우측)

### Side-by-Side

```
┌──────────────────┬──────────────────┐
│ Before (original)│ After (modified) │
├──────────────────┼──────────────────┤
│ 1 fn main() {    │ 1 fn main() {    │
│ 2   let x = 5;   │ 2   let x = 10;  │
│ 3   println!...  │ 3   println!...  │
│ 4 }              │ 4 }              │
└──────────────────┴──────────────────┘
```

- 좌: before (neutral.900 bg)
- 우: after (neutral.850 bg)
- 추가 라인: success.green bg
- 삭제 라인: error.red bg
- 변경 라인: warning.yellow bg

### Unified 모드

```
@@ -1,4 +1,4 @@
 fn main() {
-  let x = 5;
+  let x = 10;
   println!...
 }
```

### Toggle Button

좌측 상단: "Side-by-Side" / "Unified" toggle

---

## Commit Composer

### 하단 패널 (커밋 준비)

```
┌──────────────────────────────────────┐
│ Commit Message                        │
├──────────────────────────────────────┤
│ [Title line (50 chars max)]           │
│                                       │
│ [Detailed description...             │
│  Multiple lines allowed              │
│  Each line < 72 chars recommended]   │
│                                       │
│ [✓ Verify locally] [AI Suggest]      │
│ [Commit] [Cancel]                    │
└──────────────────────────────────────┘
```

- Font: Pretendard 14px
- 첫 라인: 50자 제한 (guide line)
- 본문: 72자 라인 (soft wrap)
- 색: neutral.50 text on neutral.800 bg

### AI Suggest 버튼

```
[🤖 AI Suggest]

생성된 메시지:
"feat(main): Change x value from 5 to 10
- Update main.rs x variable
- Reason: performance improvement"

[Use] [Edit] [Cancel]
```

---

## Branch Switcher

### Dropdown

```
▼ Current: main

Recent:
 └─ main (active)
 └─ develop
 └─ feature/auth

All branches:
 └─ main
 └─ develop
 └─ release/v0.1.0
 └─ feature/auth (12 branches)

[+ Create new]
```

- 검색: fuzzy match on branch name
- 우클릭: delete, rename, merge to...

---

## Log View (advanced)

### Graph + Message

```
*---------* main
|\        |
| *---*   | feature/auth (4 commits behind)
|/    \  |
*      * | develop
        \|
         * SPEC-V3-003 complete
         |
         * Added test cases
         |
         * Initial commit
```

- 커밋 point: 작은 원 (16px)
- 색: 브랜치별 (primary, secondary, etc.)
- 메시지: neutral.50 12px
- 시간: neutral.400 10px (xs)
- Author: neutral.300 10px

---

## Merge Conflict Resolver (advanced)

### 3-way Diff

```
┌─────────────┬──────────────┬──────────────┐
│ Base        │ Ours (main)  │ Theirs       │
├─────────────┼──────────────┼──────────────┤
│ x = 5       │ x = 10       │ x = 15       │
│             │ [Accept]     │ [Accept]     │
│             │ [Both]       │              │
└─────────────┴──────────────┴──────────────┘

Result: [Choose manually below]
```

- 선택: Accept ours / Accept theirs / Both / Manual
- Manual mode: inline edit (mergable unified view)

---

## Stash Management

### Stash List

```
[Stash]

📁 Stash (2)
 ├─ WIP on main 2h ago
 │  ├─ src/main.rs (M)
 │  └─ tests/lib.rs (M)
 │  [Apply] [Drop]
 └─ Working directory 8h ago
    ├─ .env (?)
    [Apply] [Drop]
```

---

## 상태

### Clean (nothing staged)
```
✓ Working tree clean
  All changes committed
```

### Dirty (uncommitted)
```
⚠ 2 staged, 3 unstaged
  [Commit] to save changes
```

### Merging
```
🔀 Merge in progress
  Resolve 1 conflict
  [Resolve] [Abort]
```

---

## 접근성

- Tab order: Status → Diff → Composer
- Keyboard: arrow navigate files, Enter to view diff
- Color contrast: ≥ 4.5:1
- Screen reader: semantic file list

---

**마지막 수정**: 2026-04-25  
**상태**: 설계 완료 — SPEC-V3-008 plan

