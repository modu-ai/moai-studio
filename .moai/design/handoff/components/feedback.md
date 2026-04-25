# UI Components — Feedback Elements

---
title: Feedback Component Library
version: 1.0.0
source: tokens.json motion tokens
last_updated: 2026-04-25
---

## Toast Notification

### 4가지 Type

#### Success

```
┌──────────────────────────────┐
│ ✓ Operation completed        │ ← green bg, white text
└──────────────────────────────┘
```

- Background: success.green (`#10B981`)
- Icon: ✓ (check)
- Duration: 5 seconds auto-dismiss
- Position: 우상단 (toast.z 2000)

#### Error

```
┌──────────────────────────────┐
│ ✗ An error occurred          │ ← red bg
│ Try again                    │
└──────────────────────────────┘
```

- Background: error.red (`#EF4444`)
- Icon: ✗ (cross)
- Duration: 10 seconds (긴 시간, 사용자 action 필요)
- Action button: optional [Retry] / [Dismiss]

#### Warning

```
┌──────────────────────────────┐
│ ⚠️ Warning message           │ ← yellow bg
└──────────────────────────────┘
```

- Background: warning.yellow (`#F59E0B`)
- Icon: ⚠️ (triangle)
- Duration: 7 seconds

#### Info

```
┌──────────────────────────────┐
│ ℹ️ Did you know?             │ ← blue bg
└──────────────────────────────┘
```

- Background: info.blue (`#3B82F6`)
- Icon: ℹ️ (circle)
- Duration: 5 seconds

### 스타일

- Padding: 12px (x) / 8px (y)
- Border radius: 6px (md)
- Font: 12px (sm), 600 weight (semibold)
- Shadow: 3 (elevated)
- Animation: spring 200ms (appear from right)
- Z-index: 2000 (topmost)

### 위치

- 우상단: X: 16px from right, Y: 16px from top
- 스택: multiple toasts → 아래로 offset (각각 +80px Y)

## Banner

### Persistent In-Page Notification

```
╔═════════════════════════════════════╗
║ ℹ️ This workspace is read-only     ║
║    Create a new fork to edit       ║
║                                     ║
║ [Learn more] [Create fork]  [×]     ║
╚═════════════════════════════════════╝
```

- 위치: 페이지 상단, full width
- 배경: info.blue @ 10% (subtle)
- 테두리: 1px info.blue (좌측만 thick 4px)
- Padding: 12px
- 버튼: 링크 또는 action (하단 우측)
- 닫기: × button (우상단, dismiss 이후 don't show again 옵션)

### Type Variations

| Type | BG | Icon | 용도 |
|------|-----|------|------|
| Info | blue @ 10% | ℹ️ | 안내 메시지 |
| Warning | yellow @ 10% | ⚠️ | 주의 메시지 |
| Error | red @ 10% | ❌ | 에러 상황 |
| Success | green @ 10% | ✓ | 성공 상황 |

## Loading Spinner

### 3가지 Size

#### Small (16px)

```
  ⟳
```

- Diameter: 16px
- Animation: 1 rotation = 1 second (linear)
- Color: primary.500
- Stroke: 2px

#### Medium (32px, 기본)

```
    ⟳
```

- Diameter: 32px
- Animation: 1 rotation = 1 second (linear)
- Color: primary.500
- Stroke: 3px

#### Large (48px)

```
      ⟳
```

- Diameter: 48px
- Animation: 1 rotation = 1 second (linear)
- Color: primary.500
- Stroke: 4px

### Accessibility

- Animation: prefers-reduced-motion 시 정지된 로딩 아이콘 (spinner → 원)
- aria-busy="true" on container

## Progress Bar

### Determinate

```
┌──────────────────────────────────┐
│ ████████░░░░░░░░░░░░░░░░░░░░    │ 35%
└──────────────────────────────────┘
```

- Height: 4px
- Background: neutral.800 (track)
- Fill: primary.500 (filled portion)
- Width: 100% (container)
- Label: 우측 percentage (optional)
- Animation: smooth fill (200ms easeOut per 1% change)

### Indeterminate

```
┌──────────────────────────────────┐
│ ████░░░░░░░░░░░░░░░░░░░░░░░░    │ 로딩 중...
└──────────────────────────────────┘
```

- 막대가 좌→우로 이동 (0→100%, 반복)
- Duration: 2 seconds per cycle (normal 200ms × 10)
- Easing: easeInOut (부드러운 흐름)

## Empty State Illustration

### Placeholder Lines (도형)

```
┌─────────────────────────────┐
│                              │
│  ▄▄▄▄▄▄▄▄  (large line)    │
│  ▄▄▄▄  (title-sized)        │
│                              │
│  ▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄  (text) │
│  ▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄         │
│  ▄▄▄▄▄▄▄▄  ▄▄▄ (clipped)     │
│                              │
│  [CTA Button]               │
└─────────────────────────────┘
```

- 색: neutral.700 (dark), neutral.200 (light)
- Radius: 4px
- Animation: shimmer 1s linear infinite (optional, prefers-reduced-motion 시 off)
- Spacing: auto (simulate real content)

### Skeleton Screen Example

```
┌──────────────────────────────────┐
│ ┌─────────┐ ▄▄▄▄▄▄▄▄           │
│ │  (img)  │ ▄▄▄▄▄▄▄▄ ▄▄▄▄      │
│ └─────────┘ ▄▄▄▄▄▄▄▄ ▄▄▄▄      │
│             ▄▄▄▄▄▄▄▄ ▄▄       │
│                                 │
│ [─────────────] [────]          │
│                                 │
└──────────────────────────────────┘
```

- 이미지 placeholder: neutral.700 rect
- 텍스트 lines: stacked neutral.700 bars
- Button placeholders: neutral.700 boxes

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — 모든 feedback 컴포넌트

