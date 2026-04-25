# File Explorer Surface

---
title: File Tree with Git Status
version: 1.0.0
source: SPEC-V3-005
last_updated: 2026-04-25
---

## 개요

**File Explorer** 는 프로젝트 파일 시스템을 hierarchical tree 로 표시한다. Git status badge, fuzzy search, drag-and-drop, context menu 를 지원한다.

상태: 설계 완료 (구현 미완료, SPEC-V3-005 계획)

---

## 트리 구조

### FsNode 데이터 모델

```
FsNode {
  path: "/path/to/file",
  name: "main.rs",
  kind: Directory | File | Symlink,
  children_state: NotLoaded | Loading | Loaded | Error,
  git_status: Untracked | Modified | Added | Deleted | Ignored,
  icon: "⚙" (file type),
  expanded: bool,
}
```

### 시각적 트리 렌더

```
📁 src/                  ← Folder icon, expandable
 ├─ 📄 main.rs    M     ← Modified (M badge, orange)
 ├─ 📄 lib.rs     ·     ← Unchanged (· dot, gray)
 ├─ 📁 services/  ▽     ← Expanded, children shown
 │  ├─ 📄 auth.rs
 │  └─ 📄 db.rs
 └─ 📁 utils/     ▷     ← Collapsed, children hidden (▷)

🔄 tests/               ← Loading state (spinner)
 
⚠ .env                  ← Error (cannot read)

? .DS_Store             ← Untracked (? icon)
```

---

## 아이콘 & 타입 매핑

### File Type Icons (16px, Phosphor or Lucide)

| 확장자 | Icon | 색상 |
|--------|------|------|
| .rs | ⚙ | primary.500 (orange) |
| .go | 🐹 | brand.secondary (orange) |
| .py | 🐍 | blue |
| .ts / .tsx | 💙 | primary.500 (blue) |
| .js / .jsx | 💛 | warning (yellow) |
| .md | 📝 | neutral.300 |
| .json | `{}` | secondary (gray) |
| .toml | 📋 | secondary |
| (folder) | 📁 | primary.500 (closed), primary.500 (open) |
| (unknown) | 📄 | neutral.300 |

### ChildState 표시

| 상태 | 아이콘 | 동작 |
|------|--------|------|
| NotLoaded | ▷ (right chevron) | 클릭 시 children load |
| Loading | ⟳ (spinner) | 로딩 중, 비활성 |
| Loaded | ▽ (down chevron) | expanded, 수정 가능 |
| Error | ⚠ | 클릭 시 retry, tooltip 에 error msg |

---

## Git Status Badge

### Badge 표시

| Status | Icon/Color | 의미 |
|--------|-----------|------|
| M (Modified) | 주황 dot · | 변경됨 |
| A (Added) | 초록 dot + | 새로 추가 |
| D (Deleted) | 빨강 dot - | 삭제됨 |
| U (Unmerged) | 보라 dot ⚡ | merge conflict |
| ? (Untracked) | 회색 ? | git 미추적 |
| (ignored) | 어두운 회색 ∞ | .gitignore 매칭 |

위치: 파일명 우측, 공간 있을 때

```
📄 main.rs     M  ← orange dot
📄 new_file.rs +  ← green dot
```

---

## 인터랙션

### 마우스
- **좌클릭**: 파일 열기 (새 탭) 또는 폴더 expand/collapse
- **더블클릭**: 파일 열기 (강제)
- **우클릭**: context menu (copy path, open in terminal, delete, rename, etc.)
- **드래그**: 파일 drag-and-drop (다른 폴더로 move 또는 재정렬)

### 키보드
- **Enter**: 선택 파일 열기
- **Space**: 폴더 expand/collapse
- **Cmd+D**: 현재 파일 삭제
- **Cmd+R**: 현재 파일 rename (inline edit)
- **Arrow Up/Down**: 트리 위/아래 이동
- **Arrow Left**: 폴더 collapse
- **Arrow Right**: 폴더 expand
- **Cmd+P**: search (fuzzy matching)

### 검색바 (Cmd+P)

```
┌─────────────────────────────────┐
│ 🔍 [_______________________]     │  ← 검색어 입력
├─────────────────────────────────┤
│ main.rs                          │
│ main.go                          │
│ main.py                          │
│ (matching files, fuzzy)          │
└─────────────────────────────────┘
```

- 실시간 fuzzy match
- 결과: 파일명 + 경로
- 선택 시: 새 탭으로 열기
- Esc: 닫기

---

## Context Menu (우클릭)

```
┌────────────────────────────┐
│ Open            Cmd+O      │
│ Open in New Tab Cmd+T      │
│ ────────────────────────   │
│ New File                   │
│ New Folder                 │
│ ────────────────────────   │
│ Copy Path       Cmd+Shift+C│
│ Open in Terminal Cmd+Shift+T
│ ────────────────────────   │
│ Rename          Cmd+R      │
│ Delete          Cmd+D      │
│ ────────────────────────   │
│ Git Ignore (if ?)          │
│ Revert Changes (if M)      │
│ ────────────────────────   │
│ Properties              →  │
└────────────────────────────┘
```

---

## 상태

### Empty State
```
📁 No workspace open
  
[+ Open Workspace]
```

### Loading State
```
⟳ Scanning files...

(spinner animation, 120ms fast)
```

### Populated State
```
📁 src/
 ├─ main.rs    M
 ├─ lib.rs
 └─ tests.rs
 
📁 examples/
```

### Error State
```
⚠ Failed to scan directory
  
[Retry]
```

---

## 디자인 토큰

| 항목 | 값 |
|------|-----|
| Indent per level | 16px (spacing.4) |
| Icon size | 16px |
| Row height | 24px (tight) |
| Font size | 14px (base) |
| Font family | Pretendard / Inter |
| Hover bg | neutral.800 (dark), neutral.100 (light) |
| Selected bg | primary.500 @ 15% alpha |
| Text color | neutral.50 (dark), neutral.950 (light) |
| Secondary text | neutral.300 (dark), neutral.700 (light) |

---

## Drag & Drop

### Drag Highlight

```
📁 src/                    ← hover target (highlight)
 ├─ 📄 main.rs   (dragging)
 └─ 📄 lib.rs
```

- Drag source: opacity 0.5 (faded)
- Drop target: primary.500 2px border (dashed)
- Feedback: "Move to folder?" or "Copy?" (context)

### Drop Validation

- 파일 간 순서 변경 (reorder, unsupported 면 visual feedback)
- 다른 폴더로 move (git mv 호출)
- 폴더 drag X (invalid, visual feedback)

---

## Accessibility

- **Keyboard-only navigation**: arrows + enter 로 100% 조작 가능
- **Focus ring**: 5px primary.500
- **Contrast**: text on hover bg ≥ 4.5:1
- **Screen reader**: semantic tree structure (ARIA roles)

---

**마지막 수정**: 2026-04-25  
**상태**: 설계 완료 — SPEC-V3-005 plan

