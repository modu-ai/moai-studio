# UI Components — Buttons & Inputs

---
title: Button and Input Component Library
version: 1.0.0
source: tokens.json component tokens
last_updated: 2026-04-25
---

## Button

### 4가지 Style

#### Primary
```
┌─────────────────┐
│  Save Changes   │ ← primary.500 bg, white text
└─────────────────┘
```
- Background: primary.500 (`#2563EB`)
- Text: neutral.0 (`#FFFFFF`)
- Hover: primary.600 (darker)
- Active: primary.700 (darker)
- Disabled: neutral.600 (gray)

#### Secondary
```
┌─────────────────┐
│  Cancel         │ ← neutral.700 border, neutral.50 text
└─────────────────┘
```
- Border: 1px neutral.700
- Text: neutral.50
- Background: transparent
- Hover: neutral.750 bg
- Active: neutral.800 bg

#### Ghost
```
  Clear          ← no border, neutral.50 text, hover only
```
- Border: none
- Text: neutral.50
- Background: transparent
- Hover: neutral.800 bg (subtle)

#### Destructive
```
┌──────────┐
│  Delete  │ ← error.red bg
└──────────┘
```
- Background: error.red (`#EF4444`)
- Text: neutral.0
- Hover: error.red @ 90%
- Confirm: double-click or dialog required

### 4가지 Size

| Size | Padding | Font | 용도 |
|------|---------|------|------|
| sm | 8/4 | 12px | Small action, secondary |
| md | 12/8 | 14px | **기본** |
| lg | 16/10 | 16px | Prominent action |
| xl | 20/12 | 18px | Hero button, large screen |

### 상태

```
Default:  ┌─────────┐      Hover:  ┌─────────┐ (shadow +1)
          │ Button  │             │ Button  │
          └─────────┘             └─────────┘

Active:   ┌─────────┐      Disabled: [Button] (gray, no interaction)
          │ Button  │ (darkened bg, pressed feel)
          └─────────┘
```

## Input

### Text Input

```
┌─────────────────────────────────┐
│ [placeholder or value]          │ ← 14px base, mono or sans
└─────────────────────────────────┘
```

- Padding: 12/8 (horizontal/vertical)
- Border: 1px neutral.700 (dark), neutral.200 (light)
- Radius: 6px (md)
- Font: Pretendard / Inter 14px
- Focus: border primary.500 + shadow.1
- Error: border error.red, bg error @ 5% alpha
- Disabled: bg neutral.900, text neutral.600

### Textarea

```
┌─────────────────────────────────┐
│ [Line 1]                        │
│ [Line 2]                        │
│ [Line 3...]                     │
└─────────────────────────────────┘
```

- Height: 120px default (resize 가능)
- Font: 14px mono (code) or sans (default)
- Vertical scroll: 10+ lines
- Resize handle: 우측 하단

### Select

```
┌─────────────────────────────────┐
│ [Option 1          ▼]           │
├─────────────────────────────────┤
│ Option 1 (selected)             │
│ Option 2                        │
│ Option 3                        │
└─────────────────────────────────┘
```

- 호버: 배경 highlight
- 선택: primary.500 체크마크
- 검색: 입력 시 fuzzy filter
- 키보드: arrow up/down + enter

### Search Input

```
┌─────────────────────────────────┐
│ 🔍 [search query]        [×]    │
└─────────────────────────────────┘
```

- 아이콘: 좌측 (🔍)
- Clear button: 우측 (× on filled)
- 배경: neutral.800 (darker, terminal-like)
- 실시간 검색 (debounce 200ms)

## Checkbox / Radio

### Checkbox

```
☑ Label text         ← checked (primary.500)

☐ Label text         ← unchecked
```

- Size: 16×16
- Border: 1px (unchecked), filled (checked)
- Color: primary.500 (checked)
- Label: 12px (sm) text, 8px gap

### Radio

```
◉ Option 1           ← selected (primary.500 dot)

○ Option 2           ← unselected
```

- Size: 16×16
- Border: 2px (unselected), filled (selected)
- Inner dot: primary.500 (selected)

## Switch / Toggle

### Switch

```
[◉────] Label text    ← enabled (primary.500)
[────◯] Label text    ← disabled (neutral.600)
```

- Width: 32px, Height: 16px
- Border radius: full (9999px)
- Color: primary.500 (on), neutral.600 (off)
- Animation: 200ms easeOut (toggle)
- Keyboard: Space to toggle

## Tooltip / Popover

### Tooltip

```
┌──────────────────┐
│ Help text here   │ ← 12px, 200ms appear
└──────────────────┘
      ▲
   [Button]
```

- 배경: neutral.900 (dark), neutral.100 (light)
- 테두리: 1px neutral.700
- Radius: 4px
- Padding: 8px
- Max width: 240px (text wrap)
- Pointer: center-aligned

### Popover

```
┌──────────────────────────┐
│ [×]                      │ ← close button
│ Title                    │
│                          │
│ Content with more space  │
│ and potential actions    │
│                          │
│ [Cancel] [Confirm]       │
└──────────────────────────┘
```

- 배경: neutral.900 (dark), neutral.50 (light)
- 테두리: 1px + shadow.2
- Radius: 8px (lg)
- Padding: 16px
- Z-index: 200

## Dialog / Modal

### Dialog (focused foreground)

```
────────────────────────────────
│ Important Confirmation        │
├────────────────────────────────┤
│                                │
│ Are you sure?                  │
│                                │
│ This action cannot be undone.  │
│                                │
│ [Cancel] [Delete]              │
└────────────────────────────────
──────────────────────────────────
```

- 배경: neutral.950 @ 80% (scrim)
- Dialog: neutral.900 (dark)
- Width: 400px (responsive, max 90vw)
- Z-index: 1000

## 접근성

### Focus Ring
- 두께: 5px
- 색: primary.500
- Offset: 4px
- 모든 interactive 요소 필수

### Contrast
- Button text on bg: ≥ 4.5:1
- Label text: ≥ 4.5:1
- Disabled state: 3:1 (acceptable for disabled)

### Keyboard

| Key | 동작 |
|-----|------|
| Tab | 다음 요소 focus |
| Shift+Tab | 이전 요소 focus |
| Enter | Button click / Select option |
| Space | Checkbox/Radio toggle / Button click |
| Arrow Up/Down | Select option navigation |
| Escape | Dialog close |

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — 모든 기본 컴포넌트

