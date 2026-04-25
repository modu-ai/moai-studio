# Visual Identity — MoAI Studio

> **Source**: `.moai/design/tokens.json` v2.0.0 (canonical) + `.moai/design/from-claude-design/` (Claude Design 핸드오프 번들, 2026-04-25 import) + 모두의AI Notion MoAI SNS Ver 1.0 디자인 시스템.
> **Status**: v2.0.0 (2026-04-25) — v1.0.0 폐기. **모두의AI 공식 브랜드 (`#144a46` 딥 틸 청록)** 적용 완료.
> **Theme**: Dark-first (terminal-dominant), light alternative.
> **Language**: 영어 우선 (글로벌 IDE 톤, VS Code 스타일), 한글 Pretendard fallback.

---

## 0. 브랜드 핵심 자산 (FROZEN — 변경 금지)

### 0.1 Primary Color
- **`#144a46`** — 어두운 청록 (딥 틸). 모두의AI 공식 브랜드 색.
- 다크 모드 대응: `#22938a` (라이트 청록 — 대비 확보)
- Hover: `#0e3835`, Active: `#0a2825`

### 0.2 Signature Gradient (코어 자산)
```css
linear-gradient(135deg, #144a46 0%, #09110f 100%)
```
hero CTA, 마스코트 영역, agent pill 등에 사용. 절대 분해 / 색상 변경 금지.

### 0.3 Pretendard Self-hosted (9 weight)
모두의AI 공식 폰트. `.moai/design/from-claude-design/project/fonts/` 에 9 weight `.otf` 자산 보존.
- Thin 100, ExtraLight 200, Light 300, Regular 400, Medium 500, SemiBold 600, Bold 700, ExtraBold 800, Black 900

### 0.4 Mascot (모아이 로고)
- `.moai/design/from-claude-design/project/assets/`
- `moai-logo-1.png` (primary)
- `moai-logo-3.png` (마스코트 — 헤더/사이드바 22-64px)
- `moai-logo-4.png` / `moai-logo-4-WH.png` (wordmark + dark variant)

---

## 1. Color Palette

