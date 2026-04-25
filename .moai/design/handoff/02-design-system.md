# MoAI Studio — Design System Tokens

---
title: Comprehensive Design System
version: 1.0.0
source: tokens.json (canonical)
last_updated: 2026-04-25
---

## 개요

모든 디자인 결정은 **`.moai/design/tokens.json`** (canonical machine-readable source) 을 중심으로 수행된다. 본 문서는 그 요약본으로, Claude Design 이 신속하게 참조할 수 있도록 구성했다.

---

## 색상 팔레트

### 1. 브랜드 색상 (Brand)

| 역할 | 색상 | HEX | 사용처 |
|------|------|-----|--------|
| Primary | moai blue | `#2563EB` | CTA 버튼, active state, focus ring, selection highlight |
| Secondary | AI violet | `#8B5CF6` | Agent activity, AI-generated content, secondary accent |
| Accent | Cyan | `#06B6D4` | Link, terminal highlight, secondary action |

### 2. 뉴트럴 (Zinc Scale)

| Stop | HEX | 용도 |
|------|-----|------|
| 0 (White) | `#FFFFFF` | Highlights, elevated surfaces |
| 50 | `#FAFAFA` | Light theme backgrounds |
| 100 | `#F4F4F5` | Light theme panels |
| 200 | `#E4E4E7` | Light theme borders |
| 300 | `#D4D4D8` | Disabled text, subtle borders |
| 400 | `#A1A1AA` | Tertiary text, placeholder |
| 500 | `#71717A` | Secondary text (light theme) |
| 600 | `#52525B` | Disabled state, subtle text |
| 700 | `#3F3F46` | Border default (dark theme) |
| 800 | `#27272A` | Surface (dark theme), tab active bg |
| 900 | `#18181B` | Panel (dark theme) |
| 950 | `#09090B` | App background (dark theme) |

### 3. 의미론적 색상 (Semantic)

| 상태 | HEX | 용도 |
|------|-----|------|
| Success | `#10B981` | Test pass, build success, AC GREEN |
| Warning | `#F59E0B` | Lint warning, deprecated, TODO |
| Error | `#EF4444` | Compile error, test failure |
| Info | `#3B82F6` | Hint, note, neutral notification |

### 4. 신택스 하이라이트 (tree-sitter scope)

| Element | HEX |
|---------|-----|
| Keyword | `#C792EA` |
| String | `#C3E88D` |
| Number | `#F78C6C` |
| Comment | `#546E7A` |
| Function | `#82AAFF` |
| Type | `#FFCB6B` |
| Variable | `#EEFFFF` |
| Operator | `#89DDFF` |
| Constant | `#F07178` |
| Tag | `#F07178` |
| Attribute | `#FFCB6B` |

### 5. 테마별 매핑 (Dark / Light)

#### Dark Theme (기본)

| 용도 | 값 | HEX |
|------|-----|-----|
| App background | neutral.950 | `#09090B` |
| Panel background | neutral.900 | `#18181B` |
| Surface | neutral.800 | `#27272A` |
| Elevated surface | neutral.700 | `#3F3F46` |
| Text primary | neutral.50 | `#FAFAFA` |
| Text secondary | neutral.300 | `#D4D4D8` |
| Text tertiary | neutral.400 | `#A1A1AA` |
| Text disabled | neutral.600 | `#52525B` |
| Link | brand.accent.500 | `#06B6D4` |
| Border subtle | neutral.800 | `#27272A` |
| Border default | neutral.700 | `#3F3F46` |
| Border strong | neutral.600 | `#52525B` |
| Focus ring | brand.primary.500 | `#2563EB` |
| Tab active bg | neutral.800 | `#27272A` |
| Tab inactive bg | neutral.900 | `#18181B` |

#### Light Theme (대안)

| 용도 | 값 | HEX |
|------|-----|-----|
| App background | neutral.0 | `#FFFFFF` |
| Panel background | neutral.50 | `#FAFAFA` |
| Surface | neutral.100 | `#F4F4F5` |
| Elevated surface | neutral.0 | `#FFFFFF` |
| Text primary | neutral.950 | `#09090B` |
| Text secondary | neutral.700 | `#3F3F46` |
| Text tertiary | neutral.500 | `#71717A` |
| Text disabled | neutral.300 | `#D4D4D8` |
| Link | brand.primary.700 | `#1E40AF` |
| Border subtle | neutral.100 | `#F4F4F5` |
| Border default | neutral.200 | `#E4E4E7` |
| Border strong | neutral.400 | `#A1A1AA` |
| Focus ring | brand.primary.500 | `#2563EB` |
| Tab active bg | neutral.0 | `#FFFFFF` |
| Tab inactive bg | neutral.100 | `#F4F4F5` |

### 6. 차트 색상 (5-step)

