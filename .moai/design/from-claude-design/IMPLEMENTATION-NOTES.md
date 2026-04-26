# Implementation Notes — Claude Design → GPUI Rust

> **Source bundle**: `.moai/design/from-claude-design/` (Claude Design 핸드오프, **Round 2 = v2 통합** 2026-04-26).
> **Target**: `crates/moai-studio-ui/` GPUI 0.2.2 Rust.
> **Goal**: HTML/CSS/JSX 시안을 GPUI 컴포넌트로 픽셀-단위 충실 재현. README 지시: "render in browser 금지, 소스 직독으로 토큰/구조 추출".

## Round 2 변경 요약 (2026-04-26)

Round 1 → Round 2 변경:
- **moai-studio.html**: 839 → 1442 lines (+603 신규 CSS — palette/popover/modal/banner)
- **moai-revisions.jsx** 신규 (640 lines, 18 component)
- **chat1.md**: 173 → 319 lines (Round 2 conversation transcript)
- **colors_and_type.css**: 동일 (토큰 정합 유지)
- **uploads/**: revision-instructions.md 가 Claude Design 에 입력됨 확인

Round 2 신규 컴포넌트 (18) — `tokens.json round2_component` 섹션 참조:
1. **Palette** — Cmd+P (CmdPalette), Cmd+Shift+P (CommandPalette), /moai (SlashBar) — 600px width, 32px row
2. **FindReplace** (.find) — Code viewer 인라인 toolbar, 26px ibtn
3. **LspHover** (.lspop) — type signature + doc preview, 420px width
4. **MXPopover** — body / fan_in / SPEC link, 360px, 4 색 (ANCHOR gold / WARN amber / NOTE info / TODO violet)
5. **MergeDiff** — 3-way conflict (base / ours / theirs)
6. **SprintPanel** — SPEC-V3-009 AC checklist sidecar, 320px
7. **SettingsModal** — 6 sections (Appearance/Keyboard/Editor/Terminal/Agent/Advanced), 880×640
8. **CrashBanner / UpdateBanner** — 영구 banner, 40-48px height
9. **LspStarting / PtyStarting** — transient state banner
10. **WorkspaceSwitching** — workspace 전환 transition
11. **CodeStub / AppearancePane / KeyboardPane** — 보조 컴포넌트

---

## 0. Bundle 자료 인덱스

| 파일 | 역할 |
|------|------|
| `README.md` | Coding agent 지침 (chat transcript 우선, HTML imports follow, no screenshots) |
| `chats/chat1.md` | 디자인 결정 transcript — color_system / scope / theme / states / variation / accent default |
| `project/moai-studio.html` | 메인 entry — 모든 surface 통합 시안 + CSS variables |
| `project/colors_and_type.css` | Canonical CSS 변수 — `tokens.json` 와 동기화됨 |
| `project/moai-app.jsx` | App shell + Sidebar + Surface wrapper + ACCENTS 객체 |
| `project/moai-surfaces.jsx` | Terminal / CodeViewer / MarkdownViewer / AgentDashboard / GitMgmt / SpecKanban / WebBrowser |
| `project/moai-states.jsx` | EmptyState / LoadingSkeleton / ErrorBanner / FirstRun |
| `project/moai-icons.jsx` | SVG 아이콘 셋 |
| `project/design-canvas.jsx` | Tweaks 패널 + 다중 artboard 디자인 캔버스 |
| `project/tweaks-panel.jsx` | Theme/density/accent/sidebar/paneLayout/agentSlot 토글 |
| `project/fonts/Pretendard-*.otf` | 9 weight self-hosted 폰트 |
| `project/assets/moai-logo-*.png` | 마스코트 + wordmark (4종) |

---

## 1. App Shell — `crates/moai-studio-ui/src/lib.rs` (RootView)

### CSS 시안 → GPUI 매핑

| CSS class | Rust 대응 | dimension |
|-----------|----------|-----------|
| `.moai-top` | `topbar()` div | h: 38px, panel bg, border-bottom |
| `.moai-traffic` | (macOS only — native window controls 사용) | — |
| `.moai-cmdbar` | `cmdbar()` div | h: 26px, surface bg, border, radius 6px, max-w 420px, centered |
| `.moai-agent-pill` | `agent_pill()` (Entity<AgentDashboard> 연동) | h: 22px, pill, signature gradient bg, dot pulse 1.4s |
| `.moai-side` | `sidebar()` div | w: 240px (compact 56px), panel bg |
| `.moai-brand` | `brand_header()` | h: 44px, mascot 22px, name 13px bold |
| `.moai-nav-item.active` | active state | bg: accent-soft, inset 2px 0 0 accent (left rail) |
| `.moai-tabs` | (V3-003 기존 구현 — bar.rs 토큰 정합 필요) | h: 36px |
| `.moai-tab.active` | (기존 구현 강화) | bg: app bg, **inset 0 2px 0 accent (top rail)** |
| `.moai-canvas` / `.moai-panes` | (V3-003 panes/render.rs 정합) | grid gap 1px (border-색) |
| `.moai-pane-head` | pane heading | h: 28px, panel bg, uppercase 11px label |
| `.moai-status` | status bar | h: 24px, panel bg, mono 10.5px, left+right 영역 |

### Pulse 애니메이션 (agent dot)
```css
@keyframes pulse {
  0%,100% { box-shadow: 0 0 0 0 rgba(95,223,182,0.4); }
  50%     { box-shadow: 0 0 0 5px rgba(95,223,182,0); }
}
```
GPUI: animation infra 가 limited 이므로 `cx.observe_animation` 으로 1.4s loop, 2-step shadow 보간.

---

## 2. Terminal — `crates/moai-studio-ui/src/terminal/`

### CSS class `.term`
- font: mono 12.5px, line-height 1.5
- padding: 10px 14px
- bg: var(--bg)
- prompt `.pr` color: var(--accent) (`#22938a` dark / `#144a46` light)
- semantic colors: `.ok` mint / `.er` crimson / `.wr` amber / `.cm` fg-3 / `.nm` violet / `.br` cyan
- cursor `.cur`: 7×14px, accent bg, blink 1s steps(2) infinite

### 변경 작업 (현재 구현 정합)
- 기존 TerminalSurface 의 prompt 색상이 `#3B82F6` (이전 토큰) 였다면 → `#144a46` (or `#22938a` dark) 로 교체
- 의미 색상 모두 `tokens.json color.ide_accent.{mint, crimson, amber, violet, cyan}` 매핑

---

## 3. File Explorer — `crates/moai-studio-ui/src/explorer/`

### CSS class `.ftree` / `.frow`
- row height: var(--row-h) = 26px (comfortable) / 22px (compact)
- indent: 14px per level (`calc(8px + var(--lvl,0) * 14px)`)
- chevron: 10px, transform rotate(90deg) on `.open`
- icon: 14px (file type 별)
- name: ellipsis, 12px
- git status `.gs`: mono 10.5px, 14px col, color per status:
  - `.M` amber `#c47b2a` / `.A` mint `#1c7c70` / `.D` crimson `#c44a3a` / `.U` violet `#6a4cc7` / `.Q` fg-3
- active row: bg `var(--accent-soft)`, color fg-1

### 변경 작업
- V3-005 explorer/view.rs 의 row 렌더 시 위 토큰 적용
- ChildState::Loading → spinner 14px (CSS `@keyframes shimmer` 동일 매핑)

---

## 4. Code Viewer — `crates/moai-studio-ui/src/viewer/code/`

### CSS class `.code` / `.code .gut` / `.code .src`
- gutter: 44px width, right-align, fg-3, padding 10px 8px 10px 0
- gutter `.diag`: amber (LSP 진단 marker)
- src: padding 10px 14px, white-space: pre, mono 12.5px, line-height 1.55
- syntax classes (dark / light 분기):
  - `.kw` keyword: `#c792ea` / `#5e3bb0`
  - `.st` string: `#88b780` / `#1c7c70`
  - `.nu` number: `#c47b2a` (동일)
  - `.fn` function: `#4f9fce` / `#155b8a`
  - `.ty` type: `#d4a45c`
  - `.co` comment: fg-3 italic
  - `.op` operator: `#6fc2c2`
  - `.va` variable: fg-1
- diagnostic line bg: `rgba(196,74,58,0.10)` (.diag-bg)

### 변경 작업
- V3-006 viewer/code/highlight.rs 의 `scope_to_color` 가 위 hex 정확 매핑
- gutter 좌측에 LSP 진단 ⚠/❌ 아이콘 + line number

---

## 5. Markdown Viewer — `crates/moai-studio-ui/src/viewer/markdown/`

### CSS class `.md` / `.md-frame`
- max-width: 780px, margin: 0 auto
- padding: 28px 40px
- font-size: 14px, line-height: 1.75 (relaxed), letter-spacing -0.005em
- h1: 26px / weight 800 / -4% tracking
- h2: 18px / weight 700 / -2% tracking / border-top
- h3: 14.5px / weight 700 / -1% tracking
- code (inline): mono 12.5px, surface bg, padding 1px 6px, radius 4px
- pre: mono 12px, line-height 1.6, surface bg, border, padding 14px 16px, radius 8px
- blockquote: border-left 3px accent, accent-soft bg, radius 0 6px 6px 0

### MX gutter — **NEW UI 패턴**
- `.md-frame`: grid `64px 1fr` (gutter + content)
- `.md-gutter`: panel bg, padding 28px 8px 0 12px, border-right
- `.md-gutter .gtag`: mono 9.5px, accent color, right-aligned, mb 18px
- 각 라인의 @MX:NOTE/WARN/ANCHOR/TODO 가 좌측 거터에 작은 mono tag 로 표시

### `.md .mx-tag` (인라인 태그 pill)
- inline-flex h: 18px, padding 0 7px
- bg: accent-soft, color: accent, border 1px primary/0.2
- mono 10.5px weight 600
- radius: 999px (full pill)

### 변경 작업
- V3-006 markdown/mod.rs: `.md` container 토큰 정합 (max-w 780, padding, type scale)
- MS-3 `mx_scan.rs` + `gutter.rs` (T21/T22) 가 본 시안 매칭
- `mx-tag` pill 컴포넌트 신규 (V3-006 MS-3 진입 시)

---

## 6. Agent Dashboard — `crates/moai-studio-ui/src/agent/`

### CSS class `.ag` (3-col grid 200/1fr/280)
- `.ag-head`: 12-16px padding, agent title 13px weight 700, sub 11px fg-3, **stats 우측** (mono 13px values, uppercase 9.5px labels)
- `.ag-grid`: `200px 1fr 280px` (filter / timeline / side panels)
- `.ag-col`: border-right (마지막 열 제외)

### Filters column (`.ag-filters` / `.ag-chip`)
- chip h: 26px, radius 4px, 11.5px text
- `.ag-chip.on`: accent-soft bg, fg-1
- `.ag-chip .sw`: 8×8px sq, color per event type
- `.ag-chip .cnt`: mono 10px

### Timeline (`.ag-tl` / `.ag-ev`)
- event row: grid `60px 18px 1fr auto` (time / icon / body / dur)
- timestamp `.ts`: mono 10.5px fg-3
- icon `.icc` 14px, color per event class:
  - `.tool` violet / `.ok` mint / `.err` crimson / `.msg` accent
- body name: fg-1 weight 500
- expanded detail: mono 10.5px, surface bg, border, radius 6px, white-space pre
- 16ms throttle (60fps event burst)

### Side cards (`.ag-card`)
- bg surface, border, radius 8px, padding 12-14px
- h4: uppercase 10.5px fg-3 weight 600

### Cost bar chart (`.ag-bar`)
- 60px height, 4px gap
- bars: accent fg, opacity 0.85
- current bar: `linear-gradient(180deg, #5fdfb6 0%, accent 100%)`

### Tree view (`.ag-tree`)
- mono 11px line-height 1.7
- l1/l2/l3 indents (0/14/28px)
- size `.sz`: float-right fg-3 10px

### Control buttons (`.ag-ctrl` / `.ag-btn`)
- h: 28px, radius 5px, border strong
- `.primary`: accent bg, white text
- `.danger`: crimson text

### 변경 작업
- V3-010 agent/dashboard_view.rs 가 위 3-col 레이아웃 + 토큰 정확 적용
- MS-2 cost panel: `ag-card` + `ag-bar` 매칭
- MS-3 instructions graph: `ag-tree` 매칭
- MS-3 control bar: `ag-ctrl` + `.primary`/`.danger` 변형

---

## 7. Git Management — `crates/moai-studio-ui/src/git/` (V3-008 신규)

### CSS class `.git`
- container padding 14px 16px
- h3 section: uppercase 11px tracking 8% fg-3 weight 600
- row grid: `auto 22px 1fr auto auto` (checkbox / status badge / path / date / actions)
- checkbox `.ck`: 14×14px, border-strong 1.5px, radius 3px
  - `.ck.on`: accent bg, white check (rotated border)
- status `.st`: mono 10px weight 700, 18px sq, radius 3px, bg per status:
  - `.M` amber 12% bg / `.A` mint 14% bg / `.D` crimson 12% bg / `.U` violet 14% bg / `.Q` surface
- path `.pa`: mono 11.5px fg-2, ellipsis (`b` strong = fg-1)

### Commit composer (`.commitbox`)
- bg surface, border, radius 8px, padding 12px
- textarea: transparent, no border, 13px
- subject (`.sub`): 12px fg-3, border-top divider
- actions: flex, commit-btn accent, weight 600, padding 7-14, radius 5

### Branch row (`.branch-row`)
- 6-10px padding, mono name 12px
- `.cur`: accent-soft bg
- `.ahead`: mono 10.5px fg-3 right-aligned

### Diff viewer (`.gitdiff`)
- mono 11.5px line-height 1.55
- hunk: surface bg, border-top/bottom, padding 2-16px, fg-3
- line: grid `36px 36px 16px 1fr` (lineNo old / new / sym / src)
  - `.add` mint 10% bg / `.del` crimson 10% bg
  - `.sym` center-aligned

### 변경 작업
- V3-008 SPEC implement 시 위 토큰 정확 적용
- existing moai-git crate API → row 렌더

---

## 8. SPEC Management (Kanban) — `crates/moai-studio-ui/src/spec/` (V3-009 신규)

### CSS class `.spec`
- 4-column kanban: `repeat(4, minmax(0,1fr))` gap 12px
- col: panel bg, border, radius 10px, padding 10px
- col h4: uppercase 11px tracking 8% weight 700, color per stage:
  - `.draft` fg-3 / `.plan` cyan / `.dev` accent / `.done` mint
- card `.card`: surface bg, border, radius 7px, padding 10-11px
  - id: mono 10px accent weight 600 tracking 4%
  - title: 12.5px weight 600 line-height 1.35
  - meta: 10.5px fg-3, gap 8px
  - **AC pip indicator** `.ac .pip`: 14×4px radius 2px (pass/fail/pending pills)
    - `.r` crimson / `.g` mint / `.y` amber
  - avatar `.av`: 18×18 round, signature gradient bg, white initial 9px
  - tag: mono 9.5px, accent-soft bg accent fg, radius 2px

### 변경 작업
- V3-009 SPEC implement 시 본 카드 + Kanban 매칭
- AC pip 가 spec.md AC 상태 자동 시각화 (FULL=g / FAIL=r / PARTIAL=y)

---

## 9. Empty / Loading / Error / FirstRun

### Empty (`.empty`)
- center column, padding 32px
- mascot img 84px (moai-logo-3.png)
- h3 17px weight 700 / p 12.5px fg-3 max-w 360px
- hint kbd: mono 10.5px, surface bg border, radius 3px
- pri-btn: 32px h, accent bg, weight 600, radius 5px

### Loading skeleton (`.skel`)
- shimmer: `linear-gradient(90deg, surface 0%, elev 50%, surface 100%)`, animation 1.6s linear

### Error banner (`.errbar`)
- bg crimson 10%, border crimson 30%, padding 10-16px
- icon 16px crimson
- buttons: `.pri` crimson bg white text, secondary border-strong

### FirstRun (`.firstrun`)
- radial gradient bg: `radial-gradient(circle at 30% 20%, primary/0.18 0%, transparent 60%) + bg`
- panel: 640px max, padding 28-32px, radius 14, shadow lg
- mascot 64px
- h2 22px weight 800
- step rows: grid `28px 1fr auto`, surface bg radius 8, padding 12-14
  - step.n: 24×24 round, accent-soft bg accent fg
  - step.active.n: accent bg white
  - step.done.n: mint bg white
- pri button: signature gradient bg, signature shadow, h 38px

### 변경 작업
- V3-005 explorer empty / V3-006 viewer empty / V3-010 agent empty 모두 본 시안 정합
- LoadingSkeleton 컴포넌트 신규 추가 (재사용)
- ErrorBanner 컴포넌트 신규 (워크스페이스 레벨)
- FirstRun 컴포넌트 신규 (V3-005 first-run flow)

---

## 10. Tweaks (개발 중 조정 패널 — production 비포함)

`tweaks-panel.jsx` 의 토글:
- theme: dark / light
- density: comfortable / compact
- accent: teal (default) / blue / violet / cyan
- sidebarSide: left / right
- paneLayout: 1 / 2 / 3 / 4
- agentSlot: right / bottom / tab

production GPUI 앱에서는 settings menu 또는 user-config 로 별도 구현. 시안 단계에서는 `--accent` CSS variable 만 노출하는 패턴.

---

## 11. Implementation 작업 순서 (권장)

| 단계 | 범위 | SPEC |
|------|-----|------|
| 1 | tokens.json v2.0 → GPUI Rust 상수 모듈 (`src/design/tokens.rs` 신규) | (chore) |
| 2 | 기존 V3-003/V3-004/V3-005/V3-006 산출물 색상 토큰 교체 (`#2563EB` → `#144a46`) | (chore) |
| 3 | App shell — topbar / sidebar / status bar / agent pill | (V3-004 확장 또는 신규 SPEC) |
| 4 | Pretendard self-hosted asset embed (Cargo asset_resources / build.rs) | (chore) |
| 5 | Mascot 로고 RootView 사이드바 + FirstRun 사용 | (V3-004 확장) |
| 6 | mx-tag pill + md gutter (V3-006 MS-3) | V3-006 |
| 7 | Agent dashboard 3-col grid 정합 | V3-010 MS-2/3 |
| 8 | Git UI 신규 | V3-008 implement |
| 9 | SPEC Kanban + AC pip | V3-009 implement |
| 10 | FirstRun / EmptyState / ErrorBanner / LoadingSkeleton | (cross-surface chore) |

---

## 12. Cross-References

- **canonical tokens**: `.moai/design/tokens.json` v2.0.0
- **brand identity doc**: `.moai/project/brand/visual-identity.md` v2.0.0
- **handoff bundle (Path A)**: `.moai/design/handoff/` (20 files, design 의뢰용)
- **이 implementation guide**: `.moai/design/from-claude-design/IMPLEMENTATION-NOTES.md` (이 파일)

---

Version: 1.0.0
Last Updated: 2026-04-25
Bundle Origin: `https://api.anthropic.com/v1/design/h/Soj3DRdFBF68x3X61_YtJQ`

---

## 13. Round 2 신규 컴포넌트 — GPUI Rust 매핑 가이드

### 13.1 Palette (Cmd+P / Cmd+Shift+P / /moai *) — `crates/moai-studio-ui/src/palette/` (신규)

**CSS class `.pal`** (overlay scrim `.ovl-scrim` 위에 mount).

| CSS | GPUI 매핑 |
|-----|-----------|
| `.ovl-scrim` | `Scrim` 컴포넌트 — `Entity<Scrim>` 또는 modal layer |
| `.pal` (600px width, radius 10px, shadow palette) | `PaletteView` Entity, `impl Render` |
| `.pal-input` (44px height) | input row with 14px icon + mono 13px input |
| `.pal-section` (uppercase 9.5px, letter-spacing 0.10em) | section header |
| `.pal-list` (max-height 320px, scrollable) | virtualized list |
| `.pal-row` (32px height, grid 16/1fr/auto) | row: icon / name / kbd shortcut |
| `.pal-row.sel` (accent-soft bg, inset 2px accent) | keyboard-selected state |
| `.pal-row .pal-nm em` | fuzzy match highlight (accent-soft bg, accent text) |
| `.pal-foot` (28px height) | footer with hints (`↑↓ Navigate`, `Enter Open`, `Esc Close`) |

**Action**: 3 variant entry points share PaletteView struct with `mode: PaletteMode::FileQuickOpen | CommandPalette | SlashBar`.

### 13.2 Find/Replace (Cmd+F) — Code Viewer 인라인

**CSS class `.find`** (Code viewer 우측 상단 absolute).

| CSS | GPUI 매핑 |
|-----|-----------|
| `.find` (absolute top-right of `.code .src`) | mount overlay above CodeViewer body |
| `.find .row` (input + count + ibtn group) | search row + replace row |
| `.find input` (26px height) | input element with focus border accent |
| `.find .cnt` (mono 10.5px) | "3 of 12" match count |
| `.find .ibtn` (icon button — Aa, .*, |abc|) | toggle: case-sensitive / regex / whole-word |
| `.find .ibtn.on` (accent-soft bg, primary border) | active toggle |
| `.find .ibtn.close` (margin-left auto) | close X button |
| `.find .actrow` (action row — replace/replace-all) | replace mode actions |
| `.find .btn.pri` (accent bg, weight 600) | primary action button |

**Action**: V3-006 MS-3 진입 시 추가. CodeViewer 내부 child Entity 로 mount/unmount.

### 13.3 LSP Hover Popover — Code Viewer

**CSS class `.lspop`** (cursor anchor 기반 absolute position).

| CSS | GPUI 매핑 |
|-----|-----------|
| `.lspop` (420px width, max 320 height, padding 14-16) | `LspHoverPopover` Entity |
| `.lspop .sig` (mono 12.5px, syntax highlight) | function signature with .kw/.fn/.ty class |
| `.lspop .doc` (12px, fg-2) | rust-analyzer markdown rendering |
| `.lspop .doc .pdef` (parameter definitions) | param row: name (accent) + desc (fg-3) |
| `.lspop .foot kbd` (mono kbd hint) | "Cmd+Click to go-to-def" |
| `.lspop .ed-anchor` | viewport edge offset 8px |

**Action**: V3-006 MS-3 LSP 통합 시. async-lsp + lsp-types `textDocument/hover` 응답 → popover render.

### 13.4 MX Tag Click Popover — Markdown/Code Viewer

| CSS | GPUI 매핑 |
|-----|-----------|
| `.mxpop` (360px, padding 12-14, radius 8) | `MxTagPopover` Entity |
| Tag color: ANCHOR `#d4a017` gold ★ / WARN `#c47b2a` amber ⚠ / NOTE `#2a8a8c` info ● / TODO `#6a4cc7` violet ◇ |
| Body text + REASON + fan_in count + SPEC link | structured popover content |

**Action**: V3-006 MS-3 mx_scan + gutter T21/T22 진입 시.

### 13.5 3-way Merge Diff — Git UI

**CSS class `.mergediff` / `.merge-cols` / `.merge-pane`**.

| 영역 | 색상 |
|-----|------|
| Base (좌) | text.tertiary |
| Ours (중) | semantic.success (mint) |
| Theirs (우) | brand.primary.dark (accent) |
| Conflict line | semantic.danger (crimson) |

**Action**: V3-008 MS-3 merge conflict resolver. 3-pane grid + accept ours/theirs/manual buttons.

### 13.6 Sprint Contract Panel — SPEC UI

**CSS class `.sprint-panel`** — SPEC detail view 의 우측 sidecar (320px).

| 영역 | 내용 |
|-----|------|
| Header | "Sprint Contract v1.0.x" + revision badge |
| AC checklist | 14px checkbox + 26px row + AC label + 상태 dot |
| Sprint Exit progress | bar chart — pass/fail/pending |
| Hard thresholds 표 | coverage / lsp / clippy / fmt / regression |

**Action**: V3-009 MS-2/MS-3 implement 시.

### 13.7 Settings/Preferences Modal

**CSS class `.settings-modal` / `.settings-sidebar` / `.settings-pane`**.

| Section | 내용 |
|---------|------|
| Appearance | theme (dark/light/system) / density / accent (teal/blue/violet/cyan) / font size |
| Keyboard | binding 테이블 + edit |
| Editor | tab size / word wrap / minimap / format on save |
| Terminal | shell / font / scrollback / opacity |
| Agent | model / cost limit / auto-approve / hook config |
| Advanced | LSP servers / tree-sitter languages / experimental flags |

**Action**: 신규 `crates/moai-studio-ui/src/settings/` 모듈. UserSettings struct + SettingsModal Entity.

### 13.8 Banners (Crash / Update / LSP starting / PTY starting / Workspace switching)

| Banner | 색상 | 위치 |
|--------|-----|------|
| Crash recovery | semantic.danger | top of app, dismissable + restore CTA |
| Update available | brand.primary | top, install/dismiss |
| LSP starting | semantic.info | bottom of CodeViewer, transient |
| PTY starting | semantic.success | bottom of Terminal, transient |
| Workspace switching | brand.primary.dark | center overlay during transition |

**Action**: 신규 `crates/moai-studio-ui/src/banners/` 모듈. Banner Entity + BannerKind enum + auto-dismiss timer.

### 13.9 Backdrop Scrim — modal/palette 공용

| 속성 | 값 |
|-----|------|
| position | absolute inset 0 |
| dark bg | rgba(8,12,11,0.55) |
| light bg | rgba(20,30,28,0.18) |
| backdrop-filter blur | 2px |
| z-index | 20 |
| padding-top | 80px (palette 위치 기준점) |

**Action**: 공용 Scrim Entity. PaletteView / SettingsModal / MergeDiff 가 mount 시 emit.

---

## 14. Round 2 후속 Implementation 작업 권장 순서

| 단계 | 작업 | SPEC | 우선도 |
|------|-----|------|------|
| A | 토큰 → GPUI Rust 상수 (`src/design/tokens.rs`) Round 2 dimension 포함 | chore | P0 |
| B | 기존 surface 색상 일괄 교체 (`#2563EB` → `#144a46` repo-wide grep) | chore | P0 |
| C | Scrim + PaletteView (Cmd+P 우선) | 신규 `palette/` | P0 |
| D | Settings modal | 신규 `settings/` | P0 |
| E | Find/Replace + LSP Hover + MX Popover | V3-006 MS-3 | P1 |
| F | Banners (Crash/Update/LSP/PTY/Workspace) | 신규 `banners/` | P1 |
| G | Sprint Contract Panel | V3-009 implement 시 | P1 |
| H | 3-way Merge Diff | V3-008 MS-3 | P1 |

---

Version: 1.1.0 (Round 2 통합)
Last Updated: 2026-04-26
Bundle Origin: `https://api.anthropic.com/v1/design/h/MIrSZ25-Uvr1x44EiPyeOg` (Round 2)
