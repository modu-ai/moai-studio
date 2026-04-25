# SPEC-V3-009 Implementation Plan — SPEC Management UI

작성: MoAI (manager-spec, 2026-04-25)
브랜치 (현행 SPEC 작성): `feature/SPEC-V3-004-render`
브랜치 (implement 진입 시): `feature/SPEC-V3-009-spec-ui` (SPEC-V3-004 PASS 후 develop 에서 분기 — CLAUDE.local.md §1.3 명명 규칙 준수)
범위: SPEC-V3-009 spec.md 의 RG-SU-1 ~ RG-SU-6, AC-SU-1 ~ AC-SU-12 를 MS-1 / MS-2 / MS-3 으로 분할 구현.
선행: SPEC-V3-004 (render layer) PASS. V3-005 / V3-006 / V3-010 와 병행 가능.

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 영역 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|----------|----------------------|-----|----|
| **T0** | MS-1 | USER-DECISION-SU-A | (게이트, 결정 보고) | — | (게이트) |
| **T1** | MS-1 | 신규 crate scaffold | `crates/moai-studio-spec/Cargo.toml`, `src/lib.rs`, `Cargo.toml` (workspace members) | T0 | (구조) |
| **T2** | MS-1 | markdown parser (EARS / AC) | `crates/moai-studio-spec/src/parser/{mod,ears,ac}.rs` | T1 | AC-SU-2 |
| **T3** | MS-1 | AcState + AC summary | `crates/moai-studio-spec/src/state/{mod,ac.rs}` | T2 | AC-SU-3 |
| **T4** | MS-1 | SpecIndex + scan | `crates/moai-studio-spec/src/state/index.rs` | T3 | AC-SU-1, AC-SU-5 |
| **T5** | MS-1 | notify watcher + debounce | `crates/moai-studio-spec/src/watch.rs` | T4 | AC-SU-4 |
| **T6** | MS-1 | SpecListView + SpecDetailView | `crates/moai-studio-ui/src/spec_ui/{mod,list_view,detail_view}.rs` + `lib.rs` 진입점 | T5 | AC-SU-1 ~ AC-SU-5 |
| **T7** | MS-2 | USER-DECISION-SU-B | (게이트) | T6 | (게이트) |
| **T8** | MS-2 | sidecar persistence | `crates/moai-studio-spec/src/state/kanban.rs` | T7 | AC-SU-7 |
| **T9** | MS-2 | KanbanBoardView + keyboard nav | `crates/moai-studio-ui/src/spec_ui/kanban_view.rs` | T8 | AC-SU-6, AC-SU-7 |
| **T10** | MS-3 | USER-DECISION-SU-C | (게이트) | T9 | (게이트) |
| **T11** | MS-3 | branch parser | `crates/moai-studio-spec/src/branch.rs` | T10 | AC-SU-8 |
| **T12** | MS-3 | MoaiCommandClient | `crates/moai-studio-ui/src/spec_ui/command_client.rs` | T11 | AC-SU-9, AC-SU-10 |
| **T13** | MS-3 | sprint contract parser + panel | `crates/moai-studio-spec/src/parser/sprint_contract.rs`, `crates/moai-studio-ui/src/spec_ui/sprint_panel.rs` | T6 (병행 가능) | AC-SU-11 |
| **T14** | 전체 | core 무변경 regression | (CI assertion, path-filter diff) | T6 ~ T13 | AC-SU-12 |
| **T15** | 전체 | progress.md 갱신 + commit | (git 작업, sprint contract revision §10.1 추가) | T1 ~ T14 | (회수) |

---

## 2. T0 — USER-DECISION-SU-A (markdown AST parser 선택)

### 2.1 호출

[USER-DECISION-REQUIRED: markdown-parser-v3-009-ms1]

질문 (AskUserQuestion):
- "EARS / AC 표 + Sprint Contract heading 추출에 사용할 markdown parser 는?"
- (a) **권장: pulldown-cmark v0.13** — rust-lang/mdbook 가 사용. 빠르고 deps 가벼움. event-driven (pull-based) 이지만 표 행 추출에 충분. Cargo.toml 1 줄 추가.
- (b) comrak v0.39 — GFM 완전 지원, AST 노드 직접 생성. 의존성 큼 (transitive ~15). 표 파싱 robust.