Dark theme: `#60A5FA` / `#A78BFA` / `#34D399` / `#FBBF24` / `#F87171`  
Light theme: `#3B82F6` / `#8B5CF6` / `#10B981` / `#F59E0B` / `#EF4444`

---

## Typography

### 1. Font Families

#### Sans (UI 기본)
```
Pretendard → Inter → system-ui → -apple-system → Segoe UI → Roboto → sans-serif
```
**순서**: 한글 우선 (Pretendard), 영문 fallback (Inter)

#### Mono (Code / Terminal)
```
JetBrains Mono → Fira Code → SF Mono → Menlo → Consolas → monospace
```
**특징**: Ligatures 지원 (=>, ->, <=>, ||, &&)

#### Serif (Markdown 본문, 선택사항)
```
Charter → Iowan Old Style → Georgia → serif
```

### 2. Font Sizes

| Scale | Size | 용도 |
|-------|------|------|
| xs | 11px | 미니 라벨, timestamp |
| sm | 12px | 보조 텍스트, caption, code small |
| base | 14px | **기본 UI 텍스트 + 코드 에디터** |
| md | 16px | 마크다운 본문 |
| lg | 18px | Subheading |
| xl | 20px | Heading 3 |
| 2xl | 24px | Heading 2 |
| 3xl | 30px | Heading 1 (large) |
| 4xl | 36px | Hero title |
| 5xl | 48px | App header |

### 3. Font Weights

| Weight | Value | 용도 |
|--------|-------|------|
| Regular | 400 | 기본 텍스트 |
| Medium | 500 | 약간 강조 (button labels, tab inactive) |
| Semibold | 600 | 강조 (tab active, heading) |
| Bold | 700 | 매우 강조 (hero, emphasis) |

### 4. Line Heights

| Name | Value | 용도 |
|------|-------|------|
| Tight | 1.2 | UI components, compact |
| Normal | 1.5 | 기본 텍스트 (UI body) |
| Relaxed | 1.75 | 마크다운 본문 (읽기 편의) |

### 5. Letter Spacing

| Name | Value | 용도 |
|------|-------|------|
| Tight | -0.01em | Heading 강조 |
| Normal | 0 | 기본 |
| Wide | 0.02em | Small caps, 특수 용도 |

---

## Spacing (4-base Scale)

| Step | Value | 용도 |
|------|-------|------|
| 0 | 0 | Reset |
| 1 | 4px | Micro-spacing, line-height gap |
| 2 | 8px | **Button padding-y, tight gap** |
| 3 | 12px | **Button padding-x, input padding** |
| 4 | 16px | **Standard padding, section gap** |
| 5 | 20px | Large gap, section margin |
| 6 | 24px | Component margin |
| 8 | 32px | Sidebar/panel spacing |
| 10 | 40px | Major section spacing |
| 12 | 48px | Hero spacing |
| 16 | 64px | Full-page margin |
| 20 | 80px | Large hero margin |
| 24 | 96px | Extra large layout |

---

## Radius

| Name | Value | 용도 |
|------|-------|------|
| none | 0 | 직선 (divider, grid) |
| sm | 4px | Small elements (tag, icon button) |
| **md** | 6px | **기본 (button, input, card)** |
| lg | 8px | Slightly rounded (card hover) |
| xl | 12px | Larger components (modal, panel) |
| 2xl | 16px | Very rounded (large hero) |
| full | 9999px | 완전 원형 (avatar, circle button) |

---

## Shadow (Elevation Scale)

Dark theme adjusted (alpha 30~50%), light theme reduced:

| Level | Value | 용도 |
|-------|-------|------|
| 0 | none | Flat, no elevation |
| 1 | 0 1px 2px rgba(0,0,0,0.30) | Subtle hover, tooltip |
| 2 | 0 2px 4px rgba(0,0,0,0.35) | Card, panel default |
| 3 | 0 4px 8px rgba(0,0,0,0.40) | Dropdown, popover |
| 4 | 0 8px 16px rgba(0,0,0,0.45) | Modal, elevated panel |
| 5 | 0 16px 32px rgba(0,0,0,0.50) | Hero, topmost |

---

## Z-Index Stack

| Level | Value | 용도 |
|-------|-------|------|
| base | 0 | Content, normal flow |
| tabs | 10 | Tab bar above content |
| sidebar | 20 | Sidebar above content |
| toolbar | 30 | Top toolbar |
| dropdown | 100 | Select dropdown, context menu |
| popover | 200 | Popover, tooltip |
| modal | 1000 | Modal dialog, scrim |
| toast | 2000 | Toast notification (topmost) |

---

## Motion

### Duration

| Name | Value | 용도 |
|------|-------|------|
| instant | 0ms | Immediate (no delay) |
| fast | 120ms | Quick feedback (button press) |
| normal | 200ms | **기본 transition** |
| slow | 320ms | Intentional delay (pane split, tab switch) |

### Easing

