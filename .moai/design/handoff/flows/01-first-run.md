# User Flow — First Run

---
title: First-Run Onboarding Flow
version: 1.0.0
source: product.md
last_updated: 2026-04-25
---

## 흐름도

```
App Launch
    ↓
Empty State? → Yes → [Welcome Screen]
    ↓                      ↓
    No                 [Select Workspace]
    ↓                      ↓
[Show Last            [Detect .moai/]
 Workspace]               ↓
    ↓                 [Load SPEC tree]
[Show Sidebar]            ↓
 + File tree         [Show Sidebar]
 + SPEC tree             ↓
 + Git status        [Create 1st Tab]
    ↓                     ↓
[Show Main            [Show Terminal/Code]
 Content]                 ↓
    ↓                 Ready for use
Ready for use
```

---

## 단계별 상세

### Step 1: Welcome Screen

```
┌────────────────────────────────────┐
│                                     │
│       🗿 MoAI Studio                │
│                                     │
│   Cross-platform Agentic IDE       │
│                                     │
│                                     │
│   [Open Workspace]                 │
│   [Recent Workspaces ▼]            │
│   [Browse Examples]                │
│                                     │
│   [Settings] [Help]                │
│                                     │
└────────────────────────────────────┘
```

- 배경: neutral.950 (dark)
- 로고: 중앙 정렬, 72px
- 한 줄: "Cross-platform Agentic IDE"
- 버튼: 중앙, primary style (lg size)

### Step 2: Select Workspace

#### 옵션 A: Open Folder

```
[📁 Open Workspace...]

Finder / Explorer dialog
→ 사용자가 git 루트 선택
→ `.moai/` 감지 (moai 프로젝트?) 또는 생성
```

#### 옵션 B: Recent Workspaces

```
📁 Recent Workspaces

1. /Users/user/projects/moai-studio    (2h ago)
2. /Users/user/projects/app-backend    (1d ago)
3. /Users/user/projects/web-frontend   (1w ago)

[Browse more...]
```

### Step 3: Load SPEC Tree

```
┌──────────────────────────────────────┐
│ 로드 중...                           │
├──────────────────────────────────────┤
│                                       │
│  ⟳ Scanning workspace...             │
│  ✓ Detected .moai/specs/ (8 SPEC)   │
│  ✓ Loaded git repository             │
│  ⟳ Analyzing dependencies...         │
│                                       │
└──────────────────────────────────────┘
```

- 진행 상황: 실시간 업데이트
- Spinner: 지속적 animation
- 로그: 각 단계별 status (✓ complete / ⟳ running)

### Step 4: Show Sidebar

```
┌────────────┐
│ Workspace  │
│ /path/to   │
├────────────┤
│ File       │ ← 자동 expand
│ ├─ src/    │
│ ├─ tests/  │
│ └─ docs/   │
│            │
│ SPEC (8)   │ ← .moai/specs 로드됨
│ ├─ V3-001✓ │
│ ├─ V3-002✓ │
│ └─ ...     │
│            │
│ Git        │
│ Branch:    │
│ main       │
│            │
│ Agent      │
│ (idle)     │
└────────────┘
```

- Sidebar: expand 상태 (240px)
- File tree: 최상위 폴더 expand
- Git: current branch 표시
- Agent: idle status

### Step 5: Create 1st Tab

```
┌──────────────────────────────────────┐
│ [README.md] [+]                      │
├──────────────────────────────────────┤
│                                       │
│ # Project README                     │
│                                       │
│ This is a moai-studio project...     │
│                                       │
└──────────────────────────────────────┘
```

자동 선택 규칙:
1. README.md (존재하면)
2. .moai/project/product.md
3. 첫 파일 (alphabetical)
4. Fallback: 빈 terminal

### Step 6: Ready

```
┌────────────────────────────────────────┐
│ File  Edit  View  Workspace  Help   ☀  │
├────────────────────────────────────────┤
│ [README.md] [+]   🔍 search            │
├──────────┬────────────────────────────┤
│          │                             │
│ Sidebar  │ Main Content (code/text)   │
│          │                             │
│ ready    │ Ready to use                │
│ for use  │                             │
│          │                             │
└──────────┴────────────────────────────┘
```

- All surfaces active
- Sidebar fully loaded
- Main content ready for editing
- User can start development

---

## 주요 상호작용

### 취소 (Cancel)

언제든 'Back' 버튼 또는 Escape:
- Step 2 (Select) → Step 1 (Welcome)
- Step 3+ (Loading) → Step 1 (Welcome)

### 에러 처리

```
⚠️ Failed to load workspace

Error: .moai/ not found
This is not a moai-adk project.

[Create .moai/] [Browse different] [Cancel]
```

---

## 시간대 및 성능

- Step 1 (Welcome): < 500ms (instant)
- Step 2 (Select): 파일 대화 시간 (사용자 의존)
- Step 3 (Load): < 2s (typical small project)
- Step 4-6 (Render): < 500ms
- **전체 Total**: < 3s (from app launch to ready)

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — 전형적인 첫 실행 흐름

