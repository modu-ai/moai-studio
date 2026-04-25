# SPEC-V3-009 Research — SPEC Management UI

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render` (현재 브랜치, 본 SPEC 은 문서 산출만 — 별도 feature 브랜치는 implement 시점에 분기)
선행: SPEC-V3-004 (Render Layer, 화면 출력 파이프라인 사전조건), SPEC-V3-005 (File Explorer, SPEC 디렉터리 navigation), SPEC-V3-006 (Markdown/Code Viewer, SPEC 본문 렌더), SPEC-V3-010 (Agent Progress Dashboard, agent run 상태 통합).
범위: `.moai/specs/` 디렉터리를 시각화하고 EARS 요구사항 / Acceptance Criteria 상태를 추적하며 Kanban board 로 SPEC 진행을 관리하는 UI 레이어. moai-adk Go CLI 와 stream-json IPC 로 통신.

---

## 1. 동기 — moai-adk 와 가장 직접적으로 통합되는 UI

### 1.1 사용자 가치

moai-studio 가 moai-adk 의 GUI shell 로서 의미를 가지는 핵심 화면이다. 사용자는 본 UI 를 통해:

- `.moai/specs/SPEC-XXX/{spec.md, plan.md, contract.md, progress.md, tasks.md}` 의 구조를 트리/리스트로 한눈에 본다.
- 각 SPEC 의 EARS 요구사항 (RG-* / REQ-*) 과 Acceptance Criteria (AC-*) 의 현재 상태 (FULL/PARTIAL/DEFERRED/FAIL/PENDING) 를 시각적으로 확인한다.
- SPEC × milestone × task 진행률을 progress.md 의 record-of-truth 와 sync 된 상태로 본다.
- Kanban board (TODO / IN-PROGRESS / REVIEW / DONE) 로 다중 SPEC 의 현황을 sprint board 처럼 다룬다.
- `/moai plan SPEC-XXX`, `/moai run SPEC-XXX`, `/moai sync SPEC-XXX` 같은 슬래시 커맨드를 1-클릭으로 호출한다.
- Sprint Contract revision (SPEC-V3-003 의 10.1 ~ 10.5 와 같은 패턴) 을 자동 추출하여 본문 옆 사이드 패널에 시각화한다.
- SPEC 과 git worktree (CLAUDE.local.md §1.3 패턴: `feature/SPEC-XXX-slug`, `hotfix/v0.1.1-slug`) 의 매핑을 유지한다.

### 1.2 v2 SPEC-M5 reference

`.moai/design/archive/spec-v2.md` §3.2 (Frame 02. Kanban Board) 에 v2 디자인이 남아 있다. 핵심 의도는 보존하되 v3 GPUI 0.2.2 기반으로 재구현한다:

- Lane: TODO / IN-PROGRESS / REVIEW / DONE (4 lane)
- Card: SPEC-ID, title, milestone progress, AC count summary, last agent run badge
- Lane header: count + WIP limit (informational, hard 제한 아님)
- 카드에 `Agent Run` 미니 그래프 (최근 3 회 실행 히트맵) — SPEC-V3-010 결합 시 점진 추가, MS-1 범위 외
- SPEC 링크 뱃지 (`SPEC-V3-009` 타이포그래피)

본 SPEC 은 v2 SPEC-M5 의 Kanban 의도를 v3 escape hatch 로 부활시킨 형태다.

### 1.3 moai-adk 와의 통합점

moai-studio 는 moai-adk Go CLI 의 GUI shell 이다. 두 가지 통합 경로가 가능하다:

- (A) **subprocess + stream-json**: `moai stream-json` 같은 stdio JSON 라인 프로토콜로 직접 통신. 기존 `crates/moai-stream-json` (codec/decoder/message) 재사용 가능. 단순/안정.
- (B) **MCP server**: moai-adk 가 MCP 서버 모드로 동작, moai-studio 가 MCP client. 양방향 streaming + capability discovery 가능하지만 layer 추가.

USER-DECISION 게이트로 결정한다 (MS-1 진입 시점). 본 SPEC 의 default 가정은 (A) 다. 이유:
- `crates/moai-stream-json` 는 이미 SDKMessage 코덱이 검증되어 있다.
- moai-adk Go 측 stream-json 출력은 `moai run` / `moai sync` / `moai plan` 의 progress 이벤트가 이미 line-delimited JSON 으로 흘러나온다 (Claude CLI stream-json 호환).
- subprocess 모델은 macOS 14+ / Ubuntu 22.04+ 양쪽에서 동일하게 동작.

---

## 2. 코드베이스 분석 — 현재 가용한 building block

### 2.1 `crates/moai-stream-json` 재사용

- `src/lib.rs` 가 `codec`, `decoder`, `message` 를 export. SDKMessage 코덱.
- `src/decoder.rs` 의 `decode_and_publish`, `decode_line` 가 라인 단위 JSON 파싱 진입점.
- `src/message.rs` (17651 bytes) 가 SDKMessage 변형들 (Assistant/User/Tool/Result/SystemInit 등) 을 정의.

본 SPEC 의 IPC 레이어는 `moai-stream-json` 위에 얇은 wrapper (`SpecCommandClient`) 만 추가한다. 새 코덱 도입 없음.

### 2.2 `.moai/specs/` 파일 시스템 구조 — 현실 관찰

현재 12 개 SPEC 디렉터리:

```
.moai/specs/
├── SPEC-M0-001/
├── SPEC-M1-001/
├── SPEC-M2-001/
├── SPEC-M2-002/
├── SPEC-M2-003/
├── SPEC-M3-001/
├── SPEC-SPIKE-001/
├── SPEC-V3-001/
├── SPEC-V3-002/
├── SPEC-V3-003/   (8 files: spec/plan/research/acceptance/strategy/contract/progress/tasks)
├── SPEC-V3-004/   (3 files: spec/plan/research)
└── SPEC-V3-009/   (본 SPEC)
```

파일 셋이 SPEC 마다 가변적이다. v2 (M0~M3) 는 `acceptance.md` + `tasks.md` 위주, v3 는 `progress.md` + `contract.md` 가 추가된다. 본 UI 는 가변 파일 셋을 graceful 하게 처리해야 한다.

### 2.3 EARS / AC 표 형식 — 파싱 가능성

SPEC-V3-003 / V3-004 의 spec.md 는 다음 형식을 따른다:

```markdown
| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-001 | Ubiquitous | ... | ... |
```

```markdown
| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 |
|------|--------------|----------|----------|
| AC-R-1 | ... | ... | ... |
```

progress.md 는 "AC-R-5: PASS / FAIL / DEFERRED" 같은 status 라인을 갖는다. markdown AST 로 표 셀을 추출하면 EARS / AC 인덱스를 자동 생성할 수 있다.

### 2.4 Sprint Contract revision 인식

SPEC-V3-003 spec.md §10.1 ~ §10.5 같은 "Sprint Contract Revision N (date) — title" 형식의 sub-section 은 GAN Loop / 본 SPEC 의 §6.3 Sprint Contract Protocol 산출물이다. 본 UI 는 §10.x revision 헤더를 추출하여 timeline panel 에 표시한다.

### 2.5 git worktree 현재 패턴

CLAUDE.local.md §1.3 / §6.1:

- feature: `feature/SPEC-{area}-{nnn}-{slug}` (예: `feature/SPEC-V3-004-render`)
- hotfix: `hotfix/v{x.y.z+1}-{slug}` (예: `hotfix/v0.1.1-pane-focus-crash`)

본 UI 는 `git branch --list 'feature/SPEC-*'` 으로 현재 활성 SPEC ↔ branch ↔ worktree 매핑을 유지하고 카드에 표시한다.

---

## 3. GPUI 0.2.2 위에서의 구현 가능성

### 3.1 Render trait 패턴 (SPEC-V3-004 카피)

SPEC-V3-004 §2.1 가 `impl Render for TerminalSurface` 를 분석했다. 본 SPEC 의 모든 UI 컴포넌트 (`SpecListView`, `SpecDetailView`, `KanbanBoardView`, `SprintContractPanel`) 는 동일 패턴:

```
impl Render for SpecListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col()
            .child(self.toolbar(cx))
            .child(self.list_body(cx))
    }
}
```

### 3.2 File watch — `notify` crate

SPEC 파일 시스템 watch 는 `notify` crate (`v8`) + `tokio::sync::mpsc` 패턴이 가장 표준적이다. SPEC-V3-001 / SPEC-M2-002 의 workspace persistence 영역에서 이미 fs watch 가 일부 사용 중이며 (`crates/moai-studio-workspace/src/persistence.rs`), 본 SPEC 은 그 패턴을 차용한다.

debounce: 100ms (markdown 파일은 vim/IDE 가 atomic write 시 여러 이벤트를 발생시킴).

### 3.3 markdown AST parser 후보 — USER-DECISION

| Crate | 장점 | 단점 |
|-------|------|------|
| **pulldown-cmark** (v0.13) | rust-lang/mdbook 가 사용, 빠르고 well-tested, no_std 가능 | event-driven (pull-based), AST 구조화 별도 작업 필요 |
| **comrak** (v0.39) | GFM 완전 지원, AST 노드 직접 생성, render-back 가능 | 의존성 큼, 빌드 시간 더 길음 |

본 SPEC 은 EARS / AC 표 + frontmatter + heading 추출만 필요하므로 pulldown-cmark 가 default 권장. 단, `| ... | ... |` table 파싱은 comrak 가 더 robust 하다는 의견이 있어 USER-DECISION 게이트로 명시.

### 3.4 Kanban DnD — USER-DECISION

GPUI 0.2.2 에는 표준 DnD framework 가 없다 (Zed main 브랜치에는 있으나 0.2.2 pin). 두 옵션:

- (A) **자체 구현**: `on_mouse_down` + `on_mouse_move` + `on_mouse_up` 로 카드 위치를 상대 추적. SPEC 수가 적을 때 (수십 개) 충분.
- (B) **외부 라이브러리**: 없음. GPUI 생태계는 미성숙. 사실상 (A) 만 viable.

본 SPEC 의 default 는 (A). MS-2 의 AC 는 keyboard-only navigation (↑↓ + Enter to move stage) 을 우선 검증하고, mouse DnD 는 nice-to-have 로 후순위.

---

## 4. EARS 요구사항 그룹 — 6 개

본 SPEC 의 RG (Requirement Group) 는 사용자 입력의 "Suggested EARS Groups" 6 개를 그대로 채택한다. 정렬:

- **RG-SU-1** SPEC document watch + parse — 파일 시스템 진입점
- **RG-SU-2** AC state tracker (FULL/PARTIAL/DEFERRED/FAIL/PENDING) — 상태 모델
- **RG-SU-3** Kanban board UI — 시각화 (v2 SPEC-M5 reference)
- **RG-SU-4** SPEC ↔ git worktree 연동 — 브랜치 상태
- **RG-SU-5** `/moai *` 슬래시 커맨드 1-클릭 호출 (subprocess) — 실행 진입점
- **RG-SU-6** Sprint Contract 시각화 (10.x revision 자동 표시) — meta-progress

---

## 5. Milestone 분할 — 3 단계

### 5.1 MS-1 — Parser + AC tracker + 기본 list view

사용자 가시 정의: `.moai/specs/` 트리가 좌측 패널에 보이고, 클릭 시 본문이 로드되며, 본문에서 EARS 요구사항 + AC 표가 status 컬러 (FULL=green / PARTIAL=yellow / DEFERRED=gray / FAIL=red / PENDING=blue) 와 함께 표시된다.

핵심 산출:
- markdown AST parser (pulldown-cmark or comrak per USER-DECISION)
- AC state tracker (`AcState` enum, `AcRecord` struct)
- file watcher (notify + debounce 100ms)
- `SpecListView` (read-only 리스트 + 본문 미리보기)

### 5.2 MS-2 — Kanban board UI + 상태 transition

사용자 가시 정의: SPEC 들이 4 lane (TODO/IN-PROGRESS/REVIEW/DONE) Kanban 으로 보이고, 키보드 (↑↓ + Enter) 로 stage 이동이 가능하다. 변경 시 progress.md 또는 별도 sidecar 파일에 persist.

핵심 산출:
- `KanbanBoardView` (4 lane × N card)
- card minimal 카드: SPEC-ID + title + AC summary (e.g., "12/15 PASS, 2 PENDING, 1 FAIL")
- stage persistence: `.moai/specs/SPEC-XXX/.kanban-stage` (1-line text) 또는 progress.md `## Kanban Stage` section append
- keyboard navigation (mouse DnD 는 후순위)