| Name | Cubic Bezier | 용도 |
|------|--------------|------|
| linear | linear | Constant speed (rare) |
| **easeOut** | cubic-bezier(0.16, 1, 0.3, 1) | **기본 (appear fast, slow end)** |
| easeInOut | cubic-bezier(0.4, 0, 0.2, 1) | Balanced motion |
| spring | cubic-bezier(0.34, 1.56, 0.64, 1) | 탭/패널 split (bouncy feel) |

### Accessibility
```
prefers-reduced-motion: all animations → 0ms
```

---

## Breakpoints (Desktop-First)

| Name | Value | Device |
|------|-------|--------|
| compact | 1024px | Small laptop, compact desktop |
| **default** | 1440px | **Standard desktop** |
| wide | 1920px | Large desktop, 4K |

**모바일 대응 비목표** (desktop-first IDE).

---

## Component Token Collections

### Button
- padding-x: 12px (spacing.3)
- padding-y: 8px (spacing.2)
- radius: 6px (radius.md)
- font-weight: 500 (medium)
- font-size: 14px (base)
- **4 size variants**: sm/md/lg/xl (스케일링)
- **4 style variants**: primary/secondary/ghost/destructive
- State: default / hover (+1 shadow) / active (darker bg) / disabled (neutral.600 text)

### Input (text, textarea, select)
- padding-x: 12px
- padding-y: 8px
- radius: 6px
- border-width: 1px
- border-color: neutral.700 (dark), neutral.200 (light)
- State: default / focus (border=primary.500, shadow.1) / error (border=error) / disabled

### Checkbox / Radio / Switch
- Size: 16×16 (checkbox/radio), 32×16 (switch)
- Color: primary.500 (checked)
- State: default / hover / active / disabled

### Tab (container 내)
- height: 32px
- padding-x: 12px
- radius: 4px (sm)
- active-font-weight: 600 (semibold)
- inactive-font-weight: 400 (regular)
- active-background: neutral.800 (dark), neutral.0 (light)
- inactive-background: neutral.900 (dark), neutral.100 (light)
- **Animation**: spring 200ms (switch smooth)

### Pane (container 내)
- min-width: 240px
- min-height: 120px
- divider-thickness: 4px
- divider-hover-color: primary.500
- divider-drag: spring 200ms smooth

### Sidebar
- width: 240px (default)
- min-width: 180px (collapsed icon)
- max-width: 480px (expanded)
- border-right: 1px neutral.700

### Explorer (File Tree)
- indent-per-level: 16px
- icon-size: 16px
- row-height: 24px (tight)
- hover-background: neutral.800 (dark), neutral.100 (light)

### Terminal
- font-family: JetBrains Mono
- font-size: **14px (base)**
- line-height: 1.4
- padding: 8px (inner)
- background: neutral.950 (dark), neutral.0 (light)

### Markdown Viewer
- font-size: 16px (md)
- line-height: 1.75 (relaxed)
- max-width: 780px (readability)
- code-font-size: 12px (sm)
- code-background: neutral.800 (dark), neutral.100 (light)
- h1-font-size: 30px (3xl)
- h2-font-size: 24px (2xl)
- h3-font-size: 20px (xl)

### Code Viewer (LSP editor)
- font-family: JetBrains Mono
- font-size: 14px (base)
- line-height: 1.4
- gutter-width: 40px
- 신택스 highlight: color.syntax.* 참조

### Agent Dashboard
- event-font-size: 12px (sm)
- event-line-height: 1.5 (normal)
- cost-chart-height: 180px
- timeline-dot-size: 12px

---

## Accessibility Standards

### Contrast Ratio
- Text ↔ Background: ≥ 4.5:1 (normal text)
- Large text (18px+): ≥ 3:1
- UI component ↔ border: ≥ 3:1

### Focus Indicator
- **Thickness**: 5px
- **Color**: primary.500
- **Offset**: 4px from element edge
- **Shape**: Rounded (follow element radius)
- **Always visible** in all states

### Keyboard Navigation
- Tab order: logical (left→right, top→bottom)
- Tab cycles through: buttons, inputs, selects, checkboxes, links
- Skip links: optional (not required for IDE)
- Shortcuts: Cmd/Ctrl 매핑 automatic

### Motion Safety
- Default: motion enabled (200ms easeOut)
- prefers-reduced-motion=reduce: **모든 animation → 0ms**

---

## 성능 제약

### 렌더링
- 목표 FPS: 60fps (120Hz 지원)
- Frame budget: 16ms per frame
- Terminal rendering: libghostty GPU-accelerated
- Code editor: virtual scrolling (1000+ lines)

### Animation Duration
- Max duration: 320ms (slow)
- Min duration: 0ms (prefers-reduced-motion)

---

**마지막 수정**: 2026-04-25  
**Canonical Source**: `.moai/design/tokens.json`  
**상태**: 안정 버전 1.0.0