### 2.2 결정 기록

option 결정 시 progress.md MS-1 entry 에 기록. parser 변경 시 T2 의 module API 는 동일 (`fn parse_spec_md(text: &str) -> ParsedSpec`).

---

## 3. T1 — 신규 crate scaffold

### 3.1 변경 대상

- 워크스페이스 `Cargo.toml` 의 `[workspace] members = [...]` 에 `"crates/moai-studio-spec"` 추가.
- `crates/moai-studio-spec/Cargo.toml` 신규:
  - `[package]` name = "moai-studio-spec", version = "0.1.0", edition = "2021"
  - `[dependencies]` — pulldown-cmark (or comrak per T0), regex, serde, serde_json, thiserror, tokio (sync, fs feature), notify, tracing
  - `[dev-dependencies]` — pretty_assertions, tempfile

- `crates/moai-studio-spec/src/lib.rs`:
  ```rust
  pub mod parser;
  pub mod state;
  pub mod watch;
  pub mod branch;

  pub use state::{SpecId, SpecRecord, AcState, AcRecord};
  pub use parser::{parse_spec_md, ParsedSpec};
  pub use watch::{SpecWatcher, SpecChangeEvent};
  ```

### 3.2 빌드 검증

`cargo build -p moai-studio-spec` 가 macOS 14+ / Ubuntu 22.04+ 양쪽에서 성공해야 한다. 단순 scaffold 만 있으므로 첫 PR 단계에서 실패가 없어야 한다.

---

## 4. T2 — markdown parser (EARS / AC)

### 4.1 EARS 요구사항 표 추출

대상: spec.md 본문에서

```markdown
| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-R-001 | Ubiquitous | ... | ... |
```

알고리즘:
1. markdown 을 event/node stream 으로 파싱.
2. `### RG-{group}-{nnn}` heading 직후의 첫 table 을 RG 의 requirements 표로 식별.
3. table header 의 첫 cell 이 `REQ ID` 인지 검증.
4. 데이터 행마다 `Requirement { id, pattern, korean, english }` 추출.

### 4.2 AC 표 추출

대상:

```markdown
| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 |
|------|--------------|----------|----------|---------|
| AC-SU-1 | ... | ... | ... | RG-SU-1 |
```

알고리즘:
1. table header 의 첫 cell 이 `AC ID` 또는 `AC` 로 시작하면 AC 표로 식별.
2. cells per row 가 4 또는 5 둘 다 허용 (RG 매핑 컬럼은 optional).

### 4.3 단위 테스트

- `tests/fixtures/spec_v3_003_snapshot.md` (실제 SPEC-V3-003 spec.md 의 sub-section 발췌) 를 fixture 로 두고 RG-P-1 ~ RG-P-12 + REQ-* + AC-P-* 의 개수 / id 일치 검증.
- malformed table (빈 행, missing pipe) 은 graceful skip + tracing warn.

---

## 5. T3 — AcState + AC summary

### 5.1 AcState enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AcState { Full, Partial, Deferred, Fail, Pending }

impl AcState {
    pub fn from_progress_label(label: &str) -> Self {
        match label.trim().to_uppercase().as_str() {
            "PASS" | "FULL" => AcState::Full,
            "PARTIAL" => AcState::Partial,
            "DEFERRED" => AcState::Deferred,
            "FAIL" => AcState::Fail,
            _ => AcState::Pending,
        }
    }
}
```

### 5.2 progress.md status 추출

두 패턴 모두 인식:
- 라인 패턴: `AC-SU-3: PASS` (regex `^AC-[\w-]+:\s*(\w+)`)
- 표 패턴: `| AC-SU-3 | ... | PASS |` (table 의 마지막 cell 또는 명시 컬럼)

### 5.3 AC summary

`SpecRecord::ac_summary()` 는 `AcSummary { full: u32, partial: u32, deferred: u32, fail: u32, pending: u32 }` 반환. UI 는 `12/15 PASS, 2 PENDING, 1 FAIL` 같은 문자열로 포맷.

---

## 6. T4 — SpecIndex + scan

### 6.1 SpecIndex

```rust
pub struct SpecIndex {
    pub specs: BTreeMap<SpecId, SpecRecord>,
    pub root: PathBuf,  // 보통 .moai/specs/
}

