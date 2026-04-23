# MoAI Studio Design System (2026-04-17)

MoAI Studio 디자인 시스템 정의. `moai-termail.pen` 에서 추출한 디자인 토큰 + 리서치 기반 보강. `.claude/skills/moai-design-craft/` 프로토콜 호환.

---

## 1. Design Intent

MoAI Studio 는 **"SPEC-First DDD 를 위한 네이티브 coding shell"** 이다.

핵심 정체성 3축:
- **Native**: macOS 관례 준수 (메뉴 바 · 툴바 · 단축키 · MenuBarExtra). Electron 래퍼 NOT.
- **Agentic**: 에이전트가 1급 객체. 사이드바 / 팔레트 / 대시보드에 Agent Run 이 항상 가시.
- **SPEC-driven**: Plan → Run → Sync 파이프라인을 UI 로 가시화. TRUST 5 게이트를 상태바/대시보드에 노출.

Design feel: **calm · deliberate · console-native**. Cursor 의 "cloud dashboard" 나 Zed 의 "pure editor" 와 구별되는 **"SPEC + Terminal + Agent Run" 3단 네이티브 셸**.

---

## 2. Domain Vocabulary

| Term | Definition |
|------|-----------|
| Workspace | 사용자 프로젝트 1개. 고유 projectPath + Ghostty terminal session + Pane tree 보유 |
| Pane | Workspace 내의 binary tree 분할 영역. leaf 또는 split 노드 |
| Surface | Pane 안에 로드되는 콘텐츠 타입 (terminal/markdown/filetree/browser/image/code-viewer/agent-run) |
| Tab | Pane 내부의 탭 인스턴스. Surface 1개와 연결 |
| SPEC | EARS 포맷 요구사항 문서. `.moai/specs/SPEC-XXX/` 디렉토리 1개 = 1 SPEC |
| Plan / Run / Sync | SPEC 워크플로우 3단계 |
| Agent Run | 1회 에이전트 실행 세션. hook 이벤트 스트림 + cost + tokens 로 구성 |
| Hook Event | 에이전트 실행 중 발생한 27개 이벤트 타입 (SessionStart/PreToolUse/PostToolUse/…) |
| MX Tag | 코드 레벨 AI 주석 (@MX:NOTE/WARN/ANCHOR/TODO) |
| TRUST 5 | 품질 게이트 5축 (Tested · Readable · Unified · Secured · Trackable) |
| Worktree | git worktree 기반 SPEC 격리 작업 공간 |
| Context Burst | 결정적 액션 직전의 고밀도 컨텍스트 주입 |

---

## 3. Craft Principles

1. **Empty state 는 금지**. 항상 CTA 버튼 + 가이드 제공.
2. **단축키 = 메뉴 아이템**. 모든 단축키는 메뉴 바 MenuItem 과 1:1 매핑.
3. **Agent Run 은 항상 가시**. 최소 상태 pill (상태바) + 필요 시 우측 패널.
4. **SPEC 문맥은 한 번만 입력**. 메뉴/단축키 어디서든 동일 SPEC-ID 유지.
5. **Destructive action 은 2단계 확인**. 워크스페이스 삭제 / worktree 제거 / --force push.
6. **Progressive disclosure**. 고급 기능은 ⌘⇧P 에서 노출, 기본 메뉴엔 빈도 높은 것만.
7. **색 단독 정보 전달 금지**. 상태/진단은 반드시 아이콘 병용.
8. **Placeholder 텍스트는 실제 동작하지 않는 기능에 금지**. 안 되는 것은 enabled=false 로 명확히.

---

## 4. Color Tokens

### Dark (primary theme)

```
$bg-base            #0a0a0b   # 전체 배경
$bg-surface         #131315   # 카드 / 사이드바 배경
$bg-surface-2       #1b1b1e   # 2차 카드 / hover
$bg-surface-3       #232327   # 활성 / selected

$fg-primary         #f4f4f5   # 타이틀 / 강조
$fg-secondary       #b5b5bb   # 본문
$fg-muted           #6b6b73   # 캡션 / 메타
$fg-dim             #3f3f46   # 구분자 / 비활성

$border-subtle      #2a2a2e   # 기본 경계
$border-strong      #3a3a40   # 강조 경계 (모달 등)

$accent-moai        #FF6A3D   # MoAI 브랜드 오렌지
$accent-moai-dim    #FF6A3D22 # 10-15% 알파 배경

$status-success     #22c55e
$status-warning     #eab308
$status-danger      #ef4444
$status-info        #3b82f6

$traffic-red        #FF5F57
$traffic-yellow     #FEBC2E
$traffic-green      #28C840
```

### Light (secondary)

```
$bg-base-light      #FAFAFB
$bg-surface-light   #FFFFFF
$bg-surface-2-light #F4F4F5
$fg-primary-light   #0a0a0b
$fg-muted-light     #71717a
$border-light       #E4E4E7
```