### 5.3 MS-3 — `/moai *` 호출 + worktree 연동 + Sprint Contract viewer

사용자 가시 정의: 카드에서 1-click 으로 `/moai run SPEC-XXX` 를 호출 (subprocess + stream-json) 할 수 있고, 진행 상황이 본문 패널 하단에 stream 된다. SPEC 헤더에 git branch (`feature/SPEC-XXX-slug`) 와 worktree 경로가 표시되고 활성 여부가 보인다. 본문 패널 옆 사이드 패널에 §10.x Sprint Contract revision 들이 timeline 으로 보인다.

핵심 산출:
- `MoaiCommandClient` (subprocess spawn + stream-json line decode + event broadcast)
- worktree state mapper (`git branch --list 'feature/SPEC-*'`)
- `SprintContractPanel` (regex `^## (\d+)\.(\d+) Sprint Contract Revision` 으로 §10.x 추출)
- SPEC-V3-010 (Agent Progress Dashboard) 와의 join point — 본 SPEC 은 진입점만, dashboard 자체는 별 SPEC

---

## 6. 위험 요약

### R1. SPEC 파일 schema drift

`.moai/specs/SPEC-XXX/` 의 파일 셋이 SPEC 마다 다르다. acceptance.md 가 있는 SPEC 도 있고 없는 SPEC 도 있다. parser 가 missing file 을 panic 없이 graceful 하게 처리해야 한다. → AC-SU-1.5 가 명시.

