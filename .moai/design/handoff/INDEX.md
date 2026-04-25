# Claude Design Handoff Bundle — File Index

---
title: Complete File Index & Navigation Guide
version: 1.0.0
source: Handoff bundle contents
last_updated: 2026-04-25
---

## 파일 목록 (20 files, ~5100 lines)

### Core Documentation (5 files)

| # | 파일 | 라인수 | 설명 | 읽기 순서 |
|---|------|-------|------|----------|
| 1 | **README.md** | ~200 | Bundle 사용법, 빠른 시작, 토큰 동기화 정책 | **1순위** |
| 2 | **01-app-overview.md** | ~300 | 한 줄 정의, IA, wireframe, 상태 분류, responsive behavior | 2순위 |
| 3 | **02-design-system.md** | ~400 | Design tokens 완전 요약 (색상/폰트/spacing/radius/shadow/motion) | 3순위 |
| 4 | **06-claude-design-prompt.md** | ~250 | **Claude.ai/design 에 붙여넣을 prompt** (가장 중요) | 1순위 |
| 5 | **INDEX.md** | ~80 | 본 파일 (전체 인덱스) | 읽는 중 |

### Surfaces (9 files, ~2250 lines)

| # | 파일 | 라인수 | Surface | SPEC | 상태 |
|---|------|-------|---------|------|------|
| 6 | surfaces/01-terminal.md | ~250 | Terminal (libghostty, 60fps) | V3-002 | ✓ 구현 |
| 7 | surfaces/02-panes-tabs.md | ~280 | Panes + Tabs (binary tree, persistence) | V3-003 | ✓ 구현 |
| 8 | surfaces/03-file-explorer.md | ~250 | File Explorer (git status, fuzzy search) | V3-005 | 설계 |
| 9 | surfaces/04-markdown-viewer.md | ~280 | Markdown Viewer (CommonMark + @MX) | V3-006 MS-1 | 진행 |
| 10 | surfaces/05-code-viewer.md | ~280 | Code Viewer (LSP + syntax highlight) | V3-006 MS-2 | 설계 |
| 11 | surfaces/06-agent-dashboard.md | ~280 | Agent Dashboard (hook event + cost + instructions) | V3-010 | 설계 |
| 12 | surfaces/07-git-management.md | ~250 | Git Management (status/diff/commit/branch) | V3-008 | 설계 |
| 13 | surfaces/08-spec-management.md | ~280 | SPEC Management (list/detail/Kanban/AC) | V3-009 | 설계 |
| 14 | surfaces/09-web-browser.md | ~200 | Web Browser (URL bar, DevTools, dev server auto-detect) | V3-007 | 설계 |

### Components (2 files, ~350 lines)

| # | 파일 | 라인수 | 설명 |
|---|------|-------|------|
| 15 | components/buttons-inputs.md | ~200 | Button (4 style × 4 size), Input, Checkbox, Radio, Switch, Tooltip, Dialog |
| 16 | components/feedback.md | ~150 | Toast (4 type), Banner, Spinner (3 size), Progress, Skeleton |

### Flows (3 files, ~400 lines)

| # | 파일 | 라인수 | 흐름 | 설명 |
|---|------|-------|------|------|
| 17 | flows/01-first-run.md | ~150 | First-Run Onboarding | Welcome → Workspace select → SPEC load → Ready |
| 18 | flows/02-file-open.md | ~150 | Open File | Explorer click / Cmd+P / Drag-drop → file type route → render |
| 19 | flows/03-pane-tab.md | ~100 | Pane/Tab Management | Split (Cmd+\\) → Divider drag → Last-focused restore → Persist |

### States (1 file, ~250 lines)

| # | 파일 | 라인수 | 설명 |
|---|------|-------|------|
| 20 | states/edge-cases.md | ~250 | 모든 surfaces 의 error states + recovery strategies |

---

## 추천 읽기 순서

### 1단계: 5분 스캔 (시간 부족할 때)

1. **README.md** — 전체 요약
2. **06-claude-design-prompt.md** — Claude Design prompt 복사

### 2단계: 30분 정독 (Claude Design 입력 준비)

1. **README.md** — bundle 구조 이해
2. **01-app-overview.md** — IA, wireframe, 상태 이해
3. **02-design-system.md** — 디자인 토큰 참조
4. **06-claude-design-prompt.md** — claude.ai/design 에 복사

### 3단계: 1시간 심화 (구현팀 참고)

위 + 다음:

5. 구현할 surfaces 선택:
   - **Tier 0** (이미 구현): `surfaces/01-terminal.md`, `surfaces/02-panes-tabs.md`
   - **Tier 1** (다음): `surfaces/03-file-explorer.md`, `surfaces/04-markdown-viewer.md`, `surfaces/05-code-viewer.md`
   - **Tier 1** (추가): `surfaces/06-agent-dashboard.md`, `surfaces/07-git-management.md`, `surfaces/08-spec-management.md`
   - **Tier 2** (이후): `surfaces/09-web-browser.md`

6. 사용자 흐름 이해: `flows/01-first-run.md`, `flows/02-file-open.md`, `flows/03-pane-tab.md`

7. Components 참고: `components/buttons-inputs.md`, `components/feedback.md`

8. Edge cases 검토: `states/edge-cases.md`