### Syntax (Code Viewer)

```
$syntax-keyword     #C084FC
$syntax-type        #60A5FA
$syntax-string      #4ADE80
$syntax-comment     #6B7280
$syntax-function    #FBBF24
$syntax-number      #F472B6
$syntax-operator    #94A3B8
```

### Diagnostics (LSP gutter)

```
$diag-error         #ef4444
$diag-warning       #eab308
$diag-info          #3b82f6
$diag-hint          #10b981
```

---

## 5. Typography

### Font Families

```
$font-sans   "Inter" → "SF Pro Display" → system-ui
$font-mono   "Geist Mono" → "SF Mono" → "Menlo" → monospace
```

### Type Scale

| 토큰 | 크기 | 줄높이 | 자간 | 용도 |
|------|------|--------|------|------|
| `$text-micro` | 9 | 1.25 | +1.5 | ALL-CAPS 섹션 라벨 |
| `$text-caption` | 10 | 1.3 | +1 | 상태바 / 거터 / 메타 |
| `$text-mini` | 11 | 1.4 | 0 | 타이틀바 보조 |
| `$text-body-sm` | 12 | 1.45 | 0 | 팔레트 본문 |
| `$text-body` | 13 | 1.5 | 0 | 일반 본문 |
| `$text-code` | 13 | 1.6 | 0 | 코드 (mono) |
| `$text-h4` | 14 | 1.4 | 0 | 카드 제목 |
| `$text-h3` | 16 | 1.3 | 0 | 섹션 제목 |
| `$text-h2` | 20 | 1.25 | -0.25 | 페이지 제목 |
| `$text-h1` | 28 | 1.2 | -0.5 | Onboarding hero |
| `$text-hero` | 40 | 1.1 | -1 | Landing CTA |

### Weights

```
$weight-regular  400
$weight-medium   500
$weight-semibold 600
$weight-bold     700
```

---

## 6. Spacing / Radius / Shadow

### Spacing (base 4px)

```
$space-1   4px
$space-2   8px
$space-3   12px
$space-4   16px
$space-5   20px
$space-6   24px
$space-8   32px
$space-10  40px
$space-12  48px
```

### Radius

```
$radius-sm    6px    # 작은 버튼
$radius-md    8px    # 기본 카드 / 탭 / 버튼
$radius-lg    10px   # 큰 카드
$radius-xl    12px   # 윈도우 shell
$radius-2xl   14px   # Command palette modal
$radius-pill  999px  # 상태 pill
```

### Shadow

```
$shadow-sm   0 1px 2px rgba(0,0,0,0.20)
$shadow-md   0 4px 12px rgba(0,0,0,0.35)
$shadow-lg   0 12px 32px rgba(0,0,0,0.45)  # Modal overlay
$shadow-xl   0 24px 60px rgba(0,0,0,0.55)  # Hero drop
```

---

## 7. Iconography