### R2. EARS / AC 표 markdown 변형

표 형식이 spec.md 와 progress.md 사이에 일치하지 않는다 (예: spec 은 `| REQ ID |` 헤더, progress 는 `AC-R-5: PASS` 라인). 두 형식 모두 인식하는 dual parser 필요. → AC-SU-2.x.

### R3. Subprocess streaming → UI 갱신 latency

`moai run` 의 stream-json 출력이 burst 로 올 때 UI thread 가 GPUI cx.notify() 를 너무 자주 호출하면 stutter. → `tokio::sync::broadcast` + 16ms throttle (60fps target).

### R4. moai-adk Go CLI 의 stream-json 안정성

moai-adk-go 의 stream-json 출력 spec 은 Claude CLI 호환이지만 새 SDKMessage 변형 (예: `evaluator-active` score card) 은 미정. unknown variant 는 `serde(other)` 로 무시하되 raw text 로 fallback 표시.

### R5. Kanban stage persistence 충돌

multi-pane / multi-tab 환경에서 같은 SPEC 의 stage 를 동시 변경 시 race. → file lock + last-write-wins, conflict 시 user 알림.

### R6. CLAUDE.local.md branch convention 변경

CLAUDE.local.md §1.3 / §1.2 의 브랜치 명명 규칙이 v0.1.0 출시 후 변경 가능. parser 는 regex constants 를 한 곳 (`branch_naming.rs`) 에 모은다.

