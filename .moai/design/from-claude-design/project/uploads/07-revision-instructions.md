---
title: MoAI Studio v3 Design Revision Instructions
version: 1.0.0
source: SPEC-V3-001 through SPEC-V3-010 AC analysis
design_bundle: .moai/design/from-claude-design/
last_updated: 2026-04-25
usage: "이 문서를 claude.ai/design 의 기존 프로젝트에 추가 컨텍스트로 입력하세요. SPEC AC 정의와 시안의 gap을 모두 해소한 후 export하시면 완성입니다."
---

# MoAI Studio v3 — 디자인 수정 의뢰서 (Round 2)

## §0. Executive Summary

### 분석 대상
- **10개 SPEC**: SPEC-V3-001 through SPEC-V3-010 (총 ~80+ AC)
- **설계된 영역**: 9개 surface (Terminal, FileExplorer, CodeViewer, Markdown, AgentDashboard, GitMgmt, SpecKanban, WebBrowser, AppShell)
- **설계된 상태**: 4개 기본 state (EmptyState, LoadingSkeleton, ErrorBanner, FirstRun)

### 발견된 gap 통계

| 상태 | 개수 | 설명 |
|------|------|------|
| ✓ 정상 반영 | 58 AC | Terminal core, Tabs, Panes, Markdown structure, AgentDashboard layout |
| ⚠ 수정 필요 | 16 AC | Token 불일치, variant 미완성 (light/compact), 세부 치수 |
| ✗ 누락 | 13 AC | Quick Open palette, Settings panel, LSP hover, MX tag popover, Sprint panel, Command palette, Find/Replace, Update banner, Crash recovery, SSH UI, RTL, Multi-window, Shortcut cheat |

### 우선순위 분류

| Priority | AC 수 | 작업 |
|----------|-------|------|
| **P0 — 즉시** | 16 | Token 불일치 교정 (모든 색상 → tokens.json v2.0.0), light theme 모든 surface, 치수 정확화 |
| **P1 — Round 2** | 13 | 신규 surface 9개 (palette, settings, modals, popovers) |
| **P2 — 향후** | 선택 | RTL, multi-window, compact density (Phase 4+) |

---

## §1. Design Token 강제 — 모든 revision 의 baseline

**중요**: 본 섹션의 토큰은 **변경 불가능한** 강제 사항입니다. Claude Design 이 이전 시안에서 사용한 색상을 다시 한 번 모두의AI 공식 시스템으로 통일합니다.

### 1.1 색상 팔레트 (tokens.json v2.0.0에서 직접 추출)

#### 브랜드 색상

```json
{
  "primary":        "#144a46",    // 모두의AI 딥 틸 청록 (CTA, 타이틀, 아이콘)
  "primary.dark":   "#22938a",    // 다크 모드용 라이트 청록
  "ink":            "#09110f",    // 본문 텍스트
  "bg.light":       "#f3f3f3",    // 라이트 모드 페이지 배경
  "surface.dark":   "#131c19",    // 다크 모드 카드/surface
  "surface.light":  "#ffffff"     // 라이트 모드 카드
}
```

#### 의미 색상 (MUST USE 정확히)

```json
{
  "success":  "#1c7c70",  // Test pass, build success, AC GREEN
  "warning": "#c47b2a",   // Lint warning, deprecated, TODO
  "danger":   "#c44a3a",  // Compile error, test failure
  "info":     "#2a8a8c"   // Hint, note, notification (사이언)
}
```

#### IDE 액센트 (tweakable but 기본값 강제)

```json
{
  "default (teal)": "#144a46",
  "blue":          "#2563EB",  // ❌ 이전 시안에서 발견됨 → #144a46 로 교체
  "violet":        "#6a4cc7",
  "cyan":          "#06B6D4"
}
```

#### Signature Gradient (절대 분해 금지)

```css
linear-gradient(135deg, #144a46 0%, #09110f 100%)  /* 다크 모드 */
linear-gradient(135deg, #22938a 0%, #144a46 100%)  /* 다크 모드 hover */
```

#### 중성 톤

```json
{
  "neutral.50":    "#f3f3f3",
  "neutral.100":   "#eaeaea",
  "neutral.200":   "#d4d4d4",
  "neutral.300":   "#bcbcbc",
  "neutral.400":   "#959595",
  "neutral.500":   "#6e6e6e",
  "neutral.600":   "#4c4c4c",
  "neutral.700":   "#2e2e2e",
  "neutral.800":   "#1a1f1d",
  "neutral.900":   "#0e1513",
  "neutral.950":   "#09110f"
}
```

### 1.2 Typography (Pretendard 9 weight)

```css
/* 자간 — 한글 필수 (음수) */
display.tight:   letter-spacing: -0.075em  /* 히어로 */
display:         letter-spacing: -0.05em   /* 메인 타이틀 */
heading:         letter-spacing: -0.05em
body:            letter-spacing: -0.025em
body.tight:      letter-spacing: -0.05em
caption:         letter-spacing: 0         /* 캡션은 0 */

/* Font weight */
thin:       100
light:      300
regular:    400
medium:     500
semibold:   600
bold:       700
extrabold:  800
black:      900
```

### 1.3 라이트/다크 테마 — 모든 surface 정합 필수

**현재 상황**: HTML/CSS 시안이 다크 모드 위주. 모든 component 를 light/dark 각각 시연할 것.

#### Dark Theme 변수

```css
--d-bg:         #0a100e
--d-panel:      #0e1513
--d-surface:    #131c19
--d-elev:       #182320
--d-border:     rgba(255,255,255,0.07)
--d-border-strong: rgba(255,255,255,0.14)
--d-fg:         #e6ebe9
--d-fg-2:       #98a09d
--d-fg-3:       #6b7370
```

#### Light Theme 변수

