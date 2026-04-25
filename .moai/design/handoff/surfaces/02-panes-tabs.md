# Panes + Tabs Surface

---
title: Multi-Pane Layout with Tab Management
version: 1.0.0
source: SPEC-V3-003
last_updated: 2026-04-25
---

## 개요

**Panes + Tabs** 는 multi-window 코드 에디터의 핵심이다. Binary tree 구조의 PaneTree 로 임의 분할이 가능하며, 각 pane 은 multiple tabs 를 가질 수 있다. 사용자는 마우스 drag 또는 Cmd 단축키로 조작한다.

상태: **SPEC-V3-003 완료** (MS-1 pane tree, MS-2 tab container + keybindings, MS-3 persistence)

---

## 아키텍처

### PaneTree (이진 트리)

```
Window
 └─ PaneTree (root node)
     ├─ Leaf {TabContainer}
     │  ├─ Tab "main.rs" (active)
     │  ├─ Tab "style.css"
     │  └─ Tab "README.md"
     │
     └─ SplitNode (horizontal split)
         ├─ Leaf {TabContainer}
         │  └─ Tab "tests.rs" (active)
         │
         └─ SplitNode (vertical split)
             ├─ Leaf {TabContainer}
             │  └─ Tab "Terminal"
             │
             └─ Leaf {TabContainer}
                 └─ Tab "Markdown"
```

### PaneConstraints

- **Min pane size**: 240px (width) × 120px (height)
- **Divider thickness**: 4px
- **Min divider ratio**: 0.3 ~ 0.7 (한쪽이 30% 이상 필요)

---

## Tab Bar (각 Pane 상단)

```
┌──────────────────────────────────────────────────────────┐
│ [main.rs] [style.css] [README.md]  [+] [×]               │
│ ▔▔▔▔▔▔▔▔                                                  │
│ (active tab 아래 underline, primary.500)                │
└──────────────────────────────────────────────────────────┘
```

### Tab Styling

| 상태 | 폰트 | 배경 | 설명 |
|------|------|------|------|
| Active | 600 (semibold) | neutral.800 (dark) | 현재 열린 탭 |
| Inactive | 400 (regular) | neutral.900 (dark) | 다른 탭들 |
| Hover | 500 (medium) | neutral.750 (dark) | 마우스 over |
| Dirty | 600 + "●" dot | neutral.800 | 저장 안 됨 (white dot) |
| Focused | focus ring 5px | — | keyboard focus |

### Tab Layout

- **Height**: 32px
- **Padding-x**: 12px
- **Padding-y**: 4px (center align)
- **Radius**: 4px (sm)
- **Icon size**: 16px (file type icon)
- **Max width**: 240px (overflow = truncate + tooltip)
- **Gap between tabs**: 0 (seamless)

### Close Button

- 호버 시 나타남 (× icon, 16px)
- 클릭 시: 탭 닫기
- Dirty 파일 시: "●" 노란 dot (저장 안 됨 경고)
- Cmd+W 와 동등

### New Tab Button

- 탭 바 우측 끝, "+" icon (16px)
- Cmd+T 와 동등
- 클릭: 파일 선택 dialog

---

## Pane 포커스 & 시각화

### Focus Indicator

```
┌─────────────────────────┐
│ ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔ │ ← 파란 2px 테두리
│ [Tab Bar]               │
│ ┌──────────────────────┤
│ │                      │
│ │  Content Area        │
│ │  (focused pane)      │
│ │                      │
│ └──────────────────────┘
```

- **Focus ring**: 2px primary.500, 파 바 상단만
- **Depth**: focused pane 은 약간 더 밝은 배경 (shadow 추가)

### last_focused_pane 복원

사용자가 여러 pane 을 toggle 하다가 특정 pane 으로 돌아갈 때, 그 pane 이 **마지막 활성 탭** 을 자동으로 표시한다. REQ-P-023 invariant.

예:
1. Pane A (main.rs 활성) → click
2. Pane B (test.rs 활성) → click
3. Pane A (다시 클릭) → **main.rs 자동 표시됨** (not first tab)

---

## Divider & Resize

### Visual

```
┌────────────────┬────────────────┐
│    Pane A      │    Pane B      │
│                │                │
├────────────────▓────────────────┤  ← divider (4px, hover=blue)
│                │                │
│                │                │
└────────────────┴────────────────┘
```

### Drag Behavior