---

## 7. AC 후보 — 12 개 (사용자 제안 10-14 범위 내)

| AC ID | 검증 시나리오 (요약) | RG 매핑 |
|-------|----------------------|---------|
| AC-SU-1 | `.moai/specs/SPEC-V3-009/` directory 자체가 SpecListView 에 1 개 카드로 등장 | RG-SU-1 |
| AC-SU-2 | `.moai/specs/SPEC-V3-003/spec.md` 의 EARS 표가 RG-P-1 ~ RG-P-12 + REQ-* 로 파싱되어 본문에 표시 | RG-SU-1 |
| AC-SU-3 | `.moai/specs/SPEC-V3-003/progress.md` 의 AC-P-* 상태가 FULL / PARTIAL / DEFERRED / FAIL / PENDING 으로 컬러 분류 | RG-SU-2 |
| AC-SU-4 | spec.md 가 외부에서 변경되면 100ms 이내에 본문이 자동 갱신 | RG-SU-1 |
| AC-SU-5 | acceptance.md 가 없는 SPEC (예: SPEC-V3-001) 도 panic 없이 "no acceptance.md" 메시지와 함께 표시 | RG-SU-1 |
| AC-SU-6 | KanbanBoardView 가 4 lane (TODO/IN-PROGRESS/REVIEW/DONE) 로 모든 SPEC 을 보임 | RG-SU-3 |
| AC-SU-7 | 카드 선택 후 ↑↓ + Enter 로 stage 변경 시 progress.md 또는 sidecar 에 persist + 재시작 후 복원 | RG-SU-3 |
| AC-SU-8 | 활성 SPEC 의 git branch (`feature/SPEC-V3-009-slug`) 가 카드 헤더에 표시 + branch 미존재 시 hint 표시 | RG-SU-4 |
| AC-SU-9 | "Run" 버튼 클릭 시 `moai run SPEC-V3-009` subprocess 가 spawn 되어 stream-json 출력이 본문 하단 패널에 stream | RG-SU-5 |
| AC-SU-10 | subprocess 종료 시 exit code + 마지막 status 라인이 카드에 반영 | RG-SU-5 |
| AC-SU-11 | spec.md 의 `^## \d+\.\d+ Sprint Contract Revision` 헤더가 SprintContractPanel timeline 으로 추출 | RG-SU-6 |
| AC-SU-12 | terminal/panes/tabs core 코드 (RG-P-7 carry from V3-002/003) 무변경 — 본 SPEC 은 신규 crate `moai-studio-spec` 으로만 변경 | (cross-cutting) |