impl SpecIndex {
    pub fn scan(root: impl AsRef<Path>) -> Result<Self, SpecError>;
    pub fn reload_one(&mut self, spec_id: &SpecId) -> Result<(), SpecError>;
}
```

### 6.2 미싱 파일 graceful 처리

`SpecRecord::files: HashMap<SpecFileKind, Option<PathBuf>>` — 파일이 없으면 `None`. UI 는 `None` 인 경우 "no acceptance.md" placeholder 를 표시.

`SpecFileKind` enum 변형: `Spec, Plan, Research, Acceptance, Contract, Progress, Tasks`.

### 6.3 단위 테스트

- 12 개 SPEC 디렉터리 fixture 를 tempfile 로 복제 후 scan → 12 개 SpecRecord 생성, 각 SpecRecord 의 files 맵이 실제 디렉터리와 일치.
- acceptance.md 가 없는 SPEC-V3-001 fixture → `SpecFileKind::Acceptance => None`, panic 없음.

---

## 7. T5 — notify watcher + debounce

### 7.1 watcher API

```rust
pub struct SpecWatcher { /* ... */ }

pub enum SpecChangeEvent {
    SpecModified(SpecId),
    SpecCreated(SpecId),
    SpecRemoved(SpecId),
}

impl SpecWatcher {
    pub fn new(root: PathBuf) -> Result<(Self, mpsc::UnboundedReceiver<SpecChangeEvent>), SpecError>;
}
```

### 7.2 debounce

- notify 가 atomic write (vim/IDE) 시 multiple events 발생.
- internal: `last_event_at: HashMap<SpecId, Instant>` + 100ms 후 flush.

### 7.3 통합 테스트

- tempfile fixture 에서 spec.md 를 fs::write 로 modify → 100ms 후 SpecChangeEvent::SpecModified 1 회 (다중 emit 아님).
- 측정 latency 100ms ± 50ms 허용 (AC-SU-4).

---

## 8. T6 — SpecListView + SpecDetailView

### 8.1 신규 모듈

`crates/moai-studio-ui/src/spec_ui/mod.rs`:
```rust
pub mod list_view;
pub mod detail_view;
pub use list_view::SpecListView;
pub use detail_view::SpecDetailView;
```

### 8.2 SpecListView

```rust
pub struct SpecListView {
    index: Arc<RwLock<SpecIndex>>,
    selected: Option<SpecId>,
    // ... cx.observe(watcher) 로 SpecChangeEvent 수신
}

impl Render for SpecListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col()
            .child(self.toolbar(cx))
            .child(self.card_list(cx))
    }
}
```

각 카드 = `SPEC-{id}` + title + AC summary (status 컬러 4 점 dot row).

### 8.3 SpecDetailView

- spec.md 본문 markdown render 는 SPEC-V3-006 의 markdown viewer 컴포넌트 (병행 SPEC, 미구현 시 plain text fallback) 호출.
- 본문 위에 EARS 표를 status badge 로 augment.
- progress.md 의 AcState 를 본문의 AC 표 행에 컬러 dot 로 overlay.

### 8.4 RootView 진입점 등록

SPEC-V3-004 의 `tab_container` 와 동일 layer 로 RootView 에 1 줄 등록 (T14 의 core 무변경 정의: `lib.rs` 진입점 1 줄은 변경 허용, panes/tabs/terminal 모듈은 무변경).

---

## 9. T7 — USER-DECISION-SU-B (Kanban DnD 라이브러리)

### 9.1 호출

[USER-DECISION-REQUIRED: kanban-dnd-v3-009-ms2]

옵션:
- (a) **권장: 자체 구현 (keyboard-only first)** — MS-2 default. 추가 mouse DnD 는 follow-up SPEC.
- (b) GPUI on_mouse_* 로 자체 DnD — MS-2 범위 확장 (+100-200 LOC).
- (c) 외부 라이브러리 — GPUI 0.2.2 호환 부재 (사실상 reject).

### 9.2 (a) 선택 시 산출

- T9 의 KanbanBoardView 가 ↑↓ + Enter 만 처리.
- mouse 클릭은 카드 selection 만, drag 는 무시.

### 9.3 (b) 선택 시 산출

- T9 + drag handler 추가. `on_mouse_down` 으로 카드 정점 capture, `on_mouse_move` 로 drag indicator, `on_mouse_up` 시 drop target lane 결정.

---

## 10. T8 — sidecar persistence (.kanban-stage)

### 10.1 파일 schema

`.moai/specs/SPEC-XXX/.kanban-stage` — UTF-8 1-line text:
```
in-progress
```

valid values: `todo`, `in-progress`, `review`, `done`. unrecognized → fallback to `todo` + tracing warn.

### 10.2 read/write API

```rust
impl SpecRecord {
    pub fn read_kanban_stage(&self) -> KanbanStage;
    pub fn write_kanban_stage(&self, stage: KanbanStage) -> io::Result<()>;
}