---

## 파일별 라인수 확인

### 분포

```
Core docs:      ~1,230 lines (24%)
Surfaces:       ~2,250 lines (44%)
Components:     ~350 lines (7%)
Flows:          ~400 lines (8%)
States:         ~250 lines (5%)
─────────────────────────────
Total:          ~5,100 lines
```

### 파일별 상세

| 파일 | 라인수 | % |
|------|-------|-----|
| 02-design-system.md | ~400 | 8% |
| 06-claude-design-prompt.md | ~250 | 5% |
| surfaces/* (9 files) | ~2,250 | 44% |
| 01-app-overview.md | ~300 | 6% |
| flows/* (3 files) | ~400 | 8% |
| components/* (2 files) | ~350 | 7% |
| states/edge-cases.md | ~250 | 5% |
| README.md | ~200 | 4% |
| INDEX.md (본 파일) | ~80 | 2% |

---

## 크로스 참조 맵

### README.md 참조
- → 01-app-overview.md (IA 상세)
- → 02-design-system.md (tokens 요약)
- → surfaces/* (각 surface 상세)
- → 06-claude-design-prompt.md (Claude Design input)

### 01-app-overview.md 참조
- → 02-design-system.md (색상/폰트 구체값)
- → surfaces/* (각 surface 상세)

### 02-design-system.md 참조
- → tokens.json (canonical source)
- → components/* (컴포넌트 토큰 적용)
- → surfaces/* (surface-specific tokens)

### 06-claude-design-prompt.md 참조
- → 모든 surfaces (참조 파일 목록)
- → components/* (UI library)
- → flows/* (user flows)
- → states/* (error states)

### surfaces/* 참조 상호
- surfaces/01-terminal.md ↔ 02-design-system.md (terminal tokens)
- surfaces/02-panes-tabs.md ↔ flows/03-pane-tab.md (pane management flow)
- surfaces/03-file-explorer.md ↔ flows/02-file-open.md (file open flow)
- etc.

---

## 용도별 가이드

### 역할별 추천

#### Claude Design (디자인 시안 생성)
1. **06-claude-design-prompt.md** (prompt 복사)
2. **02-design-system.md** (token 참고)
3. **surfaces/** 필요한 것만 (상세 확인)
4. **components/**, **flows/**, **states/** (optional, 참고용)

#### Implementation Team (코드 작성)
1. **02-design-system.md** (token 값 정확히)
2. **01-app-overview.md** (layout, wireframe)
3. **surfaces/** 구현 대상 선택
4. **components/** (UI 구현)
5. **flows/** (interaction 로직)
6. **states/** (error handling)

#### Product/QA (테스트, 검증)
1. **01-app-overview.md** (feature 확인)
2. **flows/** (happy path 검증)
3. **states/edge-cases.md** (error scenarios)
4. **surfaces/** (각 feature 상세)

#### Design Review (설계 검증)
1. **06-claude-design-prompt.md** (요구사항 확인)
2. **02-design-system.md** (일관성 체크)
3. **surfaces/** (각 surface 설계 재검증)
4. **states/edge-cases.md** (error state 설계)

---

## 버전 히스토리

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-04-25 | Initial release — 20 files, ~5100 lines, SPEC-V3-002/003 reflect, Tier 1 SPEC plan-ready |

---

## 갱신 정책

### Monthly Review
- 모든 surface files 새 feature 확인
- tokens.json 과 design-system.md 동기화
- flows/ 에 새로운 interaction 추가
- states/ 에 발견된 new edge case 추가

### Per-SPEC Completion
- 구현 완료 후 해당 surface file status 갱신 (설계 → 구현으로)
- 새 limitation 또는 design change 발견 시 해당 file 갱신
- flows/ 에서 실제 구현과 불일치 부분 수정

### Token Change
- tokens.json 변경 시:
  1. 02-design-system.md 동기화 (필수)
  2. 모든 surface files 에서 색상 hex 값 검증
  3. README 의 "주요 색상" 섹션 갱신

---

## FAQ

### Q. 이 bundle 을 전부 읽어야 하나?
**A.** 아니다. 목적에 따라:
- **Claude Design 시안 생성**: README + 06-claude-design-prompt + (필요시 2-design-system)
- **구현**: 해당 surface 파일만
- **디자인 리뷰**: 06-claude-design-prompt + surfaces/ + components/

### Q. tokens.json 이 source of truth 인가?
**A.** 네. **tokens.json 이 canonical source** 다. 이 bundle 문서는 view/reference 일 뿐.

### Q. 구현되지 않은 surface 는?
**A.** 파일 상단 "상태" 필드 확인:
- ✓ 구현 = SPEC 완료, implementation 존재
- 진행 중 = SPEC 진행, implementation 미완료
- 설계 = SPEC plan 완료, implementation 미시작

### Q. 모바일 대응 있나?
**A.** 아니다. **Desktop-first IDE**, 모바일 비목표.

### Q. Dark/Light theme 모두 설계했나?
**A.** 네. 모든 surface 가 **dark + light 양쪽 색상 명시**.

---

**마지막 수정**: 2026-04-25  
**총 파일**: 20  
**총 라인**: ~5,100  
**번들 크기**: ~350 KB (평문)  
**상태**: 완성 — Claude Design handoff ready

