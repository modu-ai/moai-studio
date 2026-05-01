---
id: SPEC-V3-009
version: 1.0.0
status: implemented
created_at: 2026-04-25
updated_at: 2026-04-26
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-004]
parallel_with: [SPEC-V3-005, SPEC-V3-006, SPEC-V3-010]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, gpui, spec-management, kanban, moai-adk-integration]
revision: v1.0.0 (initial draft, v2 SPEC-M5 Kanban reference 부활)
---

# SPEC-V3-009: SPEC Management UI — `.moai/specs/` 시각화 + EARS/AC tracker + Kanban board + moai-adk integration

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-adk 와 가장 직접적으로 통합되는 UI 의 정의. v2 design SPEC-M5 (Kanban) 의 의도를 v3 GPUI 0.2.2 위에서 부활. RG-SU-1 ~ RG-SU-6, AC-SU-1 ~ AC-SU-12, MS-1/MS-2/MS-3, USER-DECISION 3 게이트. SPEC-V3-004 (render) 선행, V3-005/006/010 와 병행 가능. terminal/panes/tabs core 무변경 (RG-P-7 carry). |
| 1.0.0 | 2026-04-26 | 전체 구현 완료. MS-1 (#30) Parser + AC tracker + List/Detail (AC-SU-1~5). MS-2 (#31) Kanban Board + sidecar persist (AC-SU-6/7), USER-DECISION-SU-B = (a) keyboard-only. MS-3 (#32) git branch parser + moai CLI subprocess + Sprint Contract panel (AC-SU-8/9/10/11/12), USER-DECISION-SU-C = (a) subprocess+stream-json. terminal/panes/tabs/RootView 0 byte change (N6 carry). RootView 진입점 등록은 후속 SPEC 위임. status: ms1-implemented → implemented. |

---

## 1. 개요

### 1.1 목적

moai-adk Go CLI 의 산출물인 `.moai/specs/SPEC-XXX/` 디렉터리 (spec.md / plan.md / research.md / acceptance.md / contract.md / progress.md / tasks.md) 를 시각화하고, EARS 요구사항 (RG-* / REQ-*) 과 Acceptance Criteria (AC-*) 의 상태 (FULL/PARTIAL/DEFERRED/FAIL/PENDING) 를 추적하며, 다중 SPEC 의 진행을 Kanban board 로 관리하는 UI 레이어를 정의한다.

본 SPEC 은 moai-studio 가 moai-adk 의 GUI shell 로 의미를 가지는 핵심 화면이다. SPEC × milestone × task progress 가 한 화면에서 join 되며, 사용자는 `/moai plan|run|sync SPEC-XXX` 슬래시 커맨드를 1-클릭으로 호출하고 그 진행을 본 UI 에서 stream 으로 본다. Sprint Contract revision (SPEC-V3-003 의 §10.x 와 같은 패턴) 도 자동 추출하여 timeline 으로 시각화한다.

### 1.2 v2 SPEC-M5 carry-over 와의 관계

`.moai/design/archive/spec-v2.md` §3.2 는 v2 Frame 02 Kanban Board 의 의도 (lane = TODO/IN-PROGRESS/REVIEW/DONE, card = SPEC link badge + Agent Run mini-graph + WIP count) 를 제시했으나 v2 단계에서 미구현으로 남아있다. 본 SPEC 의 RG-SU-3 가 그 미구현분의 v3 carrier 다.

본 SPEC 은 v2 SPEC-M5 의 의도를 보존하되 다음 두 가지를 v3 escape hatch 로 한다:
- (a) Mouse drag-and-drop 은 MS-2 default 비목표. Keyboard-only navigation (↑↓ + Enter) 가 우선.
- (b) Agent Run mini-graph 는 SPEC-V3-010 (Agent Dashboard) 의 책임. 본 SPEC 은 read-only badge 만 표시.

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-009/research.md` — 코드베이스 분석, USER-DECISION 게이트 정의, AC 후보, 위험 요약.
- `.moai/specs/SPEC-V3-003/contract.md` §10.x — Sprint Contract Revision 패턴 원본 (RG-SU-6 가 본 패턴 추출 책임).
- `.moai/specs/SPEC-V3-004/research.md` — GPUI 0.2.2 Render trait 패턴 (본 SPEC UI 컴포넌트 동일 적용).
- `.moai/design/archive/spec-v2.md` §3.2 — v2 Kanban Frame 02 reference.
- `crates/moai-stream-json/src/{lib,decoder,message}.rs` — RG-SU-5 의 IPC 기반 코덱.
- `crates/moai-studio-workspace/src/persistence.rs` — fs watch 기존 패턴 참조 (RG-SU-1).
- `CLAUDE.local.md` §1.2 / §1.3 / §6.1 — branch naming convention (RG-SU-4).

---

## 2. 배경 및 동기

본 섹션의 상세는 `.moai/specs/SPEC-V3-009/research.md` §1 ~ §6 참조. 최소 맥락만 요약한다.

- **moai-adk 통합 갭** (research §1.1): moai-studio 가 moai-adk 의 shell 임에도 `.moai/specs/` 를 직접 시각화하는 화면이 없었다. 사용자가 `cat .moai/specs/SPEC-V3-003/progress.md` 같은 CLI 동작에 의존했다.
- **v2 SPEC-M5 의 미구현 부채** (research §1.2): v2 Frame 02 Kanban 이 디자인만 남고 미구현. 본 SPEC 이 그 부채의 v3 carrier.
- **EARS / AC 자동 인덱싱 필요** (research §2.3): 12 개 SPEC 디렉터리의 EARS 요구사항과 AC 가 markdown 표로 누적되었으나 시각적 인덱스가 없다.
- **Sprint Contract revision 추적** (research §2.4): SPEC-V3-003 spec.md §10.1 ~ §10.5 의 revision 누적은 GAN Loop 의 핵심 산출이지만 timeline view 가 없다.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `.moai/specs/` 디렉터리 트리가 `SpecListView` 좌측 패널에 SPEC 카드 형태로 보인다.
- G2. 선택된 SPEC 의 EARS 요구사항 (RG-* / REQ-*) 과 AC (AC-*) 가 본문에 status 컬러 (FULL/PARTIAL/DEFERRED/FAIL/PENDING) 와 함께 표시된다.
- G3. spec.md / progress.md 가 외부에서 변경되면 100ms 이내에 본문이 자동 갱신된다 (notify + debounce).
- G4. 4 lane (TODO/IN-PROGRESS/REVIEW/DONE) Kanban board 가 모든 SPEC 을 보이고, 키보드 (↑↓ + Enter) 로 stage 변경이 가능하며 변경은 persist 된다.
- G5. 활성 SPEC 의 git branch (`feature/SPEC-XXX-slug`) 와 worktree 경로가 카드 헤더에 표시된다.
- G6. 카드의 "Run" 버튼 클릭 시 `moai run SPEC-XXX` subprocess 가 spawn 되고 stream-json 출력이 본문 하단 패널에 stream 된다.
- G7. spec.md 의 `^## \d+\.\d+ Sprint Contract Revision` 헤더가 SprintContractPanel timeline 으로 추출된다.
- G8. terminal/panes/tabs core (RG-P-7 carry from SPEC-V3-002/003) 의 코드는 변경되지 않는다.
- G9. macOS 14+ / Ubuntu 22.04+ 양쪽에서 동일 동작 (Windows 는 비목표).

### 3.2 비목표 (Non-Goals)

- N1. SPEC document **편집** — 본 UI 는 read-only + stage transition 만. spec.md 본문 편집은 별도 SPEC.
- N2. `/moai plan` 으로 **신규 SPEC 생성 wizard** — 별 SPEC.
- N3. **multi-project** (다중 워크스페이스의 specs/ 통합) — active workspace 의 specs/ 만 표시.
- N4. **SPEC dependency graph 시각화** — `depends_on` frontmatter 는 읽지만 graph view 는 별 SPEC.
- N5. **CHANGELOG / PR 자동 생성** — `/moai sync` 호출만, 결과 PR 은 GitHub 측.
- N6. **terminal/panes/tabs core 변경** — RG-P-7 carry. 본 SPEC 은 신규 crate `moai-studio-spec` 과 `moai-studio-ui/src/spec_ui/` 신규 모듈만 변경.
- N7. **mouse drag-and-drop Kanban** — MS-2 default 는 keyboard-only, mouse DnD 는 follow-up SPEC.
- N8. **Windows 빌드** — SPEC-V3-002/003/004 N10 carry.
- N9. **새 design token 추가** — SPEC-V3-001 의 `app.background` / `panel.background` / `text.primary` 등 기존 토큰 재사용. status 컬러는 기존 `status.success/warning/error/info` 토큰 재사용.
- N10. **Agent Run 상세 dashboard** — SPEC-V3-010 의 책임. 본 SPEC 은 카드에 read-only badge 만.

---

## 4. 사용자 스토리

- **US-SU-1**: 사용자가 앱을 실행하면 좌측 패널에 `.moai/specs/` 의 모든 SPEC 디렉터리가 카드 리스트로 보인다 → SpecListView 가 fs scan 후 `Vec<SpecRecord>` 를 렌더.
- **US-SU-2**: 사용자가 SPEC 카드를 클릭하면 본문에 spec.md 가 렌더되고, EARS 요구사항 표 위에 RG/REQ ID 별 status badge 가 overlay 된다 → SpecDetailView 가 markdown viewer (V3-006 위임) + metadata overlay.
- **US-SU-3**: 사용자가 다른 터미널에서 spec.md 를 수정하면 100ms 이내에 본문이 자동 갱신된다 → notify watcher + debounce 100ms + cx.notify().
- **US-SU-4**: 사용자가 acceptance.md 가 없는 오래된 SPEC (예: SPEC-V3-001) 을 클릭해도 panic 없이 "no acceptance.md" 메시지가 보인다 → graceful missing-file handling.
- **US-SU-5**: 사용자가 Kanban view 로 전환하면 4 lane 에 카드들이 stage 별로 분류되어 보인다 → KanbanBoardView 가 `.kanban-stage` sidecar 또는 progress.md 의 `## Kanban Stage` section 을 read.
- **US-SU-6**: 사용자가 카드를 ↑↓ 로 선택 + Enter 로 다음 stage (TODO → IN-PROGRESS → REVIEW → DONE) 로 이동하면 sidecar 에 즉시 persist + 다음 실행 시 동일 stage 로 복원된다 → keyboard navigation + last-write-wins.
- **US-SU-7**: 사용자가 SPEC 카드 헤더에서 현재 git branch (`feature/SPEC-V3-009-spec-ui`) 가 활성인지를 색상으로 본다 → `git branch --show-current` + worktree state.
- **US-SU-8**: 사용자가 카드의 "Run" 버튼을 클릭하면 `moai run SPEC-XXX` subprocess 가 spawn 되고 stream-json 출력이 본문 하단 패널에 line-by-line stream 된다 → MoaiCommandClient + stream-json decoder.
- **US-SU-9**: 사용자가 SPEC-V3-003 의 본문을 보면 §10.1 ~ §10.5 Sprint Contract revisions 가 사이드 패널에 timeline 형태로 추출되어 보인다 → SprintContractPanel + regex 추출.

---

## 5. 기능 요구사항 (EARS)

### RG-SU-1 — SPEC document watch + parse

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-SU-001 | Ubiquitous | 시스템은 active workspace 의 `.moai/specs/SPEC-*/` 디렉터리를 1-depth 까지 스캔하여 `Vec<SpecRecord>` 를 생성한다. | The system **shall** scan `.moai/specs/SPEC-*/` directories of the active workspace into `Vec<SpecRecord>`. |
| REQ-SU-002 | Ubiquitous | 시스템은 각 SPEC 디렉터리에 대해 spec.md / plan.md / research.md / acceptance.md / contract.md / progress.md / tasks.md 의 존재 여부를 `SpecRecord.files: HashMap<SpecFileKind, Option<PathBuf>>` 로 기록한다. | The system **shall** record the presence of each canonical file under `SpecRecord.files`. |
| REQ-SU-003 | Event-Driven | spec.md 또는 progress.md 가 외부에서 변경되면, 시스템은 100ms debounce 후 해당 SpecRecord 를 재파싱하고 `cx.notify()` 를 호출한다. | When spec.md or progress.md is modified externally, the system **shall** re-parse and `cx.notify()` after 100ms debounce. |
| REQ-SU-004 | Ubiquitous | 시스템은 markdown AST parser 를 사용하여 spec.md 의 EARS 요구사항 표 (`\| REQ ID \| 패턴 \| ...`) 와 AC 표 (`\| AC ID \| ...`) 를 추출한다. parser 선택은 USER-DECISION-SU-A 의 결과를 따른다. | The system **shall** extract EARS requirement tables and AC tables from spec.md using the markdown AST parser chosen by USER-DECISION-SU-A. |
| REQ-SU-005 | Unwanted | 시스템은 acceptance.md 또는 다른 canonical 파일이 missing 인 SPEC 에서 panic 하지 않는다. UI 는 "no {filename}" placeholder 를 표시한다. | The system **shall not** panic when canonical files are missing; UI displays "no {filename}" placeholder. |

### RG-SU-2 — AC state tracker (FULL/PARTIAL/DEFERRED/FAIL/PENDING)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-SU-010 | Ubiquitous | 시스템은 `AcState` enum 으로 다음 5 상태를 정의한다: `Full`, `Partial`, `Deferred`, `Fail`, `Pending`. | The system **shall** define `AcState` enum with five states: `Full`, `Partial`, `Deferred`, `Fail`, `Pending`. |
| REQ-SU-011 | Event-Driven | 시스템이 progress.md 를 파싱할 때, `AC-{group}-{nnn}: (PASS\|FAIL\|DEFERRED\|PARTIAL\|PENDING)` 패턴 라인 또는 `\| AC-{...} \| ... \| (PASS\|FAIL\|...) \|` 표 셀을 인식하여 `AcRecord.state` 를 채운다. | When parsing progress.md, the system **shall** recognize either status line pattern or table cell pattern to populate `AcRecord.state`. |
| REQ-SU-012 | State-Driven | progress.md 에서 명시 status 가 발견되지 않은 AC 에 대해, 시스템은 `AcState::Pending` 을 default 로 설정한다. | While no explicit status is found, the system **shall** default `AcState` to `Pending`. |
| REQ-SU-013 | Ubiquitous | UI 는 각 AcState 에 대해 design token 색상을 매핑한다: Full=`status.success`, Partial=`status.warning`, Deferred=`text.tertiary`, Fail=`status.error`, Pending=`status.info`. | UI **shall** map each `AcState` to existing design tokens: Full=success, Partial=warning, Deferred=tertiary text, Fail=error, Pending=info. |
| REQ-SU-014 | Ubiquitous | 시스템은 SPEC 별 AC 요약 (예: `12/15 PASS, 2 PENDING, 1 FAIL`) 을 `SpecRecord::ac_summary()` 로 노출한다. | The system **shall** expose per-SPEC AC summary via `SpecRecord::ac_summary()`. |

### RG-SU-3 — Kanban board UI

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-SU-020 | Ubiquitous | 시스템은 `KanbanBoardView` 에서 4 lane (TODO / IN-PROGRESS / REVIEW / DONE) 을 가로로 배치한다. | The system **shall** lay out 4 lanes (TODO / IN-PROGRESS / REVIEW / DONE) horizontally in `KanbanBoardView`. |
| REQ-SU-021 | Ubiquitous | 시스템은 각 SPEC 의 stage 를 `.moai/specs/SPEC-XXX/.kanban-stage` (1-line text: `todo\|in-progress\|review\|done`) sidecar 파일에 persist 한다. 파일이 없으면 default `todo`. | The system **shall** persist each SPEC's stage in `.moai/specs/SPEC-XXX/.kanban-stage` sidecar file. Missing file defaults to `todo`. |
| REQ-SU-022 | Event-Driven | 사용자가 카드를 선택한 상태에서 Enter 키를 누르면, 시스템은 다음 stage (TODO → IN-PROGRESS → REVIEW → DONE → TODO 순환) 로 이동시키고 즉시 sidecar 에 write 한다. | When the user presses Enter on a selected card, the system **shall** advance to the next stage and write to sidecar immediately. |
| REQ-SU-023 | Event-Driven | 사용자가 ↑↓ 키로 lane 내 카드를 navigate 할 때, 시스템은 focus highlight 를 이동시키며 sidecar 변경은 없다. | When ↑↓ navigation, the system **shall** move focus highlight without writing sidecar. |
| REQ-SU-024 | Ubiquitous | 각 카드는 SPEC-ID + title + AC summary (RG-SU-2.4) + 활성 git branch (RG-SU-4) 를 표시한다. | Each card **shall** display SPEC-ID + title + AC summary + active git branch. |
| REQ-SU-025 | Ubiquitous | Lane header 는 lane 내 카드 수 (예: `IN-PROGRESS (3)`) 를 표시한다. WIP limit 은 정보성 hint 로만 표시 (hard 차단 없음). | Lane header **shall** show card count; WIP limit is informational hint only. |

### RG-SU-4 — SPEC ↔ git worktree 연동

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-SU-030 | Ubiquitous | 시스템은 `git branch --list 'feature/SPEC-*'` 와 `git branch --show-current` 결과를 파싱하여 SPEC-ID ↔ branch ↔ active 여부를 매핑한다. | The system **shall** parse `git branch --list 'feature/SPEC-*'` and `git branch --show-current` to map SPEC-ID to branch state. |
| REQ-SU-031 | Ubiquitous | 시스템은 CLAUDE.local.md §1.3 의 feature 명명 규칙 `feature/SPEC-{area}-{nnn}-{slug}` regex 를 인식한다. legacy 이름 (`feat/v3-scaffold`) 도 SPEC-ID 매핑 시도하되 매핑 실패 시 "unmatched" 로 표시. | The system **shall** recognize the `feature/SPEC-{area}-{nnn}-{slug}` regex; legacy names are best-effort matched and otherwise marked "unmatched". |
| REQ-SU-032 | State-Driven | 활성 branch 가 SPEC-XXX 의 feature branch 와 일치하는 동안, 해당 SPEC 카드는 헤더에 active indicator (예: `▶ feature/SPEC-V3-009-spec-ui`) 를 표시한다. | While the active branch matches the SPEC's feature branch, the SPEC card **shall** show an active indicator in its header. |
| REQ-SU-033 | State-Driven | SPEC 에 매칭되는 branch 가 없는 동안, 카드는 hint (`no branch — 'git checkout -b feature/SPEC-XXX-slug'`) 를 표시한다. | While no matching branch exists, the card **shall** show a hint suggesting `git checkout -b`. |

### RG-SU-5 — `/moai *` 슬래시 커맨드 1-클릭 호출

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-SU-040 | Ubiquitous | 시스템은 `MoaiCommandClient` 가 `moai run SPEC-XXX` / `moai plan ...` / `moai sync SPEC-XXX` 를 subprocess 로 spawn 한다. spawn 방식은 USER-DECISION-SU-C 의 결과 (subprocess+stream-json or MCP) 를 따른다. | The system **shall** spawn `moai run\|plan\|sync SPEC-XXX` via `MoaiCommandClient`; the dispatch mechanism follows USER-DECISION-SU-C. |
| REQ-SU-041 | Event-Driven | subprocess stdout 가 line 단위 JSON (stream-json 프로토콜) 을 출력하면, 시스템은 `crates/moai-stream-json::decode_line` 으로 SDKMessage 로 파싱하여 본문 하단 stream panel 에 append 한다. | When subprocess stdout emits a stream-json line, the system **shall** decode via `moai-stream-json::decode_line` and append to the stream panel. |
| REQ-SU-042 | Ubiquitous | UI 는 stream panel 갱신 시 60fps 를 초과하지 않도록 `cx.notify()` 호출을 16ms throttle 한다. | UI **shall** throttle `cx.notify()` to at most every 16ms when streaming updates. |
| REQ-SU-043 | Event-Driven | subprocess 가 종료되면, 시스템은 exit code 와 마지막 status 라인을 카드의 last-run badge 에 기록한다. exit_code != 0 인 경우 badge 는 `status.error` 색상. | When subprocess terminates, the system **shall** record exit code and last status to the card's last-run badge; non-zero exit colors the badge as error. |
| REQ-SU-044 | Unwanted | 시스템은 `moai` 바이너리가 PATH 에 없는 경우 panic 하지 않는다. UI 는 사용자에게 설치 가이드 링크와 함께 inline error 를 표시한다. | The system **shall not** panic when `moai` binary is absent from PATH; UI shows an inline error with install guidance. |
| REQ-SU-045 | State-Driven | 한 SPEC 에 대해 동시에 1 개 이상의 `moai run` 이 실행되는 동안, 시스템은 추가 Run 클릭을 disabled 처리하고 hint (`already running`) 를 표시한다. | While a `moai run` is in flight for a SPEC, the system **shall** disable additional Run clicks and show an `already running` hint. |

### RG-SU-6 — Sprint Contract 시각화 (10.x revision)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-SU-050 | Ubiquitous | 시스템은 spec.md 본문에서 `^## (\d+)\.(\d+) Sprint Contract Revision` regex 에 매칭하는 heading 을 모두 추출하여 `Vec<SprintContractRevision>` 을 생성한다. | The system **shall** extract all headings matching `^## (\d+)\.(\d+) Sprint Contract Revision` into `Vec<SprintContractRevision>`. |
| REQ-SU-051 | Ubiquitous | 각 `SprintContractRevision` 는 (a) section number, (b) title, (c) date (heading 본문 또는 첫 단락에서 ISO-8601 추출), (d) body markdown 4 필드를 가진다. date 추출 실패 시 `None`. | Each `SprintContractRevision` **shall** carry section number, title, date (ISO-8601, optional), and body markdown. |
| REQ-SU-052 | Ubiquitous | UI 는 `SprintContractPanel` 에서 추출된 revision 들을 timeline (가장 최근이 위) 으로 보여준다. 각 항목 클릭 시 본문이 해당 section 으로 scroll 된다. | UI **shall** show extracted revisions in `SprintContractPanel` as a timeline (most recent on top); clicking jumps body scroll. |
| REQ-SU-053 | State-Driven | 매칭되는 revision 이 0 개인 SPEC 에 대해, panel 은 `No sprint contract revisions yet.` 메시지를 표시한다. | While no revision matches, the panel **shall** display a `No sprint contract revisions yet.` message. |

---

## 6. Acceptance Criteria

| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 |
|------|--------------|----------|----------|---------|
| AC-SU-1 | `.moai/specs/SPEC-V3-009/` 디렉터리 자체가 SpecListView 에 1 개 카드로 등장 | 카드 ID 가 `SPEC-V3-009`, title 이 본 spec.md frontmatter 의 첫 H1 또는 SPEC 헤더와 일치 | unit (SpecIndex::scan), e2e (cargo run smoke) | RG-SU-1 |
| AC-SU-2 | `.moai/specs/SPEC-V3-003/spec.md` 의 EARS 표가 RG-P-1 ~ RG-P-12 + REQ-* 로 파싱되어 본문에 표시 | 파싱된 RG 개수 = 12, REQ 개수 >= 50 (실측 기준), 표 행 개수 일치 | unit (markdown parser test, fixture: SPEC-V3-003 스냅샷) | RG-SU-1 |
| AC-SU-3 | `.moai/specs/SPEC-V3-003/progress.md` 의 AC-P-* 상태가 FULL / PARTIAL / DEFERRED / FAIL / PENDING 으로 컬러 분류 | 5 상태 모두 fixture 에 등장 + UI 컬러가 design token 매핑과 일치 | unit (AcState 분류 테스트), visual (스냅샷 비교) | RG-SU-2 |
| AC-SU-4 | spec.md 가 외부에서 변경되면 100ms 이내에 본문이 자동 갱신 | watcher 통과 시간 측정 + cx.notify() 호출 1 회 (debounce 후), 측정 100ms 이내 (±50ms 허용) | integration (notify watcher + sleep + assert) | RG-SU-1 |
| AC-SU-5 | acceptance.md 가 없는 SPEC (예: SPEC-V3-001) 도 panic 없이 "no acceptance.md" 메시지와 함께 표시 | panic free + UI 에 placeholder 텍스트 존재 | unit (missing-file fixture) | RG-SU-1 |
| AC-SU-6 | KanbanBoardView 가 4 lane (TODO/IN-PROGRESS/REVIEW/DONE) 로 모든 SPEC 을 보임 | 4 lane element 존재 + 모든 SpecRecord 가 정확히 1 lane 에 배치 | unit (lane 분류) + e2e (visual smoke) | RG-SU-3 |
| AC-SU-7 | 카드 선택 후 ↑↓ + Enter 로 stage 변경 시 sidecar 에 persist + 재시작 후 복원 | sidecar `.kanban-stage` 파일 write 확인 + 재로드 시 동일 stage | integration (write + reload assert) | RG-SU-3 |
| AC-SU-8 | 활성 SPEC 의 git branch (`feature/SPEC-V3-009-spec-ui`) 가 카드 헤더에 표시 + branch 미존재 시 hint 표시 | 두 케이스 모두 fixture 로 검증, "unmatched" 케이스도 panic 없음 | unit (branch parser 테스트, mock git) | RG-SU-4 |
| AC-SU-9 | "Run" 버튼 클릭 시 `moai run SPEC-V3-009` subprocess 가 spawn 되어 stream-json 출력이 본문 하단 패널에 stream | mock subprocess 또는 echo subprocess 로 stream-json 라인 emit + 패널에 append 확인 | integration (subprocess + assert lines) | RG-SU-5 |
| AC-SU-10 | subprocess 종료 시 exit code + 마지막 status 라인이 카드에 반영 | exit_code=0 → 성공 색상, exit_code!=0 → error 색상 두 케이스 모두 검증 | integration | RG-SU-5 |
| AC-SU-11 | spec.md 의 `^## \d+\.\d+ Sprint Contract Revision` 헤더가 SprintContractPanel timeline 으로 추출 | SPEC-V3-003 fixture 에서 §10.1 ~ §10.5 모두 추출 (5 개), 빈 SPEC (V3-001) 에서 0 개 + placeholder | unit (regex + fixture) | RG-SU-6 |
| AC-SU-12 | terminal/panes/tabs core 코드 무변경 — 본 SPEC 은 신규 crate `moai-studio-spec` + 신규 모듈 `spec_ui/` 으로만 변경 | git diff 검증: `crates/moai-studio-terminal/` 0 byte change, `crates/moai-studio-ui/src/{terminal,panes,tabs}/` 0 byte change (RootView 진입점 등록 1 줄 제외) | CI assertion (path-filter diff check) | (cross-cutting) |
| AC-SU-17 | SpecListView 의 SPEC 카드가 AC summary 1줄 (`{full}/{total} PASS, {pending} PENDING, {fail} FAIL`) 외에 5 개 mini chip (FULL / PARTIAL / DEFERRED / FAIL / PENDING) + 각 카운트를 추가로 표시 | render_spec_card 결과에 5 개 chip element 가 정확히 등장 (count 0 chip 도 포함) | unit (chip element count + ac_state_color 매핑 검증) | RG-SU-2 |
| AC-SU-18 | 각 chip 의 색상이 detail_view::ac_state_color 매핑과 동일 (FULL → SUCCESS, PARTIAL → WARNING, DEFERRED → FG_MUTED, FAIL → DANGER, PENDING → INFO) | 5 매핑이 detail_view 의 단일 진실원 (single source of truth) 을 재사용 | unit (color 매핑 동치성 테스트) | RG-SU-2 |
| AC-SU-19 | AC summary 가 0 개인 SPEC 도 chip 5 개 (모두 0) 정상 렌더, panic 없음 | 빈 ac_records SpecRecord 의 render_spec_card 가 panic 없이 element 반환 | unit (empty fixture) | RG-SU-2 / RG-SU-1 (graceful) |
| AC-SU-20 | chip 카운트 표시 형식이 `{label}:{count}` (예: "FULL:3"). 0 카운트 chip 도 동일 형식 (visibility 우선) | render output 의 chip text 가 `LABEL:N` 패턴 매치 | unit (text format) | RG-SU-2 |

---

## 7. 비기능 요구사항

| 항목 | 요구 |
|------|------|
| 시작 시 SPEC 스캔 시간 | 50 SPEC 디렉터리 가정 시 200ms 이하 (lazy-load 병행, 본문 파싱은 클릭 시점) |
| 본문 자동 갱신 latency | 외부 변경 발생 시 100ms ± 50ms (RG-SU-1.3) |
| stream panel 갱신 frame rate | 60fps 이하 (16ms throttle, REQ-SU-042) |
| memory footprint | 50 SPEC 가정, 평균 SPEC 본문 50KB 시 baseline + 5MB 이하 |
| OS | macOS 14+, Ubuntu 22.04+ (Windows 비목표) |
| Rust toolchain | workspace `rust-toolchain` 그대로 사용 (현행 1.92+) |
| GPUI | `0.2.2` pin 유지 (SPEC-V3-001 부터 carry) |
| code_comments 언어 | `ko` (`.moai/config/sections/language.yaml`) |
| 새 design token | 추가하지 않음 (N9) |
| terminal/panes/tabs core | 변경 금지 (N6, RG-P-7 carry) |

---

## 8. 의존성 / 통합 인터페이스

### 8.1 선행 SPEC

- **SPEC-V3-004 (Render Layer)**: 본 SPEC 의 모든 UI 컴포넌트 (`SpecListView`, `SpecDetailView`, `KanbanBoardView`, `SprintContractPanel`) 가 GPUI Render trait 위에 빌드되므로 V3-004 의 `impl Render` 패턴 + RootView 배선이 PASS 한 후에 implement 진입.

### 8.2 병행 가능 SPEC

- **SPEC-V3-005 (File Explorer)**: `.moai/specs/` 디렉터리를 file explorer 에서 클릭 시 본 SPEC 의 SpecListView 로 라우팅. 인터페이스 = `fn open_spec(spec_id: SpecId)`.
- **SPEC-V3-006 (Markdown/Code Viewer)**: 본 SPEC 의 SpecDetailView 가 markdown 본문 렌더는 V3-006 의 markdown viewer 컴포넌트를 호출. 본 SPEC 은 metadata overlay (EARS / AC status badge) 만 추가.
- **SPEC-V3-010 (Agent Progress Dashboard)**: 카드의 "최근 agent run badge" 는 V3-010 의 AgentRun store 에서 read-only query. 인터페이스 = `fn last_run(spec_id: &SpecId) -> Option<AgentRunSummary>`.

### 8.3 외부 의존

- `crates/moai-stream-json` (재사용): `decode_line`, `SDKMessage` (RG-SU-5).
- `notify` v8 + `tokio::sync::mpsc` (신규 도입, but 워크스페이스에 이미 transitively 존재 가능).
- markdown parser: `pulldown-cmark` v0.13 또는 `comrak` v0.39 — USER-DECISION-SU-A.
- moai-adk Go CLI: PATH 의 `moai` 바이너리 — RG-SU-5. 부재 시 graceful fallback.

---

## 9. 마일스톤 (priority-based, 시간 추정 없음)

### MS-1 (Priority: High) — Parser + AC tracker + 기본 list view

산출:
- 신규 crate `crates/moai-studio-spec/` (parser + state + watch)
- `crates/moai-studio-ui/src/spec_ui/list_view.rs` + `detail_view.rs`
- USER-DECISION-SU-A 게이트 (markdown parser 선택)
- AC-SU-1, AC-SU-2, AC-SU-3, AC-SU-4, AC-SU-5 통과

### MS-2 (Priority: High) — Kanban board UI + 상태 transition

산출:
- `crates/moai-studio-ui/src/spec_ui/kanban_view.rs`
- sidecar persistence (`.moai/specs/SPEC-XXX/.kanban-stage`)
- USER-DECISION-SU-B 게이트 (DnD 라이브러리)
- AC-SU-6, AC-SU-7 통과

### MS-3 (Priority: High) — `/moai *` 호출 + worktree 연동 + Sprint Contract viewer

산출:
- `crates/moai-studio-ui/src/spec_ui/command_client.rs` (subprocess + stream-json)
- `crates/moai-studio-spec/src/branch.rs` (git branch parser)
- `crates/moai-studio-ui/src/spec_ui/sprint_panel.rs`
- USER-DECISION-SU-C 게이트 (subprocess vs MCP)
- AC-SU-8, AC-SU-9, AC-SU-10, AC-SU-11 통과
- 전체: AC-SU-12 (core 무변경) regression 검증

### MS-4 (Priority: Polish — v0.1.2 audit carry) — SPEC card polish + terminal click integration

audit feature-audit.md E-1 의 PARTIAL 상태 (full rendering / AC table / EARS parsing missing) 와 B-4 (terminal SPEC-ID hyperlink) 해소를 위한 polish milestone. sub-divided incremental PRs.

- **MS-4a** (PR #69 merged 2026-04-30) — B-4: TerminalClickEvent::OpenSpec → SpecPanelView::select_spec wiring. AC-SU-13~16 통과.
- **MS-4b** (이번 PR) — E-1: SpecListView card AC chip expansion (mini AC chips). render_spec_card 가 AC summary 1줄 외에 5개 chip (FULL/PARTIAL/DEFERRED/FAIL/PENDING) + 각 카운트 표시. detail_view::ac_state_color 재사용으로 색상 매핑 단일 진실원 유지. AC-SU-17~20 통과.
- **후속 carry**: SpecPanelView 안 master-detail 통합 (SpecDetailView 결합), AC inline expansion (popover 또는 expand 모드), E-7 Memory Viewer (별도 SPEC 후보).

---

## 10. USER-DECISION 게이트

### 10.1 USER-DECISION-SU-A — Markdown AST parser 선택 (MS-1 진입)

질문: "EARS / AC 표 + Sprint Contract heading 추출에 사용할 markdown parser 는?"

옵션:
- (a) **권장: pulldown-cmark v0.13** — rust-lang/mdbook 가 사용. 빠르고 deps 가벼움. event-driven (pull-based) 이지만 표 행 추출에 충분.
- (b) comrak v0.39 — GFM 완전 지원, AST 노드 직접 생성. 의존성 큼. 표 파싱 robust 하다는 의견.

영향 범위: REQ-SU-004, AC-SU-2.

### 10.2 USER-DECISION-SU-B — Kanban DnD 라이브러리 (MS-2 진입)

질문: "Kanban 카드 이동을 mouse drag-and-drop 으로 지원할 것인가?"

옵션:
- (a) **권장: 자체 구현 (keyboard-only first)** — MS-2 default. ↑↓ + Enter 로 stage 이동. mouse 는 follow-up SPEC.
- (b) GPUI on_mouse_* 로 자체 DnD 구현 — MS-2 범위 확장. 추가 100-200 LOC.
- (c) 외부 라이브러리 — GPUI 0.2.2 호환 라이브러리 미존재 (사실상 불가).

영향 범위: REQ-SU-022, REQ-SU-023, AC-SU-7.

### 10.3 USER-DECISION-SU-C — moai-adk 통합 패턴 (MS-3 진입)

질문: "moai-adk Go CLI 호출 방식은?"

옵션:
- (a) **권장: subprocess + stream-json** — `moai run SPEC-XXX` 를 spawn, stdout 라인을 `crates/moai-stream-json` 으로 decode. 기존 코덱 재사용, 단순.
- (b) MCP server pattern — moai-adk 가 MCP 서버 모드, moai-studio 가 MCP client. 양방향 streaming + capability discovery 가능하지만 layer 추가, moai-adk 측 server 구현 의존.

영향 범위: REQ-SU-040, REQ-SU-041, AC-SU-9, AC-SU-10.

---

## 11. 위험 (Risk Register)

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-SU-1 | SPEC 파일 schema drift (acceptance.md 부재 등) | 본문 로딩 실패 | RG-SU-1.5 (Unwanted) + AC-SU-5 |
| R-SU-2 | EARS / AC 표 markdown 변형 (spec.md vs progress.md) | 상태 추적 부정확 | RG-SU-2.1 의 dual pattern 인식 |
| R-SU-3 | subprocess streaming burst 로 UI stutter | 60fps 유지 실패 | REQ-SU-042 의 16ms throttle |
| R-SU-4 | moai-adk Go 의 unknown SDKMessage variant | decode 실패 | `serde(other)` fallback + raw text 표시 |
| R-SU-5 | Kanban stage persistence race (multi-pane 동시 변경) | 마지막 write 가 덮어씀 | sidecar file lock + last-write-wins + user 알림 |
| R-SU-6 | CLAUDE.local.md branch convention 변경 (v0.1.0 후) | branch 매핑 실패 | regex constants 를 `branch.rs` 한 곳에 모음 |

---

## 12. 외부 인터페이스 (불변 약속)

본 SPEC 은 다음 인터페이스를 fix 한다. 후속 SPEC 이 본 SPEC 의 산출물을 consume 할 때 신뢰할 수 있다:

```rust
// crates/moai-studio-spec/src/lib.rs (개념적 export)

pub struct SpecId(String);              // "SPEC-V3-009"
pub enum AcState { Full, Partial, Deferred, Fail, Pending }

pub struct SpecRecord {
    pub id: SpecId,
    pub title: String,
    pub files: HashMap<SpecFileKind, Option<PathBuf>>,
    pub requirements: Vec<RequirementGroup>,
    pub acceptance: Vec<AcRecord>,
    pub sprint_contract_revisions: Vec<SprintContractRevision>,
    pub branch: Option<BranchState>,
}

pub fn open_spec(spec_id: SpecId);                          // V3-005 join point
pub fn last_run(spec_id: &SpecId) -> Option<AgentRunSummary>; // V3-010 join point
```

후속 SPEC 이 변경할 수 없는 부분: enum variant 이름, struct 필드 이름. 신규 필드 추가는 가능 (semver minor).

---

## 13. 추적성

### 13.1 v2 SPEC-M5 ↔ 본 SPEC

| v2 의도 | v3 carrier |
|---------|------------|
| Frame 02 Kanban Board (lanes + cards) | RG-SU-3 + AC-SU-6/7 |
| 카드의 SPEC link badge | RG-SU-3.4 (REQ-SU-024) |
| Lane WIP limit | RG-SU-3.5 (REQ-SU-025, informational) |
| 카드의 Agent Run mini-graph | **deferred to SPEC-V3-010** (본 SPEC 비목표 N10) |

### 13.2 RG-P-7 carry (terminal/panes/tabs 무변경)

SPEC-V3-002 RG-P-7, SPEC-V3-003 RG-P-7, SPEC-V3-004 G6 → 본 SPEC G8 + AC-SU-12 로 carry.

---

## 14. 용어 정의

| 용어 | 정의 |
|------|------|
| `.moai/specs/SPEC-XXX/` | moai-adk 가 생성/관리하는 SPEC 디렉터리 단위. canonical 파일 셋: spec.md / plan.md / research.md / acceptance.md / contract.md / progress.md / tasks.md. |
| EARS 요구사항 | Easy Approach to Requirements Syntax. spec.md `\| REQ ID \| 패턴 \| ...` 표 행 단위. |
| AC | Acceptance Criteria. spec.md 또는 acceptance.md 의 `\| AC-{group}-{nnn} \| ...` 표 행 단위. |
| AcState | AC 의 진행 상태 5 분류: Full / Partial / Deferred / Fail / Pending. |
| Sprint Contract Revision | spec.md 의 `^## \d+\.\d+ Sprint Contract Revision` heading 으로 시작하는 sub-section. GAN Loop 의 산출. |
| sidecar | SPEC 디렉터리 안의 보조 파일. 본 SPEC 에서는 `.kanban-stage` (1-line text). |
| stream-json | Claude CLI / moai-adk Go 가 사용하는 line-delimited JSON 프로토콜. `crates/moai-stream-json` 가 코덱. |

---

## 15. 변경 이력 정책

본 spec.md 는 추가 revision 누적 시 `## 16. Sprint Contract Revisions` section 을 신설하고 `### 16.1 / 16.2 / ...` 로 누적한다 (SPEC-V3-003 §10.x 패턴 따름). RG-SU-6 의 self-application — 본 SPEC 자신이 본 SPEC 의 SprintContractPanel 의 첫 fixture 가 된다.

---

작성 종료. 본 spec.md 는 plan.md (구현 milestone × task) 와 함께 SPEC-V3-009 implement 진입의 입력이다. implement 는 별도 feature 브랜치 (`feature/SPEC-V3-009-spec-ui`) 에서 SPEC-V3-004 PASS 후 시작한다.