pub enum KanbanStage { Todo, InProgress, Review, Done }
```

### 10.3 race 처리

- `tokio::fs::write` (atomic) — full-file replace.
- multi-pane 동시 변경 시 last-write-wins. user 에게 별도 알림 없이 watcher 가 변경을 reflect.

---

## 11. T9 — KanbanBoardView + keyboard navigation

### 11.1 4 lane 레이아웃

```rust
impl Render for KanbanBoardView {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_row()
            .child(self.lane(KanbanStage::Todo, cx))
            .child(self.lane(KanbanStage::InProgress, cx))
            .child(self.lane(KanbanStage::Review, cx))
            .child(self.lane(KanbanStage::Done, cx))
    }
}
```

### 11.2 keyboard navigation

- ↑↓: lane 내 카드 selection 이동.
- Enter (또는 Space): selected card 의 stage 를 next (Todo → InProgress → Review → Done → Todo) 로 advance + sidecar write.
- Tab: 다음 lane focus 이동.
- Esc: detail view 로 돌아감.

키 dispatch 는 SPEC-V3-003 의 `dispatch_tab_key` / SPEC-V3-004 의 keystroke 패턴을 reference (단, KanbanBoardView 자체는 `tab_container` 와 별 layer).

---

## 12. T10 — USER-DECISION-SU-C (subprocess vs MCP)

### 12.1 호출

[USER-DECISION-REQUIRED: moai-adk-integration-v3-009-ms3]

옵션:
- (a) **권장: subprocess + stream-json** — `moai run SPEC-XXX` spawn, stdout 라인을 `crates/moai-stream-json::decode_line` 으로 decode. 기존 코덱 재사용. 단순.
- (b) MCP server pattern — moai-adk 가 MCP 서버 모드. moai-studio 가 MCP client. 양방향 streaming + capability discovery 가능, 단 layer 추가 + moai-adk 측 server 구현 의존.

### 12.2 (a) 선택 시 산출

- T12 의 MoaiCommandClient 가 `tokio::process::Command` 로 spawn, `BufReader` 로 stdout line read, `decode_line` 호출.

### 12.3 (b) 선택 시 산출

- 본 SPEC 의 MS-3 범위 확장. MCP client crate 도입 (예: rmcp). moai-adk 측 MCP server 가 ready 인지 의존성 추가. 사실상 별 SPEC 으로 분리 권장.

---

## 13. T11 — branch parser

### 13.1 git command wrapper

```rust
pub struct BranchState {
    pub current: Option<String>,
    pub spec_to_branch: HashMap<SpecId, String>,
}

