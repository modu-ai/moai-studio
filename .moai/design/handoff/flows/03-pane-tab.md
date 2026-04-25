# User Flow — Pane & Tab Management

---
title: Pane Split and Tab Operations
version: 1.0.0
source: SPEC-V3-003
last_updated: 2026-04-25
---

## Pane Split (Cmd+\\)

### 초기 상태

```
┌──────────────────────────────────┐
│ [main.rs]                        │
├──────────────────────────────────┤
│                                   │
│ Single pane (full width)          │
│                                   │
└──────────────────────────────────┘
```

### 사용자 입력: Cmd+\\

### 결과: Horizontal Split

```
┌──────────────────────────────────┐
│ [main.rs]        [+]             │
├──────────────────────────────────┤
│ Left Pane        Right Pane      │
│ (code)     ▓▓▓▓  (empty)         │
│ (50:50)    ▓▓▓▓  (placeholder)   │
│ content    ▓▓▓▓                   │
│            ▓▓▓▓                   │
└──────────────────────────────────┘
```

- Divider: 4px primary.500 (hover)
- Animation: spring 200ms (split slide)
- Right pane: empty, "파일을 선택하세요" or terminal prompt

---

## Vertical Split (Cmd+Shift+\\)

### 초기

```
┌──────────────────────────────────┐
│ [main.rs]                        │
├──────────────────────────────────┤
│ Single pane (horizontal)          │
└──────────────────────────────────┘
```

### 결과: Vertical Split

```
┌──────────────────────────────────┐
│ [main.rs]        [+]             │
├──────────┬──────────────────────┤
│          │                       │
│ Top      │  Bottom (empty pane) │
│ (code)   │                       │
│          │                       │
│ ▓▓▓▓▓▓▓▓ │ placeholder          │
│          │                       │
└──────────┴──────────────────────┘
```

- Divider: 4px, 수직 orientation
- Top pane: 기존 content (code)
- Bottom pane: empty

---

## Divider Drag & Resize

### Hover

```
┌──────────────────────────────────┐
│ [main.rs]        [+]             │
├──────────────────────────────────┤
│ Left Pane  ▓▓▓▓▓▓▓▓▓  Right     │
│            ▓▓▓▓▓▓▓▓▓            │
│            ▓▓▓▓▓▓▓▓▓ (hover: blue)
│            ▓▓▓▓▓▓▓▓▓            │
└──────────────────────────────────┘
```

- Divider: primary.500 (blue) on hover
- Cursor: resize-column (↔️)

### Drag

```
User drags right:

Left: 40%  │ Right: 60%
           ↕ (drag)
           ↓
Left: 60%  │ Right: 40%
```

- 실시간 ratio 업데이트 (no throttle, instant)
- Smooth animation (20ms frame sync)
- Min constraint: 각 pane min 240px (clamped on drop)

### Drop

```
Final state:
┌──────────────────────────────────┐
│ [main.rs]        [test.rs]  [+]  │
├──────────────┬──────────────────┤
│ Left (60%)   │ Right (40%)      │
│ main.rs      │ test.rs content  │
│ content      │                   │
└──────────────┴──────────────────┘
```

---

## Tab 전환 (Cmd+1/2/3)

### 초기

```
Pane A (Code Viewer):
[main.rs] [lib.rs] [README.md]  ← main.rs active
```

### 사용자: Cmd+2

### 결과

```
[main.rs] [lib.rs] [README.md]
           ↑ now active (bold)

Content → lib.rs code 렌더
```

- Animation: content cross-fade 200ms easeOut
- Tab styling: inactive → active (font weight 400→600)

---

## New Tab (Cmd+T)

### 상태 A: 파일 선택 dialog

```
┌──────────────────────────────────┐
│ Choose file to open              │
├──────────────────────────────────┤
│ 📁 src/                          │
│  ├─ main.rs                      │
│  ├─ lib.rs        ← 선택         │
│  └─ tests.rs                     │
│                                   │
│ [Open] [Cancel]                  │
└──────────────────────────────────┘
```

### 상태 B: Tab 생성 & 콘텐츠 렌더

```
[main.rs] [lib.rs]  [+]  ← lib.rs 새 탭
           ↑ active (bold)

Content renders lib.rs
```

---

## Close Tab (Cmd+W)

### 상태 A: 모든 탭 저장됨

```
[main.rs] [lib.rs] [+]
           ← Cmd+W

Result:
[main.rs] [+]  ← lib.rs 닫힘
↑ auto-focus (이전 탭)
```

### 상태 B: Dirty (저장 안 됨)

```
[main.rs] ● [lib.rs]  ← main.rs dirty (white dot)
           Cmd+W

Confirm dialog:
┌──────────────────────────────────┐
│ Unsaved changes                  │
│                                   │
│ main.rs has unsaved changes       │
│                                   │
│ [Save] [Discard] [Cancel]        │
└──────────────────────────────────┘

[Save] → save & close
[Discard] → close without save
[Cancel] → keep open
```

---

## Pane 포커스 변경 (Cmd+}/Cmd+{)

### 초기

```
┌───────────────────┬──────────────┐
│ Pane A (focused)  │ Pane B       │
│ [main.rs]         │ [terminal]   │
│ ▓▓▓▓▓▓▓▓▓        │ ▓▓▓▓▓▓▓▓   │
│ ▓▓▓▓▓▓▓▓▓        │ ▓▓▓▓▓▓▓▓   │
│ (blue border)     │              │
└───────────────────┴──────────────┘
```

### 사용자: Cmd+}

### 결과: Pane B 포커스

```
┌───────────────────┬──────────────┐
│ Pane A            │ Pane B (focus)
│ [main.rs]         │ [terminal]   │
│ content           │ ▓▓▓▓▓▓▓▓   │
│                   │ ▓▓▓▓▓▓▓▓   │
│                   │ (blue border)
└───────────────────┴──────────────┘
```

- Focus ring: 2px primary.500 (new pane border)
- last_focused_pane: Pane B 의 마지막 active tab 복원

---

## Last-Focused Pane Restoration

### 사용자 시나리오

1. **Pane A (main.rs active)** → focus

```
[main.rs] [lib.rs]
 ▓▓▓▓▓▓▓▓  (active)
```

2. **Switch to Pane B** (Cmd+})

```
Switch to [test.rs] (most recent in Pane B)
[test.rs] shown
```

3. **Back to Pane A** (Cmd+{)

```
main.rs 자동 복원 (last viewed in A)
NOT lib.rs (other tab in A)
```

- Invariant: REQ-P-023
- Mechanism: pane 별 `last_active_tab_id` 저장

---

## Persistence (Save & Restore)

### Save (앱 종료)

```json
{
  "panes": {
    "type": "split",
    "direction": "horizontal",
    "ratio": 0.6,
    "left": {
      "type": "leaf",
      "active_tab_index": 0,
      "tabs": [
        {"path": "src/main.rs", "scroll": 1234},
        {"path": "src/lib.rs", "scroll": 0}
      ]
    },
    "right": {
      "type": "leaf",
      "active_tab_index": 0,
      "tabs": [
        {"path": "tests/integration.rs", "scroll": 0}
      ]
    }
  }
}
```

### Restore (앱 재시작)

1. Load JSON
2. Rebuild pane tree
3. Restore scroll positions
4. Set last_focused_pane
5. Render all content

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — pane/tab 관리 흐름