### 1.1 Brand
- Primary `#144a46` / Hover `#0e3835` / Active `#0a2825`
- Primary (dark mode) `#22938a` / Hover `#2bafa3`
- Ink `#09110f` (#000 대체) / Bg `#f3f3f3` (#fff 사용 금지)
- Surface `#ffffff`

### 1.2 Neutral Scale (모두의AI 그레이)
50 `#f3f3f3` / 100 `#eaeaea` / 200 `#d4d4d4` / 300 `#bcbcbc` / 400 `#959595` / 500 `#6e6e6e` / 600 `#4c4c4c` / 700 `#2e2e2e` / 800 `#1a1f1d` / 900 `#0e1513` / 950 `#09110f`

### 1.3 Semantic
- success `#1c7c70` (mint, 청록 계열)
- warning `#c47b2a` (amber)
- danger `#c44a3a` (crimson)
- info `#2a8a8c` (cyan)

### 1.4 IDE Surface Accents (tweakable)
**default = teal `#144a46`** (브랜드 정합 우선). 사용자 toggle 시 blue/violet/cyan 가능.

### 1.5 Theme — Dark / Light

| 용도 | Dark | Light |
|------|------|-------|
| App background | `#0a100e` | `#f3f3f3` |
| Panel | `#0e1513` | `#ffffff` |
| Surface | `#131c19` | `#fafaf9` |
| Elevated | `#182320` | `#ffffff` |
| Text primary | `#e6ebe9` | `#09110f` |
| Text secondary | `#98a09d` | `#4c4c4c` |
| Text tertiary | `#6b7370` | `#8a908e` |
| Border subtle | rgba(255,255,255,0.06) | `#eaeaea` |
| Border default | rgba(255,255,255,0.07) | `#e6e6e3` |
| Border strong | rgba(255,255,255,0.14) | `#d4d4d0` |
| Accent base | `#22938a` | `#144a46` |
| Accent soft | rgba(20,74,70,0.14) | rgba(20,74,70,0.14) |

### 1.6 Syntax Highlight (tree-sitter, dark/light 분기)

| Scope | Dark | Light |
|-------|------|-------|
| keyword | `#c792ea` | `#5e3bb0` |
| string | `#88b780` | `#1c7c70` |
| number | `#c47b2a` | (동일) |
| comment | `#6b7370` (italic) | (동일) |
| function | `#4f9fce` | `#155b8a` |
| type | `#d4a45c` | (동일) |
| operator | `#6fc2c2` | (동일) |
| variable | `#e6ebe9` | (theme text-primary) |
| constant / tag | `#c44a3a` | (동일) |
| attribute | `#d4a45c` | (동일) |

---

## 2. Typography

### 2.1 Font Family
- **Sans (Brand)**: `Pretendard` self-hosted 9 weight → system-ui fallback
- **Latin**: `Inter` (lang=en 영문 전용 보완)
- **Mono**: `JetBrains Mono` → SF Mono / Menlo / Consolas

### 2.2 Letter-Spacing (한글 자간 — 필수)
Pretendard 한글은 음수 자간 적용 필수:
- display.tight `-7.5%` (히어로)
- display `-5.0%` (메인 타이틀)
- heading `-5.0%`
- body `-2.5%` (기본)
- body.tight `-5.0%`
- caption `0%`

### 2.3 Scale (rem)
xs 0.75 (12px caption) / sm 0.875 (14px) / **base 1.0 (16px body)** / lg 1.125 / xl 1.25 / 2xl 1.5 (h3) / 3xl 1.875 (h2) / 4xl 2.25 (h1) / 5xl 3 / 6xl 3.75 / display `clamp(2.25rem, 4.5vw, 4rem)`

### 2.4 Weight + Line Height
weights: 100/200/300/400/500/600/700/800/900
line-heights: tight 1.05 (display) / snug 1.25 / **normal 1.5** / relaxed 1.75 (markdown)

---

## 3. Spacing — 4-base (Notion)
0 / 4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48 / 64 / 80 / 96 / 128 px

---

## 4. Radius
none 0 / sm 4 / **md 8 (default)** / lg 16 / xl 24 / pill 32 / full 9999

---

## 5. Shadow (절제 — alpha 0.04~0.20)
xs `0 1px 2px ink/0.04` / sm `0 2px 4px ink/0.06` / md `0 4px 12px ink/0.08` / lg `0 8px 24px ink/0.10` / xl `0 16px 48px ink/0.12` / **signature `0 8px 32px primary/0.20`** (hover only — 청록 글로우)

---

## 6. Motion
- duration: instant 75 / fast 150 / normal 250 / slow 400 / page 600 ms
- easing: default `cubic-bezier(0.4,0,0.2,1)` / bounce `cubic-bezier(0.34,1.56,0.64,1)` / smooth `cubic-bezier(0.16,1,0.3,1)`
- prefers-reduced-motion 시 모든 transition/animation 1ms (WCAG 2.1)

---

## 7. IDE Layout (moai-studio.html 시안 정합)

| 영역 | 크기 |
|------|------|
| Topbar | 38px (traffic lights + cmdbar + agent pill) |
| Tab strip | 36px |
| Status bar | 24px (mono font) |
| Sidebar | 240px (compact 56px) |
| Pane head | 28px |
| Row height | 26px (comfortable) / 22px (compact) |
| Code gutter | 44px |
| Markdown max-width | 780px |
| Agent dashboard cols | 200 / flex / 280 |

---

## 8. /moai design Workflow 통합

본 토큰 = **canonical input** for `/moai design`:

1. **Path A (Claude Design import)**: `.moai/design/from-claude-design/` 가 baseline 산출. `tokens.json` 의 `claude_design_handoff` 메타로 기록.
2. **Path B (code-based)**: `moai-domain-brand-design` skill 이 `tokens.json` 직접 읽기.
3. **Implementation**: `expert-frontend` 가 토큰 → GPUI Rust 상수 변환 (참조: `.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md`).

---

## 9. Cross-platform 일관성

| 영역 | 정책 |
|------|------|
| 색상 / typography / spacing / radius / shadow | 100% 동일 |
| 키 modifier | macOS=Cmd, Linux/Windows=Ctrl |
| 시스템 폰트 fallback | Pretendard 우선 → OS system-ui |
| 메뉴 위치 | macOS=상단 메뉴바, Linux/Windows=윈도우 내부 |
| 네이티브 widget | OS 네이티브 (rfd) |

---

## 10. v1.0.0 → v2.0.0 변경 사항

| 항목 | v1.0.0 (폐기) | v2.0.0 (현재) |
|------|--------------|--------------|
| Primary | `#2563EB` Tailwind blue | **`#144a46` 딥 틸 청록** |
| Neutral | Zinc | 모두의AI 그레이 |
| Background light | `#fff` | `#f3f3f3` (강제) |
| Pretendard | npm/CDN | self-hosted .otf 9 weight |
| Signature gradient | (없음) | `135deg, #144a46 → #09110f` |
| 한글 자간 | 미정의 | display -7.5%, body -2.5% |
| Mascot | (없음) | moai-logo-3.png 헤더/사이드바 |

---

Version: 2.0.0
Last Updated: 2026-04-25
Bundle: `.moai/design/from-claude-design/` (Claude Design handoff, 14MB — Pretendard fonts + logos + JSX components + chat transcript)
Canonical tokens: `.moai/design/tokens.json` v2.0.0