impl BranchState {
    pub fn detect(repo_root: &Path) -> Result<Self, BranchError>;
}
```

내부 구현:
- `git branch --show-current` → `current`.
- `git branch --list 'feature/SPEC-*'` → 각 라인을 regex `^[* ]+feature/(SPEC-[\w-]+)-([\w-]+)$` 로 파싱. SPEC-ID 추출 실패 시 "unmatched" 로 분류.

### 13.2 legacy 이름 처리

`feat/v3-scaffold` 같은 legacy branch 는 spec_to_branch 에서 "unmatched" key 아래 묶음. UI 는 hint 표시.

### 13.3 단위 테스트

- mock `git` (Command 인터페이스 추상화 또는 stdout fixture 직접 주입) 로 verifying.

---

## 14. T12 — MoaiCommandClient (subprocess + stream-json)

### 14.1 API

```rust
pub struct MoaiCommandClient { /* ... */ }

pub enum MoaiCommandEvent {
    Stdout(SDKMessage),     // moai-stream-json 의 SDKMessage
    StderrLine(String),
    Exit(ExitStatus),
    SpawnError(io::Error),
}

impl MoaiCommandClient {
    pub fn spawn_run(spec_id: SpecId) -> mpsc::UnboundedReceiver<MoaiCommandEvent>;
    pub fn spawn_plan(args: PlanArgs) -> mpsc::UnboundedReceiver<MoaiCommandEvent>;
    pub fn spawn_sync(spec_id: SpecId) -> mpsc::UnboundedReceiver<MoaiCommandEvent>;
}
```

### 14.2 stream 처리

- `tokio::process::Command::new("moai")` + args + `stdout(Stdio::piped())`.
- `BufReader::lines()` async stream.
- 각 line 을 `moai_stream_json::decode_line` 으로 decode → `SDKMessage` → `MoaiCommandEvent::Stdout`.
- decode 실패 시 raw line 을 `MoaiCommandEvent::StderrLine` 으로 fallback.

### 14.3 throttle

`cx.notify()` 호출은 `Instant` 기준 16ms 이내 중복 시 합치기. UI 패널 append 자체는 매 이벤트마다 진행.

### 14.4 already running 처리

`HashMap<SpecId, ChildHandle>` 으로 in-flight 추적. `spawn_run(spec_id)` 가 이미 entry 있으면 `Err(MoaiCommandError::AlreadyRunning)` 반환.

### 14.5 통합 테스트

- mock binary script (echo 가 stream-json 라인 emit) 를 PATH 에 두고 spawn 검증.
- exit_code 0 / 1 두 케이스 검증 (AC-SU-10).

---

## 15. T13 — sprint contract parser + panel

### 15.1 regex 추출

```rust
const SPRINT_CONTRACT_RE: &str = r"^## (\d+)\.(\d+) Sprint Contract Revision\b(.*)$";
```

multi-line mode + heading line 매칭.

### 15.2 SprintContractRevision struct

```rust
pub struct SprintContractRevision {
    pub section: (u32, u32),     // (10, 1) for §10.1
    pub title: String,
    pub date: Option<chrono::NaiveDate>,
    pub body: String,            // heading 다음 섹션 본문
}
```

date 추출:
- heading 본문에서 `\b(\d{4}-\d{2}-\d{2})\b` regex.
- 매칭 없으면 첫 단락에서 동일 regex.
- 모두 실패 시 `None`.

### 15.3 SprintContractPanel UI

- `crates/moai-studio-ui/src/spec_ui/sprint_panel.rs`.
- 본문 우측 또는 좌측 패널로 timeline (most recent on top).
- 각 항목 클릭 → SpecDetailView 가 해당 section 으로 scroll (intra-doc anchor).

### 15.4 단위 테스트

- SPEC-V3-003 spec.md 의 §10.1 ~ §10.5 fixture → 5 개 SprintContractRevision 추출.
- SPEC-V3-001 fixture (revision 없음) → 0 개 + UI placeholder "No sprint contract revisions yet."

---

## 16. T14 — core 무변경 regression

### 16.1 CI assertion

별도 GitHub Action step 또는 pre-merge 검사:

```bash
git diff --name-only origin/develop...HEAD \
  | grep -E '^crates/moai-studio-terminal/' && exit 1 || true
git diff --name-only origin/develop...HEAD \
  | grep -E '^crates/moai-studio-ui/src/(terminal|panes|tabs)/' \
  | grep -v 'lib.rs$' && exit 1 || true