---

## 8. 파일 변경 / 신규 예측

### 8.1 신규 crate

- `crates/moai-studio-spec/` (신규)
  - `Cargo.toml`
  - `src/lib.rs` — re-export
  - `src/parser/mod.rs` — markdown parser entry
  - `src/parser/ears.rs` — EARS 표 추출
  - `src/parser/ac.rs` — AC 표 + status line 추출
  - `src/parser/sprint_contract.rs` — §10.x revision 추출
  - `src/state/mod.rs` — `SpecIndex`, `SpecRecord`, `AcState`
  - `src/state/persistence.rs` — kanban stage persist (sidecar 파일)
  - `src/watch.rs` — notify + debounce
  - `src/branch.rs` — `git branch --list` parser

### 8.2 신규 UI 모듈 (`crates/moai-studio-ui/src/spec_ui/`)

- `mod.rs` — re-export
- `list_view.rs` — `SpecListView`
- `detail_view.rs` — `SpecDetailView`
- `kanban_view.rs` — `KanbanBoardView` (MS-2)
- `sprint_panel.rs` — `SprintContractPanel` (MS-3)
- `command_client.rs` — `MoaiCommandClient` (subprocess + stream-json) (MS-3)

### 8.3 변경 (최소)

- `crates/moai-studio-ui/src/lib.rs` — `RootView` 에 spec_ui 진입점 등록 (SPEC-V3-004 의 `tab_container` 와 같은 layer)
- `Cargo.toml` workspace members 에 `moai-studio-spec` 추가

### 8.4 무변경 (RG-P-7 carry)

- `crates/moai-studio-terminal/**`
- `crates/moai-studio-ui/src/{terminal,panes,tabs}/**` (core)
- `crates/moai-studio-workspace/src/persistence.rs` (workspace 영역만)

---

## 9. 의존성 다이어그램

```
SPEC-V3-001 (scaffold) — 완료
SPEC-V3-002 (terminal core) — 완료
SPEC-V3-003 (panes/tabs logic) — MS-3 carry-over closed
SPEC-V3-004 (render layer) — 진행 중 (선행, 본 SPEC implement 진입 시 PASS 필요)
        │
        └──→ SPEC-V3-005 (file explorer) ──┐
        └──→ SPEC-V3-006 (md/code viewer) ─┼──→ SPEC-V3-009 (본 SPEC)
        └──→ SPEC-V3-010 (agent dashboard)─┘
```

