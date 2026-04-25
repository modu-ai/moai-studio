# Claude Design Handoff Bundle — MoAI Studio v3

---
title: Claude Design Handoff Bundle
version: 1.0.0
source: moai-studio v3 (Rust + GPUI 0.2.2)
last_updated: 2026-04-25
---

## 본 Bundle 에 대해

이 디렉터리는 **moai-studio v3 전체 UI/UX 디자인을 claude.ai/design 에 전달**하기 위한 종합 핸드오프 패키지이다.

moai-studio 는 **moai-adk (Agentic Development Kit) 의 크로스플랫폼 Agentic Coding IDE** 다. Rust + GPUI 0.2.2 로 작성되었으며, macOS / Linux / Windows 를 목표로 한다. 다크테마 우선, 한국어/영문 이중언어 지원, 키보드 우선 인터페이스이다.

---

## 빠른 시작 — 5분 안에

### 1단계: 이 번들 이해하기 (2분)

이 `handoff/` 디렉터리 구조를 스캔하면 moai-studio 의 전체 IA 와 기능을 파악할 수 있다:

- **README.md** (본 파일) — bundle 사용법
- **01-app-overview.md** — 앱 전체 한줄 정의, IA, 윈도우 레이아웃, 상태 분류
- **02-design-system.md** — 모든 디자인 토큰 요약 (색상/폰트/spacing/radius/shadow/motion)
- **06-claude-design-prompt.md** — **claude.ai/design 에 그대로 복사-붙여넣을 prompt** (가장 중요함)
- **surfaces/** — 9개 주요 UI 영역별 상세 설명
  - 01-terminal.md — GPU 가속 터미널 (libghostty)
  - 02-panes-tabs.md — 탭 바 + 패널 분할 기능
  - 03-file-explorer.md — 파일 트리 + git status
  - 04-markdown-viewer.md — 마크다운 렌더 + MX 거터
  - 05-code-viewer.md — 코드 에디터 + LSP 진단
  - 06-agent-dashboard.md — 에이전트 진행 상황 + cost + hook event
  - 07-git-management.md — git status/diff/commit/branch UI
  - 08-spec-management.md — SPEC 목록 + Kanban + 상태 추적
  - 09-web-browser.md — 내장 웹 브라우저
- **components/** — 공용 UI 컴포넌트 (버튼, 입력, 피드백)
- **flows/** — 주요 사용자 흐름 (첫 실행, 파일 열기, 탭 전환)
- **states/** — edge cases 와 에러 상태
- **INDEX.md** — 전체 파일 인덱스 + 우선순위 가이드

### 2단계: claude.ai/design 에 입력하기 (3분)

1. **06-claude-design-prompt.md** 를 열기
2. 전체 내용 선택 (Cmd/Ctrl+A)
3. claude.ai/design 새 프로젝트 생성
4. prompt 섹션에 붙여넣기
5. "Design" 또는 "Generate Mockups" 클릭
6. 참조 파일: 본 번들의 다른 파일들을 Claude Design 이 읽도록 지시 (선택사항)

---

## 파일 구조 인덱스

```
handoff/
├── README.md                    (본 파일)
├── 01-app-overview.md          (IA, wireframe, 상태)
├── 02-design-system.md         (토큰 전체 요약)
├── 06-claude-design-prompt.md  ⭐ 가장 중요 — claude.ai/design 입력용
├── INDEX.md                     (전체 파일 목록 + 우선순위)
├── surfaces/
│   ├── 01-terminal.md
│   ├── 02-panes-tabs.md
│   ├── 03-file-explorer.md
│   ├── 04-markdown-viewer.md
│   ├── 05-code-viewer.md
│   ├── 06-agent-dashboard.md
│   ├── 07-git-management.md
│   ├── 08-spec-management.md
│   └── 09-web-browser.md
├── components/
│   ├── buttons-inputs.md
│   └── feedback.md
├── flows/
│   ├── 01-first-run.md
│   ├── 02-file-open.md
│   └── 03-pane-tab.md
└── states/
    └── edge-cases.md
```

**총 20 파일, ~5000 라인**

---

## 디자인 토큰 참조

모든 색상/폰트/spacing 값은 canonical source **`.moai/design/tokens.json`** 에서 가져온다:

### 핵심 색상
- **Primary (brand)**: `#2563EB` (moai blue) — CTA, active state, focus ring
- **Secondary (AI)**: `#8B5CF6` (violet) — agent activity
- **Accent**: `#06B6D4` (cyan) — links, terminal highlight
- **Semantic**: success `#10B981`, warning `#F59E0B`, error `#EF4444`, info `#3B82F6`

### 핵심 폰트
- **Sans (UI)**: Pretendard → Inter → system-ui (한글 우선)
- **Mono (Code/Terminal)**: JetBrains Mono → Fira Code (ligatures 지원)

### 핵심 spacing 단계
- 4-base 스케일: 0 / 4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48 / 64 / 80 / 96 px

### 주요 컴포넌트 토큰
- Button: padding 12/8, radius 6, weight 500
- Input: padding 12/8, radius 6, border 1px
- Tab: height 32, padding-x 12, radius 4, weight 600 (active)
- Pane: min 240×120, divider 4px
- Terminal: JetBrains Mono 14px, line-height 1.4
- Markdown viewer: 16px, relaxed (1.75), max-width 780px

자세한 토큰은 **`.moai/design/tokens.json`** 참조.

---

## 테마 전략

### Dark Theme (기본)
- Background: neutral.950 (앱 전체), neutral.900 (패널), neutral.800 (surface)
- Text: neutral.50 (주), neutral.300 (보조), neutral.400 (tertiary)
- Border: neutral.700 (기본), neutral.600 (강), primary.500 (focus ring)

### Light Theme (대안)
- Background: neutral.0 (앱), neutral.50 (패널), neutral.100 (surface)
- Text: neutral.950 (주), neutral.700 (보조), neutral.500 (tertiary)
- Border: neutral.200 (기본), neutral.400 (강), primary.500 (focus ring)

Focus ring 은 양 테마 모두 **primary.500** (thick 5px, 4px offset).

---

## 접근성 + 성능 제약

### WCAG 2.1 AA
- 텍스트 대비율 ≥ 4.5:1 (main text), ≥ 3:1 (large text/UI components)
- Focus ring 항상 가시적 (thick 5px)
- 키보드 네비게이션 100% (마우스 보조, 키보드 필수)
- prefers-reduced-motion: all animations → 0ms

### 성능
- 터미널: 60fps @ 4K (libghostty GPU 가속)
- 코드 에디터: 16ms frame budget (60fps)
- 패널 분할: 60fps smooth drag
- 가상 스크롤 (markdown/code large file)

### 크로스플랫폼
- macOS / Linux / Windows 동일 토큰
- OS-specific: key modifier (macOS=Cmd, Linux/Windows=Ctrl), file picker (rfd native)

---

## 우선순위 가이드

### Tier 0 — 이미 구현됨 (시안 매칭 필요)
- Terminal (SPEC-V3-002)
- Panes + Tabs (SPEC-V3-003)
- 기본 layouts, toolbar, sidebar

### Tier 1 — 다음 구현 (P1 SPEC)
- File Explorer (SPEC-V3-005)
- Markdown Viewer (SPEC-V3-006 MS-1)
- Code Viewer (SPEC-V3-006 MS-2)
- Git Management (SPEC-V3-008)
- SPEC Management (SPEC-V3-009)
- Agent Dashboard (SPEC-V3-010)

### Tier 2 — 이후 (P2 SPEC)
- Web Browser (SPEC-V3-007)
- Advanced agent features
- Cross-platform packaging

### Claude Design 산출물 권장 순서
1. **Tier 0 surfaces** (terminal, panes, tabs, sidebar, toolbar) — 재확인 + 일관성 수정
2. **Tier 1 surfaces** — full design system + mockup (36+ 상태 = 9 surface × dark/light × empty/loading/populated/error)
3. **Components library** — Figma/design system export
4. **Flows** — clickthrough prototype
5. **Interactive specs** — Inspect-mode JSON for GPUI implementation

---

## 동기화 정책

### Token 변경 시
1. **`.moai/design/tokens.json`** 편집
2. 본 번들의 **02-design-system.md** 수동 동기화 (canonical = JSON, doc = view)
3. 모든 surface 파일의 토큰 참조 검증

### Surface 구현 변경 시
1. 해당 surface 파일 (e.g., `surfaces/01-terminal.md`) 갱신
2. 06-claude-design-prompt.md 의 "산출물 요청" 섹션 갱신 (필요시)

### 월 1회 전체 검토
- 모든 상태 행렬 검증 (empty / loading / populated / error)
- dark/light theme 일관성 재검증
- WCAG 2.1 AA 준수 재확인

---

## Bundle 외부 참조

이 bundle 은 다음 canonical sources 를 참조한다. 갱신 시 먼저 원본 읽기:

| 파일 | 용도 | 경로 |
|------|------|------|
| Visual Identity v1.0.0 | 브랜드 철학, typography, color rationale | `.moai/project/brand/visual-identity.md` |
| Brand Voice v1.0.0 | tone, vocabulary, audience | `.moai/project/brand/brand-voice.md` |
| Design Tokens v1.0.0 | Machine-readable tokens (canonical) | `.moai/design/tokens.json` |
| Product v0.0.x | v3 Pivot notice, feature list | `.moai/project/product.md` |
| Structure v0.0.x | 18 crate topology, v3 module map | `.moai/project/structure.md` |
| SPEC-V3-001/002/003 | Feature acceptance criteria (AC) | `.moai/specs/SPEC-V3-*/spec.md` |

---

## Claude Design Workflow

### Path A (Claude Design import) — 권장
본 bundle 의 **06-claude-design-prompt.md** → claude.ai/design 붙여넣기 → import tokens.json 섹션 → 자동 design system 생성 → mockup generation.

**장점**: 토큰 일관성 자동 보장, moai brand context 자동 적용

### Path B (code-based design) — 안내용
`moai-domain-brand-design` skill 이 `tokens.json` 직접 읽고 GPUI Rust 상수 생성. hand-coded design spec 없이 token → code 자동 변환.

---

## 질문 & 문제해결

### Q. 아직 구현되지 않은 surface 는?
**A.** File Explorer (V3-005), Markdown/Code Viewer (V3-006), Git/SPEC/Agent 관리 UI (V3-008/009/010), Web Browser (V3-007) 등은 상세 설계만 있고 구현 미완료. Claude Design 이 시안을 제시하면 implementation team 이 GPUI 코드로 변환한다.

### Q. Figma / 다른 도구로 export 해야 하나?
**A.** 아니다. 본 bundle 만으로 충분하다. Claude Design 이 고-피델리티 mockup (Figma-style) 또는 interactive prototype 직접 생성 가능.

### Q. tokens.json 값이 bundle 문서와 다르면?
**A.** **tokens.json 이 canonical source** 다. 문서가 outdated 다. 즉시 이 README 작성자에게 보고.

### Q. Windows / Linux 레이아웃 차이?
**A.** 없다. 토큰 100% 동일. OS-specific 은 component-level 만 (key modifier, file picker 네이티브 대화).

---

## 버전 히스토리

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-04-25 | Initial v3 bundle — 20 files, ~5000 lines, SPEC-V3-002/003 reflect |

---

## 라이센스 & 귀속

- **Design Tokens**: moai brand identity + agentic coding IDE conventions (Zed / VS Code / Cursor)
- **Typography**: Pretendard (모두의 폰트, 오픈소스), Inter, JetBrains Mono
- **Icon Recommendations**: Phosphor, Lucide (오픈소스)
- **Framework**: GPUI 0.2.2 (Zed, Apache 2.0)

---

**마지막 수정**: 2026-04-25  
**담당**: moai-studio design team  
**상태**: v1.0.0 — stable for Claude Design input