```

`crates/moai-studio-ui/src/lib.rs` 의 1 줄 진입점 추가 (RootView 에 spec_ui 진입점 등록) 만 허용.

### 16.2 검증 시점

- 매 PR (feature/SPEC-V3-009-spec-ui → develop) 의 CI step 으로 등록.
- T15 commit 직전 local check.

---

## 17. T15 — progress.md + commit

### 17.1 progress.md 추가

본 SPEC implement 진입 시 `.moai/specs/SPEC-V3-009/progress.md` 신규 생성:
- MS-1 / MS-2 / MS-3 entry 별 AC 통과 상태 누적
- USER-DECISION-SU-A/B/C 결정 기록
- §10.1 Sprint Contract Revision (initial draft → MS-1 PASS) 추가

### 17.2 commit 정책 (현 SPEC 작성 단계)

본 plan.md / spec.md / research.md 의 commit 은:

```
docs(spec): SPEC-V3-009 SPEC Management UI v1.0.0 (research/plan/spec)

🗿 MoAI <email@mo.ai.kr>
```

브랜치: `feature/SPEC-V3-004-render` (현재). DO NOT push, DO NOT PR (사용자 지시).

### 17.3 commit 정책 (implement 진입 시)

- MS-1 PASS 시: `feat(spec-ui): SPEC-V3-009 MS-1 — parser + AC tracker + list view`
- MS-2 PASS 시: `feat(spec-ui): SPEC-V3-009 MS-2 — kanban board + sidecar persistence`
- MS-3 PASS 시: `feat(spec-ui): SPEC-V3-009 MS-3 — moai-adk integration + sprint contract panel`

각 commit 은 `🗿 MoAI <email@mo.ai.kr>` sign-off. CLAUDE.local.md §4.1 conventional commits 형식 준수.

---

## 18. 위험 완화 (구현 단계)

| 위험 (research §6) | 완화 task |
|------|----------|
| R-SU-1 schema drift | T4 + AC-SU-5 (SpecFileKind => Option) |
| R-SU-2 markdown 변형 | T2 + T3 의 dual pattern (line + table) 인식 |
| R-SU-3 streaming burst | T12 의 16ms throttle (REQ-SU-042) |
| R-SU-4 unknown SDKMessage | T12 의 `serde(other)` fallback + raw text |
| R-SU-5 stage persistence race | T8 의 atomic write + watcher reflect |
| R-SU-6 branch convention 변경 | T11 의 regex constants 한 곳 집중 |

---

## 19. 의존 SPEC 과의 join contract

| 병행 SPEC | 본 SPEC 이 노출 | 본 SPEC 이 consume |
|-----------|----------------|--------------------|
| SPEC-V3-005 (File Explorer) | `fn open_spec(spec_id: SpecId)` (RG-SU-1 진입점) | (없음) |
| SPEC-V3-006 (Markdown Viewer) | (없음, V3-006 컴포넌트를 호출) | `moai_studio_ui::markdown::MarkdownView` (V3-006 export) |
| SPEC-V3-010 (Agent Dashboard) | `fn last_run(spec_id: &SpecId) -> Option<AgentRunSummary>` (read-only contract) | `moai_studio_agent::AgentRunStore` (V3-010 export) |

V3-006 / V3-010 가 미구현 상태에서도 본 SPEC 은 plain-text fallback / `last_run = None` 으로 graceful 동작. 따라서 implement 순서에 강한 lock 없음.

---

## 20. 정리 및 다음 단계

1. 본 plan.md + spec.md + research.md 를 commit (T15.2 형식).
2. SPEC-V3-004 PASS 대기.
3. SPEC-V3-004 PASS 시 `feature/SPEC-V3-009-spec-ui` 분기 (develop 에서).
4. T0 USER-DECISION-SU-A 호출 → MS-1 진입.
5. MS-1/MS-2/MS-3 순차 진행. 각 MS 종결 시 progress.md + sprint contract revision §10.x 누적.

---

작성 종료. 본 plan.md 는 SPEC-V3-009 spec.md 의 Acceptance Criteria 12 개 + RG 6 개 + USER-DECISION 3 개를 task 15 개 + 3 milestone 으로 매핑한 실행 청사진이다.