- Divider 색: neutral.700 (default), primary.500 (hover)
- Divider width: 4px (hit area = 6px, cursor 도 40px+ 우측 범위에서 activate)
- Drag smooth: spring easing 200ms
- Min constraint: 각 pane 최소 240×120 px 유지
- Ratio clamp: 분할 비율 min 0.3, max 0.7 (한쪽 minimum 30%)

---

## Keybindings (SPEC-V3-003 MS-2)

| 키 | 동작 | 설명 |
|----|------|------|
| **Cmd+T** | New tab | 파일 선택 dialog |
| **Cmd+W** | Close tab | 현재 탭 닫기 (dirty면 confirm) |
| **Cmd+1~9** | Select tab N | 위치별 탭 선택 (1번째, 2번째, ...) |
| **Cmd+\\** | Split H | 현재 pane 수평 분할 |
| **Cmd+Shift+\\** | Split V | 현재 pane 수직 분할 |
| **Cmd+}** | Focus right | 우측 pane 포커스 |
| **Cmd+{** | Focus left | 좌측 pane 포커스 |
| **Cmd+↓** | Focus below | 아래 pane 포커스 |
| **Cmd+↑** | Focus above | 위 pane 포커스 |
| **Cmd+Shift+]** | Next tab | 다음 탭 |
| **Cmd+Shift+[** | Previous tab | 이전 탭 |

---

## 상태 시각화

### State: Empty (파일 없음)
```
┌──────────────────────────────────┐
│ [+]                              │
├──────────────────────────────────┤
│                                   │
│   "파일을 선택하세요"              │
│   [+ New File]                    │
│                                   │
└──────────────────────────────────┘
```

### State: Populated (정상)
```
┌──────────────────────────────────┐
│ [main.rs] [style.css] [README]  [+]
├──────────────────────────────────┤
│ ... file content rendered ...    │
└──────────────────────────────────┘
```

### State: Dirty (저장 안 됨)
```
┌──────────────────────────────────┐
│ [main.rs] ● [style.css] [README] │  ← ● white dot on main.rs
├──────────────────────────────────┤
│ ... modified content ...          │
└──────────────────────────────────┘
```

### State: Loading
```
┌──────────────────────────────────┐
│ [main.rs] [loading...]           │
├──────────────────────────────────┤
│                                   │
│     ⟳ 로드 중...                 │
│                                   │
└──────────────────────────────────┘
```

---

## 다크/라이트 테마

### Dark Theme
- Tab active bg: neutral.800 (`#27272A`)
- Tab inactive bg: neutral.900 (`#18181B`)
- Tab text: neutral.50 (active) / neutral.300 (inactive)
- Divider: neutral.700 (default) / primary.500 (hover)

### Light Theme
- Tab active bg: neutral.0 (`#FFFFFF`)
- Tab inactive bg: neutral.100 (`#F4F4F5`)
- Tab text: neutral.950 (active) / neutral.700 (inactive)
- Divider: neutral.200 (default) / primary.500 (hover)

---

## Animation & Motion

### Tab Switch
- Duration: 200ms (normal)
- Easing: spring (bouncy feel)
- Effect: underline slide + content cross-fade

### Pane Split
- Duration: 200ms (normal)
- Easing: spring
- Effect: divider slide + panes expand/shrink smooth

### Divider Drag
- 실시간 smooth (no easing, instant follow mouse)
- Min constraint check (on drop)

---

## Persistence (SPEC-V3-003 MS-3)

### Save on Exit
```yaml
panes:
  - type: "leaf"
    active_tab: 0
    tabs:
      - path: "/path/to/main.rs"
        scroll_position: 1234
        selection: {start: 0, end: 0}
      - path: "/path/to/style.css"
  - type: "split"
    direction: "horizontal"
    ratio: 0.6
    left:
      # ... subtree
    right:
      # ... subtree
```

### Restore on Open
1. 저장된 pane tree JSON 로드
2. 각 파일 존재 여부 확인 (없으면 skip)
3. Scroll position 및 cursor 복원
4. last_focused_pane 복원

---

## Accessibility

### Keyboard Navigation
- Tab order: pane 순서 (좌→우, 위→아래)
- Focus ring: 5px primary.500
- 모든 조작 keyboard-only 가능

### Color Contrast
- Active tab text: neutral.50 on neutral.800 (dark) = 16:1 ✓
- Inactive tab text: neutral.300 on neutral.900 (dark) = 5.8:1 ✓
- Divider: primary.500 on panel = 3:1 ✓ (UI component)

### Motion Safety
- prefers-reduced-motion: animation duration → 0ms

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — SPEC-V3-003 MS-1/2/3 구현됨