- **Primary**: [Lucide](https://lucide.dev/) — 일관된 선 스타일, 1.5pt stroke
- **Fallback**: SF Symbols (SwiftUI `Image(systemName:)` 자동 사용)
- 크기: 11 / 13 / 16 / 20 / 22
- 기본 색: `$fg-secondary`, hover: `$fg-primary`

### 의미 매핑

| 개념 | lucide | SF Symbols |
|------|--------|------------|
| 검색 | `search` | `magnifyingglass` |
| 알림 | `bell` | `bell` |
| 설정 | `settings` | `gearshape` |
| 터미널 | `terminal` | `terminal` |
| 코드 | `code` | `chevron.left.forwardslash.chevron.right` |
| FileTree | `folder-tree` | `folder` |
| 에이전트 | `bot` | `cpu` |
| Git | `git-branch` | `arrow.branch` |
| LSP | `circle-dot` | `waveform` |
| 재생 | `play` | `play.fill` |
| 일시정지 | `pause` | `pause.fill` |
| 정지 | `square` | `stop.fill` |
| 체크포인트 | `flag` | `flag.fill` |

### MX 태그 거터

| MX Tag | 아이콘 | 색 |
|--------|--------|-----|
| `@MX:NOTE` | `info` | `$status-info` |
| `@MX:WARN` | `alert-triangle` | `$status-warning` |
| `@MX:ANCHOR` | `anchor` | `$accent-moai` |
| `@MX:TODO` | `circle-dashed` | `$fg-muted` |

---

## 8. Layout Rules

### Window Shell

- 기본 크기: **1600 × 1000** (`.pen` frame 표준)
- 최소: 960 × 600
- 타이틀바: **44 px**
- 상태바: **28 px**
- 사이드바: **260 px** (min 220, max 400)
- 우측 Agent Run Viewer: **460 px** (min 360, max 600)

### 3-Pane Body

```
┌─Sidebar 260─┬─Terminal──┬─Code Viewer──┬─Agent Run Viewer 460─┐
│             │  60Hz     │  TreeSitter  │  Span tree +          │
│  Workspaces │  Ghostty  │  + LSP       │  stats + chart +      │
│  Worktrees  │  Metal    │  + @MX       │  detail + controls    │
└─────────────┴───────────┴──────────────┴───────────────────────┘
                              ↑
                   각 pane 은 NSSplitView binary tree 로
                   수평/수직 분할 가능 (최소 200pt)
```

### Status Bar

```
[⎇ main ↑2↓0] · [LSP: rust-analyzer ✓] · [moai-core v0.3.2]  ·  [⌘K to search]
```

- 항상 우측에 `⌘K to search` 힌트 (Command Palette 발견성).
- 좌측 아이콘은 클릭 가능 (예: 브랜치 클릭 → git widget).

### Toolbar (신규 필수)

```
[+ New Workspace] [⊟ Split H] [⊞ Split V] [▶ Run SPEC] [⌘K Palette] ... [? Help]
```

- Primary actions 5-7 개 (macOS Toolbar API 활용)
- Customizable 허용

---

## 9. Motion

- 기본 easing: `ease-out`, **180ms**
- Modal open: 180ms ease-out + slight scale (0.98 → 1)
- Palette focus: 120ms ease-out
- Pane resize: no animation (드래그 프레임 일치)
- Agent streaming tokens: 80ms fade-in per delta
- Toast: 200ms slide-in from top, 4s dwell, 200ms slide-out

---

## 10. Keyboard Shortcuts (Canonical)

| 단축키 | 동작 | 메뉴 위치 |
|--------|------|-----------|
| ⌘N | 새 워크스페이스 | File → New Workspace |
| ⌘⇧N | 새 Pane | Pane → New Pane |
| ⌘T | 새 탭 | File → New Tab |
| ⌘W | 탭 닫기 | File → Close |
| ⌘⇧W | Pane 닫기 | Pane → Close Pane |
| ⌘\ | Pane 수평 분할 | Pane → Split Horizontally |
| ⌘⇧\ | Pane 수직 분할 | Pane → Split Vertically |
| ⌘K | Command Palette | View → Open Command Palette |
| ⌘P | Go to File | Go → Go to File |
| ⌘⇧P | Go to Symbol | Go → Go to Symbol |
| ⌘, | Settings | MoAI Studio → Settings |
| ⌘0 | Sidebar 토글 | View → Toggle Sidebar |
| ⌘⌥R | Agent Run Viewer 토글 | View → Toggle Agent Run |
| ⌘B | File Tree 토글 | View → Toggle File Tree |
| ⌘⇧F | 프로젝트 검색 | Edit → Find in Project |
| ⌘G | Agent follow 토글 | View → Follow Agent |

[HARD] 모든 단축키는 메뉴 바 MenuItem 을 반드시 가진다.

---

## 11. Accessibility

- 모든 인터랙티브 요소 `accessibilityIdentifier` 필수 (UITest 용)
- 색 단독 정보 전달 금지 → 아이콘 병용 (색맹 지원)
- 키보드 포커스 링 기본 표시
- VoiceOver 라벨: 아이콘 전용 버튼은 `accessibilityLabel` 필수
- 콘트라스트: `$fg-secondary` / `$bg-surface` ≥ 4.5:1
- 최소 터치/클릭 타겟: 28 × 28

---

## 12. Anti-Patterns (학습된 것)

1. **빈 첫 화면**: 현재 구현이 이를 위반. 워크스페이스 0개 상태에서 아무 CTA 없음.
2. **Placeholder 텍스트로 기능 미구현 표시**: "Ghostty Metal surface will render here" 처럼. → M2.5 에서 제거됨.
3. **메뉴 바 공백**: Workspace/Pane/Palette 메뉴 0 → macOS 관례 위반.
4. **단축키-메뉴 비대칭**: Cmd+N 이 동작 안 하면서 메뉴는 노출.
5. **Project Path TextField 만**: NSOpenPanel 누락 → 절대 경로 수동 입력 마찰.
6. **침묵 실패**: 버튼 누르면 아무 반응 없음 (에러도 없음).

---

## 13. Per-Feature Direction

SPEC 별 디자인 방향은 `.moai/specs/SPEC-XXX/design-direction.md` 에 기록. 여기엔 cross-cutting 만 유지.

### Cross-Cutting

- **MX 태그 거터**: Code Viewer + Markdown Viewer 공통. 4종 아이콘, 클릭 → popover + REASON.
- **Agent Run pill**: 상태바 항상 표시. 실행 중이면 spin + cost running total.
- **Context Burst indicator**: 이번 턴에 주입된 추가 컨텍스트 개수 배지 (Agent Run Viewer 상단).

---

버전: 2.0.0 · 2026-04-17 (1.0 → 2.0: moai-design-craft 템플릿 대체, 리서치 기반 토큰 확정)