본 SPEC 은 V3-005 / V3-006 / V3-010 와 **병행** 가능하다. 각각 다음 join point 에서 통합:

- V3-005 (File Explorer): `.moai/specs/` 디렉터리 노드를 file explorer 트리에서 클릭하면 본 SPEC 의 SpecListView / SpecDetailView 로 라우팅. interface = `SpecRecord::id`.
- V3-006 (Markdown Viewer): SpecDetailView 가 markdown 본문 렌더는 V3-006 의 markdown viewer 컴포넌트를 호출. 본 SPEC 은 metadata overlay (EARS / AC status badge) 만 추가.
- V3-010 (Agent Dashboard): 카드에 표시되는 "최근 agent run badge" 는 V3-010 의 AgentRun store 에서 query. 본 SPEC 은 read-only consumer.

---

## 10. USER-DECISION 게이트

| ID | 시점 | 질문 | Default 권장 |
|----|------|------|--------------|
| USER-DECISION-SU-A | MS-1 진입 | markdown AST parser 선택: pulldown-cmark vs comrak | pulldown-cmark (이유: 작은 deps, 표 파싱 충분) |
| USER-DECISION-SU-B | MS-2 진입 | Kanban DnD 라이브러리: 자체 구현 vs 외부 | 자체 구현 (이유: GPUI 0.2.2 외부 lib 부재) |
| USER-DECISION-SU-C | MS-3 진입 | moai-adk 통합: subprocess + stream-json vs MCP server | subprocess + stream-json (이유: `moai-stream-json` crate 재사용, 단순) |

---

## 11. 비목표 (Non-Goals)

- N1. SPEC document 편집 — 본 UI 는 read-only + stage transition 만. spec.md 본문 편집은 별도 SPEC.
- N2. `/moai plan` 으로 신규 SPEC 생성 wizard — 별 SPEC.
- N3. multi-project (`.moai/specs/` 가 여러 워크스페이스) — SPEC-M2-002 가 다중 워크스페이스를 지원하지만 본 UI 는 active workspace 의 specs/ 만 본다.
- N4. SPEC dependency graph 시각화 — `depends_on` frontmatter 는 읽지만 graph view 는 별 SPEC.
- N5. CHANGELOG / PR 자동 생성 — `/moai sync` 호출만, 결과 PR 은 GitHub 측.
- N6. terminal/panes/tabs core 변경 — RG-P-7 carry.
- N7. mouse drag-and-drop Kanban — MS-2 default 는 keyboard-only, mouse 는 follow-up SPEC.
- N8. Windows 빌드 — SPEC-V3-002/003/004 N6/N10 carry.

---

## 12. 참고 자료

- `.moai/design/archive/spec-v2.md` §3.2 — v2 Kanban Board reference (Frame 02)
- `.moai/specs/SPEC-V3-003/contract.md` — Sprint Contract Protocol 의 §10.x revision 패턴 원본
- `.moai/specs/SPEC-V3-004/research.md` — GPUI 0.2.2 Render trait 패턴
- `crates/moai-stream-json/src/{lib,decoder,message}.rs` — stream-json 코덱
- `crates/moai-studio-workspace/src/persistence.rs` — 기존 fs watch 패턴 참조
- `CLAUDE.local.md` §1.2 / §1.3 / §6.1 — branch naming convention
- pulldown-cmark v0.13 (Context7: `/raphlinus/pulldown-cmark`)
- comrak v0.39 (Context7: `/kivikakk/comrak`)
- notify v8 (Context7: `/notify-rs/notify`)

---

작성 종료 — 본 research.md 는 spec.md 와 plan.md 의 근거 문서다. SPEC-V3-009 의 implement 진입은 SPEC-V3-004 PASS 후 별도 feature 브랜치 (`feature/SPEC-V3-009-spec-ui`) 분기 시 시작한다.
