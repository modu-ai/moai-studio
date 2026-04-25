# Visual Identity — MoAI Studio

> **Source**: `.moai/design/tokens.json` (canonical machine-readable) + 모두의AI 브랜드 정체성 + agentic coding IDE 모범 사례 (Zed / VS Code / Cursor) + cross-platform GPUI 0.2.2.
> **Status**: v1.0.0 baseline (2026-04-25). Notion 모두의AI 디자인 문서 미접근 — 후속 PR 로 갱신 가능.
> **Theme**: Dark-first (developer-friendly, terminal-dominant), light alternative.

---

## 0. 디자인 철학

- **Agentic coding IDE**: 터미널 + 코드 에디터 + AI 진행 상황을 한 화면에서 본다.
- **Cross-platform 일관성**: macOS / Linux / Windows 동일 토큰 (네이티브 OS 차이는 component-level 만).
- **Bilingual**: 한국어 (Pretendard) + 영문 (Inter) 동등 우선.
- **Keyboard-first**: 마우스 보다 키보드 단축키 (Cmd/Ctrl 일관성, vim/emacs 친화).
- **Density**: VS Code 보다 컴팩트, Cursor 와 유사. 터미널 14px 기본.

---

## 1. Color Palette

### 1.1 Brand
- **Primary**: `#2563EB` (moai blue) — CTA, active state, focus ring
- **Secondary**: `#8B5CF6` (AI violet) — agent activity, AI-generated content
- **Accent**: `#06B6D4` (cyan) — link, terminal highlight

### 1.2 Neutrals (Zinc 스케일, 50→950)
`#FFFFFF / #FAFAFA / #F4F4F5 / #E4E4E7 / #D4D4D8 / #A1A1AA / #71717A / #52525B / #3F3F46 / #27272A / #18181B / #09090B`

### 1.3 Semantic
- success `#10B981`, warning `#F59E0B`, error `#EF4444`, info `#3B82F6`

### 1.4 Theme Mappings (Dark / Light)

| 용도 | Dark | Light |
|------|------|-------|
| App background | neutral.950 | neutral.0 |
| Panel | neutral.900 | neutral.50 |
| Surface | neutral.800 | neutral.100 |
| Text primary | neutral.50 | neutral.950 |
| Text secondary | neutral.300 | neutral.700 |
| Border default | neutral.700 | neutral.200 |
| Focus ring | primary.500 | primary.500 |
| Tab active bg | neutral.800 | neutral.0 |

### 1.5 Syntax (tree-sitter scope) — `tokens.json` color.syntax 참조

keyword `#C792EA` / string `#C3E88D` / number `#F78C6C` / comment `#546E7A` / function `#82AAFF` / type `#FFCB6B` / variable `#EEFFFF` / operator `#89DDFF`

---

## 2. Typography

- **Sans (UI)**: Pretendard → Inter → system-ui (한글 우선)
- **Mono (Code/Terminal)**: JetBrains Mono → Fira Code → SF Mono → Menlo (ligatures)
- **Serif (Markdown 옵션)**: Charter → Iowan Old Style → Georgia

Scale: 11/12/**14**/16/18/20/24/30/36/48 px (default UI base = 14)
Weight: 400/500/600/700
Line height: 1.2 (tight) / 1.5 (normal) / 1.75 (relaxed)

---

## 3. Spacing (4-base)
`0 / 4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48 / 64 / 80 / 96 px`

## 4. Radius
sm 4 / **md 6 (default)** / lg 8 / xl 12 / 2xl 16 / full 9999

## 5. Shadow (Dark, alpha 30~50%)
0 none / 1 subtle / 2 card / 3 dropdown / 4 modal / 5 hero

## 6. Motion
- Duration: 0 / 120 / 200 / 320 ms
- Easing: easeOut (default) / spring (탭/패널)
- prefers-reduced-motion 시 모두 0ms (WCAG 2.1)

---

## 7. Component Tokens (highlights)

| Component | 핵심 |
|-----------|------|
| Button | padding 12/8, radius md, weight medium |
| Input | padding 12/8, radius md, border 1px |
| Tab | height 32, padding-x 12, radius sm, active=semibold |
| Pane | min 240×120, divider 4px, hover=primary.500 |
| Sidebar | width 240, min 180, max 480 |
| Explorer | indent 16, icon 16, row 24 |
| Terminal | mono 14, line-height 1.4 |
| Markdown viewer | md 16, relaxed, max-w 780 |

자세한 토큰: `.moai/design/tokens.json`

---

## 8. /moai design Workflow 통합

본 토큰 = **canonical input**.

- **Path A (Claude Design import)**: `tokens.json` 의 `claude_design_handoff` 섹션 → claude.ai/design 에 입력 → bundle import
- **Path B (code-based)**: `moai-domain-brand-design` skill 이 `tokens.json` 직접 읽기
- **Implementation**: `expert-frontend` 가 token 값 → GPUI Rust 상수 변환

token 변경 시 본 visual-identity.md 동기화 필요 (canonical = JSON, doc = view).

---

## 9. Cross-platform 일관성

| 영역 | 정책 |
|------|------|
| 색상 / typography / spacing / radius / shadow | 100% 동일 |
| 키 modifier | macOS=Cmd, Linux/Windows=Ctrl (component-level) |
| 시스템 폰트 fallback | OS system-ui 마지막 fallback |
| 메뉴 위치 | macOS=상단 메뉴바, Linux/Windows=윈도우 내부 |
| 네이티브 widget | OS 네이티브 (rfd crate file picker 등) |

---

## 10. Brand Mark / Logo (Pending)

- moai 로고는 모두의AI 메인 브랜드 가이드 의존
- moai-studio sub-mark = wordmark + "Studio" suffix 권장
- 상세 가이드 = Notion 모두의AI 디자인 문서 (접근 가능 시 후속 갱신)

---

Version: 1.0.0
Last Updated: 2026-04-25
Pending: 모두의AI Notion 디자인 문서 정합 (접근 시 PR)