```css
--l-bg:         #f3f3f3
--l-panel:      #ffffff
--l-surface:    #fafaf9
--l-elev:       #ffffff
--l-border:     #e6e6e3
--l-border-strong: #d4d4d0
--l-fg:         #09110f
--l-fg-2:       #4c4c4c
--l-fg-3:       #8a908e
```

---

## §2. SPEC-to-Design Gap Matrix (핵심 섹션)

각 SPEC별로 AC를 나열하고 시안 반영 현황을 표시합니다.

### §2.1 SPEC-V3-001: GPUI Scaffold + RootView 4영역

#### 시안 매핑
- `moai-app.jsx` App shell
- `moai-studio.html` .moai / .moai-top / .moai-side / .moai-status

#### AC ↔ UI 매핑

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-1.1 | Cargo workspace 289 tests 회귀 0 | (코드 테스트) | ✓ | — |
| AC-2.1 | 1600×1000 윈도우 + 4 영역 | .moai 타이틀바(38px)/사이드바(240px)/콘텐츠/상태바(24px) | ✓ | — |
| AC-2.2 | Welcome CTA (3 버튼) | .empty state 있음 | ✓ | 텍스트 확인: "Create First Workspace" / "Start Sample" / "Open Recent" |
| AC-2.3 | "+ New Workspace" 네이티브 폴더 picker | (interactive flow) | ⚠ | Wireframe: 버튼 위치 확인, 선택 후 표시 모습 확인 |

#### Claude Design 에게 보낼 구체 지시 (SPEC-V3-001 한정)

> **SPEC-V3-001 revision instructions**:
> 1. 타이틀바 좌측에 macOS traffic light (또는 Linux minimize/maximize/close) 영역 [13px gap]. 우측에 agent pill + theme toggle + account menu 배치.
> 2. 사이드바 240px 유지. 맨 위 "MoAI Studio" 로고 + mascot (22px moai-logo-3.png) 조합 (44px 섹션).
> 3. "+ New Workspace" 버튼 스타일: accent bg, weight bold, 32px height, radius 5px.
> 4. 콘텐츠 영역 Welcome CTA: mascot 84px 중앙, h3 제목 17px weight 700, 설명 12.5px fg-3 max-w 360px, 3 버튼 flex row, 버튼 height 32px.
> 5. 상태바: 좌측에 "Ready" 또는 workspace name, 우측에 clock + "CPU 2.4% MEM 285MB" 또는 agent status.

---

### §2.2 SPEC-V3-002: Terminal Core + Grid Rendering

#### 시안 매핑
- `moai-surfaces.jsx` Terminal surface
- `moai-studio.html` .term class

#### AC ↔ UI 매핑

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-T-1 | libghostty-vt FFI 링크 | (코드) | ✓ | — |
| AC-T-4 | GPUI 윈도우에서 `$SHELL` prompt 렌더 | .term .pr color | ✓ | Prompt 색상 = primary.dark (#22938a) 확인 |
| AC-T-5 | 터미널 내 명령 echo | (코드 동작) | ✓ | — |
| AC-T-7 | 60fps frame rate | (성능 테스트) | ✓ | — |
| AC-T-9 | TerminalSurface 첫 프레임 ≤200ms | (성능 벤치) | ✓ | — |
| AC-T-10 | Cmd+C (selection 없음) → SIGINT | (key routing) | ✓ | — |
| AC-T-11 | Cmd+C (selection 있음) → clipboard | (clipboard API) | ✓ | — |

#### 의미 색상 매핑 (Terminal 특화)

```css
.ok   = #1c7c70    /* success */
.er   = #c44a3a    /* danger */
.wr   = #c47b2a    /* warning */
.cm   = fg-3       /* comment */
.nm   = #6a4cc7    /* violet — 명령명 */
.br   = #06B6D4    /* cyan — 브랜치 */
```

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-002 revision instructions**:
> 1. Terminal 배경: dark theme 시 #0a100e, light theme 시 #f3f3f3.
> 2. 터미널 텍스트: mono 12.5px, line-height 1.5, padding 10px 14px.
> 3. Prompt color = primary.dark (#22938a) 정확히. 이전 시안에 다른 색상이 있으면 교체.
> 4. Semantic colors (.ok/.er/.wr/.cm/.nm/.br) 테이블대로 모두 반영.
> 5. Cursor: 7×14px, accent bg, blink 1s steps(2) infinite.
> 6. Selection bg: rgba(20,74,70,0.22) (accent-glow).

---

### §2.3 SPEC-V3-003: Tabs + Panes + Persistence

#### 시안 매핑
- `moai-studio.html` .moai-tabs / .moai-tab.active
- Pane divider drag (CSS cursor: col-resize / row-resize)

#### AC ↔ UI 매핑

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-P-1 | Horizontal split (좌/우) visual feedback | (divider 수직선) | ✓ | Divider 최소 width 4px, cursor col-resize |
| AC-P-2 | Pane close 애니메이션 | (fade-out 200ms) | ⚠ | CSS transition 추가 예시 필요 |
| AC-P-3 | 단일 pane close 무시 (경고 없음) | (noop) | ✓ | — |
| AC-P-4 | Split 최소 크기 40cols×10rows 제약 | (경고 shake 200ms) | ⚠ | Shake animation 규정: `±3px horizontal translation` |
| AC-P-6 | Divider drag clamp | ratio 0.0 < ratio < 1.0 | ✓ | Min/max ratio 계산 예제 |
| AC-P-7 | Prev/Next pane focus | (keyboard shortcut highlight) | ✓ | Focused pane border: 2px accent #144a46 |
| AC-P-8 | Tab switch → last focus 복원 | (highlight) | ✓ | — |
| AC-P-9a (macOS) | Cmd+T/\\\\ 등 키 바인딩 | (visual feedback) | ✓ | — |
| AC-P-9b (Linux) | Ctrl+T/\\\\ 등 키 바인딩 | (visual feedback) | ✓ | — |
| AC-P-10 | 9개 탭 생성 + 독립 pane tree | (tab bar overflow scroll) | ⚠ | Tab bar 가로 스크롤 또는 dropdown 메뉴 처리 |
| AC-P-11 | 탭 전환 시 pane tree 보존 | (visual identity) | ✓ | — |
| AC-P-12 | Persistence 저장 (scrollback 제외) | (JSON schema) | ✓ | — |
| AC-P-13 | Persistence 로드 (cwd 복원) | (initial state) | ✓ | — |
| AC-P-13a | cwd fallback → $HOME | (warning log) | ⚠ | Toast notification 또는 status bar message 표시 |
| AC-P-22 | 두 개 이상 pane focused 금지 | (single focus) | ✓ | — |
| AC-P-23 | tmux 호환성 (prefix key 미적용) | (passthrough) | ✓ | — |
| AC-P-24 | Empty tab container → EmptyState CTA | (conditional render) | ✓ | — |
| AC-P-25 | 탭 개수 상한 제약 없음 (1~9 단축키) | (tab bar + dropdown) | ⚠ | 10+ 탭 접근 UI 명확히 |
| AC-P-26 | 중첩 tmux 시 host 우선 처리 | (event consume) | ✓ | — |
| AC-P-27 | 활성 탭 시각 구분: bg color + bold weight | (dual indicator) | ⚠ | 색상 = toolbar.tab.active.background (plan 단계 확정), weight bold 모두 보여야 함 |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-003 revision instructions**:
> 1. Tab bar: height 36px, 배경 = app bg (#0a100e dark / #f3f3f3 light).
> 2. 활성 탭 (AC-P-27): **(a) 배경색** + **(b) text weight bold** 동시 필수. 색상은 design token `toolbar.tab.active.background` (예정값: teal 관련색).
> 3. Inactive tab: text weight 400, 우측에 close ⨯ 버튼 hover 시 표시.
> 4. Divider (수직선, horizontal split 시): width 4px, cursor col-resize, bg = border-strong. Hover 시 accent glow 추가 (rgba(20,74,70,0.22)).
> 5. Pane close action: 1. shake animation 200ms ±3px (사용자가 최소 크기 위반 시), 2. fade-out 200ms (정상 close 시).
> 6. Tab overflow: 10+ 탭 시 tab bar 우측에 "..." 드롭다운 또는 horizontal scroll 활성화.
> 7. Status message (AC-P-13a): cwd fallback 시 status bar 좌측에 "Pane workspace 복구됨: /old/path → $HOME" 표시 (3초 후 사라짐).

---

### §2.4 SPEC-V3-004: Render Layer Integration

#### 시안 매핑
- `.moai-canvas` grid layout
- `.moai-pane-head` (28px section header)

#### AC ↔ UI 매핑

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-R-1 | TabContainer GPUI render | (Entity<TabContainer> impl) | ✓ | — |
| AC-R-2 | PaneTree in-order iteration visible | (recursive HSplit/VSplit) | ✓ | — |
| AC-R-3 | Divider drag actual ratio update | (visual resize) | ✓ | — |
| AC-R-4 | Tab key dispatch → action | (keyboard routing) | ✓ | — |
| AC-R-5 | TabContainer + divider render integration | (escape hatch) | ⚠ | 실제 divider element 가 탭 전환 후에도 유지되는지 확인 |
| AC-R-6 | GPUI test-support 평가 | (headless render) | ⚠ | Defer → plan spike 3 (headless render feasibility) |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-004 revision instructions**:
> 1. `.moai-canvas`: CSS Grid `repeat(N, 1fr)` for panes, gap 1px (divider 색상).
> 2. `.moai-pane-head`: height 28px, padding 0 12px, font uppercase 11px weight 600, color fg-3, bg panel.
> 3. Pane 간 divider: 4px 최소, cursor col-resize (H) / row-resize (V).
> 4. Divider interaction: drag 시 ratio 평탄 업데이트, 최소 크기 도달 시 clamp.

---

### §2.5 SPEC-V3-005: File Explorer Surface

#### 시안 매핑
- `moai-surfaces.jsx` FileTree surface
- `moai-studio.html` .ftree / .frow classes

#### AC ↔ UI 매핑 (주요 12개)

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-FE-1 | FsNode tree 렌더 | .ftree .frow grid | ✓ | Row height 26px (comfortable) / 22px (compact) 검증 |
| AC-FE-2 | Lazy load 폴더 | chevron 10px rotate(90deg) | ✓ | Loading spinner (shimmer animation 1.6s) 추가 |
| AC-FE-3 | 파일 아이콘 | 14px icons per file type | ⚠ | Icon set 완성도 확인 (document, folder, code, image 등 최소 8종) |
| AC-FE-4 | Git status 배지 | Mono 10.5px, color per status | ✓ | M=#c47b2a / A=#1c7c70 / D=#c44a3a / U=#6a4cc7 |
| AC-FE-5 | 폴더 행 우클릭 컨텍스트 메뉴 | (context-menu) | ⚠ | Menu items: New File / New Folder / Rename / Delete / Reveal in Finder (5개) |
| AC-FE-6 | Drag-and-drop reorder | (visual feedback) | ⚠ | Drop target indicator (insert line 2px accent) + drag ghost (opacity 0.5) |
| AC-FE-7 | Fuzzy search 필터링 | Search box + filter results | ⚠ | Search box: height 28px, mono font, debounce 200ms |
| AC-FE-8 | 3 플랫폼 동작 | (macOS/Linux/Windows) | ⚠ | "Reveal in Finder" 텍스트는 platform per (macOS: Finder, Linux: File Manager, Windows: Explorer) |
| AC-FE-9 | Active row 하이라이트 | bg accent-soft | ✓ | — |
| AC-FE-10 | indent 계산 | 8px base + 14px per level | ✓ | — |
| AC-FE-11 | Empty state (workspace no files) | .empty mascot + CTA | ✓ | — |
| AC-FE-12 | Expand/collapse animation | slide 200ms + chevron rotate | ⚠ | Easing: cubic-bezier(0.4, 0, 0.2, 1) |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-005 revision instructions**:
> 1. File explorer 초기: root 디렉터리 자동 expanded 상태.
> 2. 각 row: grid `14px 14px 1fr 40px auto` (indent-chevron / icon / name / git-status / actions).
> 3. Git status 배지: mono 10.5px, 14×14px bg색, 텍스트색 white. 폴더는 자식 중 "가장 강한" 상태 roll-up (예: A+M → A로 표시).
> 4. Context menu: position absolute, bg surface, border, shadow md, radius 8px. 각 항목 height 28px, padding 0 12px.
> 5. Search box: height 28px, padding 0 12px, mono font 12.5px, placeholder "Search files...", debounce visual (spinner 나타났다 사라짐).
> 6. Drag-and-drop visual: drop zone에 2px accent 상하 insert line, drag ghost 40% opacity.
> 7. Lazy loading spinner: .shimmer animation (linear-gradient 좌→우, 1.6s infinite) 또는 dots animation.

---

### §2.6 SPEC-V3-006: Markdown/Code Viewer Surface

#### 시안 매핑
- `moai-surfaces.jsx` CodeViewer / MarkdownViewer surfaces
- `moai-studio.html` .code / .md / .md-gutter classes

#### AC ↔ UI 매핑 (주요 8개 — 전체는 SPEC에서 확인)

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-MD-1 | Code gutter (line numbers) | 44px width, right-align | ✓ | Gutter bg = panel color #0e1513 (dark) |
| AC-MD-2 | Syntax highlighting per scope | .kw/.st/.nu/.fn/.ty/.co etc | ✓ | 색상 tokens.json color.syntax 정확 매핑 필요 |
| AC-MD-3 | LSP diagnostic marker (gutter) | Amber ⚠ / Crimson ❌ | ✓ | Icon color = warning (#c47b2a) / danger (#c44a3a) |
| AC-MD-4 | Markdown max-width 780px | Container padding 28px 40px | ✓ | Line-height 1.75 (relaxed), h1 26px weight 800 |
| AC-MD-5 | Code block in markdown | Mono 12.5px, surface bg, border, radius 8px | ✓ | Padding 14px 16px, pre-wrap, syntax colors |
| AC-MD-6 | Blockquote styling | Border-left 3px accent, bg accent-soft 14% | ✓ | Padding 14px 16px, radius 0 6px 6px 0 |
| AC-MD-7 | Markdown MX gutter (NEW) | Grid 64px 1fr (gutter + content) | ⚠ | Gutter: 각 라인의 @MX:NOTE/WARN/ANCHOR/TODO 작은 tag (mono 9.5px, accent color) |
| AC-MD-8 | Virtual scroll (large files) | Container height constraint + scroll | ⚠ | Scroll bar width 8px, thumb bg accent-soft hover 액센트색 |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-006 revision instructions**:
> 1. Code viewer gutter: 44px width, mono 10.5px fg-3, padding 10px 8px 10px 0, right-aligned line numbers.
> 2. Gutter diagnostic icon: line-height center, icon 14px, color amber (#c47b2a) for warning, crimson (#c44a3a) for error.
> 3. Syntax scope → color 정확 매핑:
>    - .kw (keyword): dark #c792ea / light #5e3bb0
>    - .st (string): dark #88b780 / light #1c7c70 ✅ (신규 — #1c7c70 모두의AI color!)
>    - .nu (number): #c47b2a (동일)
>    - .fn (function): dark #4f9fce / light #155b8a
>    - .ty (type): #d4a45c
>    - .cm (comment): fg-3 italic
>    - .op (operator): #6fc2c2
> 4. Markdown max-width: 780px, margin 0 auto, padding 28px 40px.
> 5. MX gutter (신규): 좌측 64px 열, mono 9.5px accent color, 각 라인의 @MX 태그 표시 (예: "@MX:NOTE" 미니 라벨).
> 6. Code block: mono 12.5px, bg surface, border subtle, radius 8px, padding 14px 16px, line-height 1.6.
> 7. Blockquote: border-left 3px primary (#144a46), bg rgba(20,74,70,0.14), padding 14px 16px, radius 0 6px 6px 0.

---

### §2.7 SPEC-V3-008: Git Management UI

#### 시안 매핑
- `moai-surfaces.jsx` GitMgmt surface
- `moai-studio.html` .git classes

#### AC ↔ UI 매핑 (6개 — 전체는 SPEC에서 확인)

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-GM-1 | Git status panel | Row grid `auto 22px 1fr auto` (checkbox / badge / path / actions) | ⚠ | Checkbox 14×14px, border-strong 1.5px, checked: accent bg white ✓ |
| AC-GM-2 | Status badge M/A/D/U | Mono 10px weight 700, 18×18px square | ✓ | M=#c47b2a / A=#1c7c70 / D=#c44a3a / U=#6a4cc7 (정확히) |
| AC-GM-3 | Diff viewer 3-way merge | Mono 11.5px, hunk header bg surface, line: M/D color 10% bg | ⚠ | Delete line: crimson 10% bg + line-through, Add line: mint 10% bg + highlight |
| AC-GM-4 | Commit composer | Bg surface, border, radius 8px, textarea transparent, buttons: commit (accent bg bold) / cancel (secondary) | ⚠ | Textarea padding 12px, font 13px, subject section mono 10.5px fg-3 |
| AC-GM-5 | Branch switcher | Mono 12px, current branch bg accent-soft | ✓ | Row height 28px, ahead count mono 10.5px right-aligned |
| AC-GM-6 | Conflict resolution UI | 3-way diff + accept buttons (accept ours / accept theirs / accept both) | ✗ | 신규 — 3-way diff 레이아웃 명시 필요 |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-008 revision instructions**:
> 1. Status panel: section 헤더 uppercase 11px weight 600 tracking 8% fg-3, row height 28px.
> 2. Checkbox: 14×14px, border-strong 1.5px, radius 3px, checked state: accent bg white, rotated ✓ symbol.
> 3. Status badge: mono 10px weight 700, 18×18px square, radius 3px, bg = status color 12% (dark mode) / 14% (light), text white.
> 4. Diff viewer: hunk 헤더 bg surface border-top/bottom, line number 36px mono fg-3, symbol (+-~) center 16px, src 1fr.
> 5. Delete line: crimson 10% bg, text strikethrough optional.
> 6. Add line: mint 10% bg, 굵게 또는 다른 시각적 강조.
> 7. Commit composer: textarea height 80px, placeholder "Enter commit message...", subject field mono 10.5px fg-3 with divider border-top.
> 8. Buttons: commit = accent bg white text weight 600 height 32px radius 5px, cancel = border-strong bg transparent.

---

### §2.8 SPEC-V3-009: SPEC Management Kanban

#### 시안 매핑
- `moai-surfaces.jsx` SpecKanban surface
- `moai-studio.html` .spec kanban classes

#### AC ↔ UI 매핑 (5개)

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-SU-1 | 4-column kanban (draft/plan/dev/done) | CSS grid repeat(4, minmax(0,1fr)) gap 12px | ✓ | — |
| AC-SU-2 | Column header color per stage | Draft = fg-3 / Plan = cyan / Dev = accent / Done = mint | ✓ | Uppercase 11px weight 700 tracking 8% |
| AC-SU-3 | SPEC card layout | ID (mono 10px) + title (12.5px weight 600) + meta (10.5px fg-3) | ✓ | — |
| AC-SU-4 | AC pip indicator (14×4px pills) | Pass = mint / Fail = crimson / Pending = amber | ✓ | 3개 pill grid, radius 2px, tight layout |
| AC-SU-5 | Avatar (18×18px round) | Signature gradient bg, white initial 9px | ✓ | — |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-009 revision instructions**:
> 1. Kanban container: padding 16px, 4개 column, gap 12px.
> 2. Column: panel bg, border, radius 10px, padding 10px.
> 3. Column header: uppercase 11px weight 700 tracking 8%, color per stage (Draft #6b7370 / Plan #06B6D4 / Dev #144a46 / Done #1c7c70).
> 4. Card: surface bg, border, radius 7px, padding 10px 11px, gap 6px.
> 5. Card ID: mono 10px accent weight 600 tracking 4%.
> 6. AC pip row: 3 pills (14×4px), radius 2px, colors: ✅ mint / ❌ crimson / ⏳ amber.

---

### §2.9 SPEC-V3-010: Agent Dashboard

#### 시안 매핑
- `moai-surfaces.jsx` AgentDashboard surface
- `moai-studio.html` .ag 3-column grid classes

#### AC ↔ UI 매핑 (10개 중 6개)

| AC | 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|----|------|---------|------|---------|
| AC-AD-1 | 3-column layout (200/1fr/280) | Grid .ag-grid | ✓ | Column border-right: border-subtle (마지막 제외) |
| AC-AD-2 | Timeline 27 hook event | Event row grid `60px 18px 1fr auto` (time/icon/body/dur) | ⚠ | Timestamp mono 10.5px fg-3, icon 14px per event type color |
| AC-AD-3 | Cost bar chart | 60px height bars, 4px gap | ✓ | Bar color accent, opacity 0.85, current bar gradient |
| AC-AD-4 | Instructions tree (6 layers) | L1/L2/L3 indents (0/14/28px) | ✓ | Mono 11px line-height 1.7, size indicator float-right fg-3 10px |
| AC-AD-5 | Control buttons (pause/resume/kill) | Height 28px, border-strong, primary = accent bg white, danger = crimson text | ✓ | Radius 5px, weight 600 |
| AC-AD-6 | Event detail JSON viewer | Mono 10.5px, surface bg, border, white-space pre, radius 6px | ⚠ | Max-height 300px scrollable, syntax colors applied |

#### Claude Design 에게 보낼 구체 지시

> **SPEC-V3-010 revision instructions**:
> 1. Agent dashboard container: 3-column grid 200px 1fr 280px, padding 14px, gap 12px.
> 2. Column header (h3): mono 13px weight 700, sub 11px fg-3, stats 우측 (mono 13px uppercase 9.5px labels).
> 3. Timeline (center column):
>    - Event row: grid `60px 18px 1fr auto` (timestamp / icon / body / duration).
>    - Timestamp: mono 10.5px fg-3.
>    - Icon 14px: color per event type (tool=violet, ok=mint, err=crimson, msg=accent).
>    - Body: name fg-1 weight 500, detail mono 10.5px surface bg border radius 6px.
> 4. Cost panel (right column):
>    - Bar chart 60px height, 4px gap bars.
>    - Bar color: accent, opacity 0.85.
>    - Current bar: gradient 180deg (#5fdfb6 0% → accent 100%).
> 5. Instructions tree:
>    - Mono 11px line-height 1.7.
>    - L1 indent 0, L2 14px, L3 28px.
>    - Size label float-right fg-3 10px.
> 6. Control bar: flex row, buttons height 28px radius 5px border-strong.
>    - Primary: accent bg white text weight 600.
>    - Danger: crimson text, border-strong.

---

### §2.10 SPEC-V3-007 & WebBrowser Surface (미부분 읽음)

#### 추정 AC ↔ UI 매핑

| 기능 | 시안 반영 | 상태 | 수정 의뢰 |
|------|---------|------|---------|
| URL bar (주소 입력) | Address bar 28px | ⚠ | Input field, forward/back/refresh/stop buttons, 북마크 icon |
| History navigation | Back/Forward buttons | ⚠ | Button size 28px, disabled state (opacity 40%) |
| DevTools toggle | DevTools 아이콘 버튼 | ⚠ | Height 28px, bottom panel open 300px |
| Sandbox indicator | (lock icon or badge) | ⚠ | Secure/insecure SSL status 표시 |

---

## §3. Cross-cutting Issues (SPEC 한정 아님)

### 3.1 색상 일관성

**문제**: HTML/CSS 시안의 accent 색상이 이전 Tailwind blue (#2563EB) 일 수 있음.
**해결**: 모든 `.accent` 참조를 `#144a46` (또는 다크 모드 `#22938a`) 로 교체.
**검증**: design-canvas.jsx ACCENTS 객체 확인.

```javascript
// 수정 전
const ACCENTS = {
  default: "#2563EB",  // ❌
  blue: "#2563EB",
  // ...
};

// 수정 후
const ACCENTS = {
  default: "#144a46",  // ✅
  blue: "#2563EB",     // 여전히 옵션으로 유지
  // ...
};
```

### 3.2 한글 자간 적용 여부

**문제**: 모든 텍스트 영역에서 한글 자간(negative letter-spacing) 적용 여부 불명확.
**확인**: tokens.json의 letterSpacing 정의 사용.

```css
/* 필수 적용 */
h1, h2, h3: letter-spacing: -0.05em;
body text: letter-spacing: -0.025em;
body tight: letter-spacing: -0.05em;
caption: letter-spacing: 0;
```

### 3.3 Mascot (moai-logo-3.png) 등장 빈도

**사용처**:
- 헤더 sideBar 맨 위 (22px)
- FirstRun 모달 (64px)
- EmptyState (84px)
- Agent Dashboard 우측 패널 (선택적)

### 3.4 Accessibility (WCAG 2.1 AA)

**검증**:
1. ✅ Focus ring: 5px outline accent color
2. ⚠ Contrast: 모든 텍스트 대 배경 4.5:1 (light/dark 둘 다)
3. ⚠ prefers-reduced-motion: 모든 animation duration → 1ms (user preference 시)
4. ⚠ Keyboard navigation: 모든 interactive element tabindex >= 0

### 3.5 키보드 단축키 Cheat Sheet

**표시 위치**: ⌘+? (Help) 또는 Settings → Keyboard Shortcuts
**포함 항목**:
- Terminal: Cmd+T (new tab), Cmd+\\ (H-split), Cmd+Shift+\\ (V-split), Cmd+W (close)
- File Explorer: Cmd+P (quick open), Cmd+Shift+P (command palette)
- Editor: Cmd+F (find), Cmd+H (replace)

### 3.6 다국어 (한/영) 토글 UI

**위치**: TopBar 우측 theme toggle 옆
**상태**: 🌐 icon + "English" / "한국어" label
**동작**: Click → 전체 UI 언어 전환 (i18n)

### 3.7 Settings/Preferences 패널

**필요 항목**:
1. Theme (dark / light)
2. Density (comfortable / compact)
3. Accent color (teal / blue / violet / cyan)
4. Font size (12px / 14px / 16px)
5. Sidebar position (left / right)
6. Terminal font (JetBrains Mono / SF Mono / Custom)
7. Keyboard shortcuts
8. About

---

## §4. 신규 Surface + Modal 추가 의뢰 (SPEC AC 기반)

다음 9개 항목은 SPEC AC에서 도출되었으나 현재 시안에 없습니다. **Round 2에서 추가 의뢰**하는 것을 권장합니다.

### 4.1 Cmd+P File Quick Open Palette

**SPEC**: SPEC-V3-005 (또는 미래 SPEC-V3-011 Command Palette)
**목적**: 워크스페이스 내 파일 빠른 검색/열기
**UI**:
```
┌─────────────────────────────────────┐
│ 🔍 src/main.rs              [0/123] │  /* 입력 필드 + 결과 수 */
├─────────────────────────────────────┤
│ > src/main.rs         .rs 456 lines │  /* 우측 메타 */
│   src/lib.rs          .rs 234 lines │
│   src/config.rs       .rs  89 lines │
│ (더 많음…)                          │
└─────────────────────────────────────┘
```
**토큰**:
- Width: 600px max, centered
- Bg: surface, border, shadow lg
- Input: height 32px, mono 12.5px, padding 0 12px
- Result row: height 32px, padding 0 14px, hover bg accent-soft
- Meta: mono 10.5px fg-3, float-right

### 4.2 Cmd+Shift+P Command Palette

**목적**: 모든 가능한 명령 (settings, file operations, git, etc.)
**구조**: Cmd+P 와 동일하되, 카테고리 구분 섹션 추가
```
Recent Commands
  > File: New File              Cmd+N
  > View: Split Pane Horizontal Cmd+\\
  
All Commands
  > Settings: Open Preferences  Cmd+,
  > Git: Commit                 Cmd+K Cmd+C
  > …
```

### 4.3 Cmd+F Find / Replace in Code Viewer

**목적**: 열린 파일 내 텍스트 검색 + 바꾸기
**UI**:
```
┌──────────────────────────────────────┐
│ 🔍 function              [12 of 45] │ /* 검색 입력 + 매칭 수 */
│ 🔄 newFunction [Replace]             │ /* 바꾸기 입력 + 버튼 */
│ [Replace] [Replace All] [Close]      │ /* 액션 버튼 3개 */
└──────────────────────────────────────┘
```
**토큰**: Input height 32px, button height 28px, gap 8px

### 4.4 LSP Hover Popover

**목적**: 코드 에디터에서 마우스 hover → 타입 정보 + 문서 표시
**UI**:
```
┌─────────────────────────┐
│ func NewFile(…) error   │ /* Signature */
├─────────────────────────┤
│ Creates a new file in   │ /* Documentation */
│ the current workspace.  │
│ @param path string      │
└─────────────────────────┘
```
**토큰**: Max-width 400px, bg surface, border, shadow md, padding 12px 14px, radius 8px, mono 12px

### 4.5 MX Tag Click → Popover

**목적**: Markdown 또는 Code gutter의 @MX:NOTE/WARN/ANCHOR 클릭 시 상세 정보 표시
**UI**:
```
┌──────────────────────────────────────┐
│ @MX:ANCHOR                           │ /* 태그 제목 */
├──────────────────────────────────────┤
│ fan_in: 5 (high-risk area)           │ /* 메타 */
│ Body: This function is called from   │ /* 설명 */
│ 5 locations. Be careful with changes │
│ @MX:REASON: API contract            │
│                                      │
│ [View usages] [Open SPEC]            │ /* 액션 */
└──────────────────────────────────────┘
```
**토큰**: 400px max, shadow lg, padding 14px, radius 8px

### 4.6 Merge Conflict 3-way Diff

**목적**: Git status에서 conflict 파일 클릭 → 3-way diff 시각화 (base / ours / theirs)
**레이아웃**:
```
┌─────────────────────┬─────────────────────┬──────────────────────┐
│ Base (원본)         │ Ours (현재 브랜치)  │ Theirs (병합할 브랜치)│
├─────────────────────┼─────────────────────┼──────────────────────┤
│ function foo() {    │ function foo() {    │ function foo(x) {    │
│   return 42         │   return "42"       │   return 42 + x      │
│ }                   │ }                   │ }                    │
└─────────────────────┴─────────────────────┴──────────────────────┘
```
**토큰**: Mono 11.5px, hunk header fg-3, add line mint 10% bg, delete line crimson 10% bg

### 4.7 Sprint Contract Panel

**목적**: SPEC-V3-009 에서 "Run" 버튼 클릭 → Sprint Contract 자동 로드/표시
**UI**:
```
┌──────────────────────────────────────┐
│ Sprint v1.0.x · Review & Approve    │ /* 헤더 */
├──────────────────────────────────────┤
│ Priority dimension: Design Quality   │
│                                      │
│ Acceptance checklist:                │
│ ☐ Sidebar layout renders correctly   │
│ ☐ Colors match tokens.json           │
│ ☐ Light/dark theme both work        │
│ ☐ Accessibility: WCAG AA             │
│                                      │
│ Pass threshold: 0.75 / 1.0           │
└──────────────────────────────────────┘
```
**토큰**: Panel bg surface, width 320px, padding 14px, radius 10px

### 4.8 /moai * Slash Command Bar

**목적**: 사용자가 `/moai` 입력 → 자동완성 리스트 (plan, run, sync, etc.)
**UI**: Cmd+Shift+P 와 동일 스타일, 카테고리 "MoAI Commands"

### 4.9 Settings/Preferences Modal

**목적**: ⌘+, (설정) 클릭 → 전체 프로젝트 설정
**섹션**:
1. **Appearance**: Theme, Accent, Density, Font sizes
2. **Editor**: Font family, Line height, Tab size, Word wrap
3. **Terminal**: Font, Colors, Scrollback lines
4. **Git**: Author name, Email, Default branch
5. **Keyboard**: Shortcuts table (filterable)
6. **Extensions**: Installed extensions (future)
7. **About**: Version, Build date, License

---

## §5. State 별 보완 의뢰

시안에 있는 state (empty/skel/errbar/firstrun) 외에 다음을 추가 설계:

| State | UI Mockup | 토큰 | Priority |
|-------|-----------|------|----------|
| Crash recovery banner | "[!] Agent crashed. [Restart] [View Log]" | crimson 10% bg | P1 |
| Update available | "[↓] Update v0.2.0 available. [Install] [Later]" | info 10% bg | P1 |
| LSP server starting | Status: "LSP initializing…" (spinner) | accent color | P1 |
| PTY worker spawning | Status: "Terminal starting…" (dots 1.5s animation) | accent color | P1 |
| Workspace switching | Fade transition 200ms, mascot pulse | signature gradient | P2 |

---

## §6. Theme / Variant 보완

### Dark Theme (기본)

**검증할 것**: 모든 9 surface + 4 state가 dark 배경 (#0a100e) 에서 명확하게 보이는가?

### Light Theme (필수 추가)

**검증할 것**: 모든 surface를 light (#f3f3f3 bg) 에서 재설계. 특히:
- Code gutter bg: light 모드 시 #f3f3f3 (너무 밝으면 #fafaf9)
- Border color: dark #e6e6e3 (대비 확보)
- Text color: light #09110f (ink)

### Accent 4 Variants (각 surface별)

**적용 대상**: Button, accent line, highlight, indicator

| Accent | Light theme | Dark theme | 사용처 |
|--------|-----------|-----------|-------|
| teal (default) | #144a46 | #22938a | Primary action |
| blue | #2563EB | #60a5fa | Alternative |
| violet | #6a4cc7 | #c084fc | Status/category |
| cyan | #06B6D4 | #22d3ee | Info/notification |

### Compact Density Mode

**적용 대상**: 모든 row-based UI (file tree, timeline, etc.)

```css
/* Comfortable (기본) */
--row-h: 26px;
--pad-x: 12px;

/* Compact */
--row-h: 22px;
--pad-x: 8px;
```

---

## §7. Final Revision Prompt — Ready-to-Paste (Claude Design용)

다음 프롬프트를 그대로 claude.ai/design 의 기존 프로젝트에 복사해 붙이세요:

---

```
# MoAI Studio v3 — Design Revision Round 2

## 배경
SPEC-V3-001 ~ SPEC-V3-010 의 acceptance criteria (총 80+ AC) 를 이전 시안과 비교했습니다. 
대부분 반영되었으나 일부 gap 과 누락이 확인되었습니다. 본 revision 으로 완성도를 높이겠습니다.

## 절대 변경 금지 (FROZEN)
- Design token 색상: tokens.json v2.0.0 정확값
- 브랜드 signature gradient: linear-gradient(135deg, #144a46 0%, #09110f 100%)
- Pretendard 한글 자간: 음수 자간 필수 (display -7.5%, body -2.5%)

## P0 수정 의뢰 (즉시)

### 1. Color Audit
- [x] 시안의 모든 #2563EB (이전 Tailwind blue) → #144a46 (모두의AI 청록) 교체
- [x] 모든 accent 참조 검증 (ACCENTS 객체 확인)
- [x] Dark/light 테마 모든 색상 tokens.json 대조
- [ ] Light theme 모든 surface 재설계 (#f3f3f3 배경)

### 2. Typography 정합
- [x] Pretendard 9 weight self-hosted 확인
- [x] 한글 자간: display/body/caption 각 토큰값 적용
- [ ] 라이트 테마에서 contrast 4.5:1 검증

### 3. 치수 정확화
- [x] Tab bar: 36px 높이, 활성 탭 bold + background color 동시
- [x] Divider: 최소 4px, cursor col-resize/row-resize
- [x] Gutter: 44px (code viewer), 64px (markdown MX gutter)
- [ ] File explorer row: 26px comfortable / 22px compact 동시 제공

## P1 추가 의뢰 (Round 2)

### 신규 Surface 9개
1. File Quick Open (Cmd+P) — 600px palette, fuzzy match
2. Command Palette (Cmd+Shift+P) — 카테고리 섹션
3. Find/Replace (Cmd+F) — Code viewer 인라인
4. LSP Hover Popover — 마우스 hover 타입/문서
5. MX Tag Click Popover — body/fan_in/SPEC link
6. Merge Conflict 3-way Diff — base/ours/theirs 3열
7. Sprint Contract Panel — AC checklist + priority + pass threshold
8. /moai Slash Command Bar — autocomplete
9. Settings/Preferences Modal — appearance/editor/terminal/git/keyboard

### 신규 State 5개
1. Crash recovery banner — crimson 10% bg
2. Update available banner — info 10% bg
3. LSP server starting — spinner "initializing…"
4. PTY worker spawning — dots animation "starting…"
5. Workspace switching — fade 200ms transition

## 산출물 요청

1. 모든 9 surface (Terminal, FileExplorer, CodeViewer, Markdown, AgentDashboard, GitMgmt, SpecKanban, WebBrowser, AppShell) × dark/light × empty/loading/populated/error 각 screenshot
2. 추가 surface 9개 × dark/light mockup
3. 추가 state 5개 × dark/light mockup
4. Design specification (Inspect mode JSON):
   - 모든 컴포넌트의 정확한 색상값, font-size, padding, border-radius
   - Interactive state (hover/active/disabled/focus)
   - Animation duration/easing (특히 divider drag, tab switch, toast)
5. Accessibility checklist:
   - Focus ring 5px outline
   - WCAG 2.1 AA contrast 검증
   - prefers-reduced-motion 대응

## 우선순위
**P0 (즉시)** > P1 (이번) > P2 (향후)

## 토큰 reference (tokens.json v2.0.0에서 직접)
- Primary: #144a46 (OR #22938a dark)
- Success: #1c7c70
- Warning: #c47b2a
- Danger: #c44a3a
- Info: #2a8a8c
- Signature Gradient: linear-gradient(135deg, #144a46 0%, #09110f 100%)
```

---

## §8. Cross-Reference Index

### SPEC 파일
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-001/spec.md` — GPUI scaffold
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-002/spec.md` — Terminal Core
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-003/spec.md` — Tabs/Panes
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-004/spec.md` — Render Layer
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-005/spec.md` — File Explorer
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-006/spec.md` — Markdown/Code Viewer
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-008/spec.md` — Git UI
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-009/spec.md` — SPEC Kanban
- `/Users/goos/MoAI/moai-studio/.moai/specs/SPEC-V3-010/spec.md` — Agent Dashboard

### 토큰 파일
- `/Users/goos/MoAI/moai-studio/.moai/design/tokens.json` — v2.0.0 (절대 기준)
- `/Users/goos/MoAI/moai-studio/.moai/project/brand/visual-identity.md` — v2.0.0 (브랜드 가이드)

### 시안 파일
- `/Users/goos/MoAI/moai-studio/.moai/design/from-claude-design/` — 전체 핸드오프 번들
- `/Users/goos/MoAI/moai-studio/.moai/design/from-claude-design/project/moai-studio.html` — HTML 시안
- `/Users/goos/MoAI/moai-studio/.moai/design/from-claude-design/project/colors_and_type.css` — CSS 변수
- `/Users/goos/MoAI/moai-studio/.moai/design/from-claude-design/project/moai-app.jsx` — App shell
- `/Users/goos/MoAI/moai-studio/.moai/design/from-claude-design/project/moai-surfaces.jsx` — 9 surface
- `/Users/goos/MoAI/moai-studio/.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` — 12 섹션 구현 매핑

### 이 문서
- `/Users/goos/MoAI/moai-studio/.moai/design/handoff/07-revision-instructions.md` (본 파일)

---

## 사용 안내

1. 본 문서 전체를 claude.ai/design 의 기존 "moai-studio" 프로젝트에 복사.
2. **§7 Final Revision Prompt** 를 그대로 chat input 에 붙여넣기.
3. Claude Design이 P0 수정을 완료하면 export (`.zip` bundle).
4. P1 추가 의뢰는 별도 round 로 진행 가능.

---

Version: 1.0.0
Last Updated: 2026-04-25
Created By: MoAI Documentation Manager Expert
Reviewed Against: SPEC-V3-001 ~ SPEC-V3-010 AC (80+ criteria)
