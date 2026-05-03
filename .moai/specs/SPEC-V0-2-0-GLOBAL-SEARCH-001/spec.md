---
id: SPEC-V0-2-0-GLOBAL-SEARCH-001
version: 1.1.0
status: ready
created_at: 2026-05-01
updated_at: 2026-05-02
author: MoAI (manager-spec sess 8 + annotation iteration 1 sess 9)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-004, SPEC-V3-005, SPEC-V3-LINK-001, SPEC-V3-006]
parallel_with: [SPEC-V0-2-0-PLUGIN-MGR-001, SPEC-V0-2-0-MISSION-CTRL-001]
milestones: [MS-1, MS-2, MS-3, MS-4]
language: ko
labels: [v0.2.0, ui, gpui, multi-workspace, search, demo-visible, audit-D-4]
revision: v1.0.0 (initial draft, multi-workspace global search v1)
---

# SPEC-V0-2-0-GLOBAL-SEARCH-001: Global Search Across Workspaces — ⌘⇧F multi-workspace 콘텐츠 검색 + 결과 클릭 점프

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-01 | 초안 작성. v0.2.0 cycle 의 첫 신규 SPEC. design v3 D-4 + audit §4 Priority 1 #1 ⭐⭐⭐⭐⭐ "MoAI 의 multi-project 차별화 핵심" 의 v1 정의. 신규 crate `crates/moai-search/` (logic-only, GPUI-free) + `crates/moai-studio-ui/src/search/` (GPUI Panel + Result View). RG-GS-1 ~ RG-GS-7 의 7 개 요구사항 그룹 + AC-GS-1 ~ AC-GS-12 의 12 개 acceptance criteria + 3 USER-DECISION 게이트 (검색 엔진 / 신규 crate / 결과 cap) + 4 milestones (engine / UI / navigation / polish). gitignore 처리는 `ignore = "0.4"` crate, regex 매칭은 `regex = "1"`, cancel 은 `Arc<AtomicBool>` 패턴. 결과 navigation 은 SPEC-V3-LINK-001 의 `OpenCodeViewer` 패턴 재사용. Command Palette 진입은 `palette/registry.rs:154` 의 기존 `workspace.search` entry handler dispatch. |
| 1.1.0-ready | 2026-05-02 | annotation iteration 1 완료. 3 USER-DECISION 모두 권장 옵션 (a) 채택 lock-in: A=pure Rust (`ignore = "0.4"` + `regex = "1"`), B=신규 crate `crates/moai-search/`, C=per-file 50 / per-workspace 200 / total 1000. status: draft → ready. plan.md 작성 + MS-1 manager-tdd 위임 진입 가능. SearchPanel placement (사이드바), 결과 click 항상 새 tab (N13), input disabled when 0 workspace 결정 항목도 spec 본문 그대로 lock-in (변경 없음 — initial draft 가 권장 옵션 반영). |

---

## 1. 개요

### 1.1 목적

moai-studio v0.2.0 cycle 의 multi-project 차별화 핵심 — 여러 active workspace 에 걸친 파일 **콘텐츠** 검색 (file content grep, not just filename). VS Code Cmd+Shift+F 와 동등 UX 를 단일 코드베이스 (Rust + GPUI) 에서 구현.

활성 워크스페이스 전체의 파일 트리를 단일 input 으로 검색하고, 결과 entry 를 클릭하면 (a) 해당 workspace 활성화, (b) 해당 파일을 새 tab 으로 open, (c) 해당 line 으로 scroll 하는 e2e 플로우.

### 1.2 차별화 위치

design v3 spec.md v3.1.0 Tier D `D-4 Global search across workspaces` (Priority **High**, v0.1.2 GA 시점 status **NONE**). audit `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §4 Top 8 #1 등급 ⭐⭐⭐⭐⭐:

> MoAI 의 multi-project 차별화 핵심. ⌘⇧F → 전체 workspace 검색 → 결과 클릭 → tab 으로 점프. VS Code Cmd+Shift+F 와 동등 UX.

본 SPEC 은 그 v1 단일 채택 결정이며, `RootView::search_panel: Option<Entity<SearchPanel>>` 가 사이드바의 새 toggleable section 으로 추가된다.

### 1.3 근거 문서

- `.moai/specs/SPEC-V0-2-0-GLOBAL-SEARCH-001/research.md` — 4 검색 엔진 비교, 인덱싱 전략 분석, 다중 workspace 동시성 패턴, gitignore 처리, SearchPanel UI placement, 위험 평가, 미해결 결정 포인트.
- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier D, §4 Top 1, §10 carry table, §7 Sprint 4.
- `.moai/design/v3/spec.md` v3.1.0 Tier D (D-4), §7 IA Sidebar, §8 키바인딩 ⌘⇧F / Ctrl+Shift+F.
- `.moai/specs/SPEC-V3-LINK-001/spec.md` §4 — `OpenCodeViewer` struct 패턴 (본 SPEC 의 결과 click 동작 차용).
- `.moai/specs/SPEC-V3-005/spec.md` §6 — Render Entity 분리 패턴 (logic ↔ render) 차용.
- `crates/moai-studio-workspace/src/lib.rs:181` — `WorkspacesStore::list()` API.
- `crates/moai-studio-ui/src/palette/registry.rs:154` — `workspace.search` entry (이미 등록).
- `crates/moai-studio-ui/src/tabs/container.rs` — `TabContainer::new_tab` API.

---

## 2. 배경 및 동기

상세 분석은 `.moai/specs/SPEC-V0-2-0-GLOBAL-SEARCH-001/research.md` §1~§10 참조. 요구사항 진입 전 알아야 할 최소 맥락:

- **격차 분석** (research §1.3): `WorkspacesStore::list()` 와 `TabContainer::new_tab` 와 `palette/registry.rs::workspace.search` entry 는 모두 존재하지만, 검색 엔진 / multi-workspace walker / SearchPanel GPUI / 결과 navigation 모두 미존재. 본 SPEC 이 4 갈래 격차 수렴.
- **검색 엔진 결정** (research §2): 4 후보 (ripgrep subprocess / tantivy / GNU grep / pure Rust ignore+regex) 비교. 권장 = **pure Rust (`ignore = "0.4"` + `regex = "1"`)** — 외부 binary 의존 0, gitignore 정확 처리, in-process cancel.
- **인덱싱 전략** (research §3): v1 = **lazy** (background index 없음). v0.3.0+ 에서 eager 옵션 검토.
- **동시성** (research §4): worker per workspace, `ignore::WalkBuilder::build_parallel` 가 walker 내부 multi-thread, cancel = `Arc<AtomicBool>`.
- **결과 navigation** (research §5): SPEC-V3-LINK-001 의 `OpenCodeViewer { path, line, col }` 패턴 + `WorkspacesStore::touch(workspace_id)` 로 active 전환 + `TabContainer::new_tab(LeafKind::Code)`.
- **UI placement** (research §6): 사이드바 toggleable section (VS Code Cmd+Shift+F 패턴) + Command Palette `workspace.search` entry 양립.
- **신규 crate** (research §9): `crates/moai-search/` (logic-only) + `crates/moai-studio-ui/src/search/` (GPUI). Render Entity 분리 패턴.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `crates/moai-search/` (신규 crate) 가 GPUI 의존 없이 `SearchSession` / `SearchHit` / `SearchOptions` / `CancelToken` 을 정의하고 단위 테스트된다.
- G2. ⌘⇧F (macOS) / Ctrl+Shift+F (Linux/Windows) 단축키가 사이드바의 SearchPanel section 을 토글하고 input field 에 focus 한다.
- G3. 사용자가 query 를 입력하면 모든 active workspace (`WorkspacesStore::list()`) 의 파일에서 병렬 검색이 시작된다.
- G4. 검색 결과는 workspace 별로 grouped 되어 표시되며, 각 hit 은 (workspace name, file rel_path, line number, preview text) 를 포함한다.
- G5. 검색은 `.gitignore` / `.git/` / `target/` / `node_modules/` / `dist/` / `build/` / `.moai/state/` 패턴을 자동 제외한다.
- G6. 사용자가 결과 entry 클릭 시 (a) 해당 workspace 가 활성화되고, (b) 해당 파일이 새 tab 으로 열리고, (c) 해당 line 으로 점프한다.
- G7. 검색 진행 중 사용자가 cancel button 클릭 시 모든 worker 가 abort 되며, 진행 중이던 walk loop 이 cancel token check 시 즉시 중단된다.
- G8. Command Palette `⌘K` → `workspace.search` entry 선택 시 동일 SearchPanel 이 활성화된다 (직접 결과 표시 안 함, panel redirect).
- G9. 활성 workspace 가 1 개일 때도 동일 UI 가 동작하되, grouping 은 단일 그룹으로 표시된다.
- G10. 빈 query 입력 시 worker 는 spawn 되지 않고 결과 영역이 clear 된다.
- G11. macOS 14+, Ubuntu 22.04+, Windows 11 의 3 플랫폼에서 동일 기능 동작 (Windows 는 `cargo check` 수준만).

### 3.2 비목표 (Non-Goals) / Exclusions

본 SPEC 이 의도적으로 만들지 않는 것 (Exclusions — at least one entry, [HARD] manager-spec rule):

- N1. **Tantivy / 영속 인덱스**: v1 은 lazy walk only. background indexing + index storage + invalidation 은 별 SPEC (v0.3.0+ 검토).
- N2. **regex / case sensitivity / word boundary toggle UI**: v1 의 검색은 default case-insensitive substring + simple regex auto-detect (메타 문자 미포함 시 literal). UI toggle (Aa / .* / W) 는 v0.2.1 carry.
- N3. **glob include 패턴 (`*.rs` 만 검색)**: v1 미지원. v0.2.1 carry.
- N4. **결과 entry 의 drag-and-drop**: v1 의 결과 row 는 click only.
- N5. **cross-workspace 단일 tab**: 다른 workspace 의 결과 click 시 workspace 자체가 전환됨 (D-3 state preserve 정합). VS Code multi-root 같은 단일 window 에 다중 workspace tab 은 v0.3.0+.
- N6. **Replace (Find & Replace 의 Replace half)**: v1 은 검색 only. Replace 는 별 SPEC (v0.3.0+).
- N7. **검색 history / saved searches**: v1 미지원.
- N8. **binary file 검색**: v1 은 binary file 자동 skip (첫 8KB 에 NUL byte 검출 시).
- N9. **Windows GPUI e2e 검증**: cross-platform path 정규화 단위 테스트만 본 SPEC 책임. Windows GPUI 환경 e2e 는 후속 SPEC.
- N10. **인덱스 dropbox / disk 캐시**: lazy 채택 → 캐시 0.
- N11. **stream 결과의 incremental sort**: walker 가 streaming 으로 결과 송출하나 sort 는 100 hits 또는 1000ms 주기로 batch flush. perfect ordering 은 비목표.
- N12. **검색 결과 의 ANSI color highlight**: v1 의 preview text 는 plain text (matched substring 만 `<mark>` 또는 다른 색 강조). ripgrep --color=ansi 같은 ANSI escape 미사용.
- N13. **이미 같은 path 가 다른 tab 으로 열려 있을 때의 reuse 정책**: v1 은 항상 새 tab 생성 (단순). 추후 v0.2.1 reuse 정책 추가.
- N14. **새 design token 추가**: SPEC-V3-001 의 토큰 (`BG_SURFACE`, `FG_PRIMARY`, `ACCENT_MOAI`) 그대로 재사용. match highlight 색만 새 const 추가하되 token 모듈 구조는 유지.

---

## 4. 사용자 스토리

- **US-GS1**: 개발자가 여러 workspace 를 동시에 열어둔 상태에서 ⌘⇧F 누름 → 사이드바에 SearchPanel section 등장 + input focus → "TODO" 입력 → 모든 workspace 의 파일에서 매칭 결과가 workspace 별로 grouped 표시.
- **US-GS2**: 개발자가 결과 entry "moai-studio-A / src/main.rs:42 — `// TODO: implement`" 클릭 → workspace A 가 active 로 전환 + `src/main.rs` 가 새 tab 으로 open + 42 line 으로 scroll.
- **US-GS3**: 개발자가 큰 monorepo (5k 파일) 에서 너무 일반적인 query ("the") 를 입력 → 결과가 cap (1000) 도달 → "Too many results — narrow your query" 메시지 + 진행 중이던 worker 자동 cancel.
- **US-GS4**: 개발자가 검색 도중 다른 query 입력 → 이전 search session 자동 abort + 새 session 시작.
- **US-GS5**: 개발자가 Command Palette `⌘K` → `Search in Workspace` entry 선택 → 사이드바 SearchPanel 활성 + input focus (palette 자체에 결과 표시 안 함).
- **US-GS6**: 개발자가 검색 결과 cancel button 클릭 → 모든 worker abort + 현재까지 결과는 유지.
- **US-GS7**: 개발자가 활성 workspace 가 1 개일 때 ⌘⇧F → 동일 UI 동작, 결과는 단일 그룹 (workspace name) 아래 표시.
- **US-GS8**: 개발자가 binary file (e.g. PNG) 가 포함된 디렉터리에서 검색 → binary file 은 자동 skip, text file 만 결과에 등장.

---

## 5. 기능 요구사항 (EARS)

### RG-GS-1 — Search engine domain model (logic-only crate)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-001 | Ubiquitous | 시스템은 신규 crate `crates/moai-search/` 에 `SearchSession` / `SearchHit` / `SearchOptions` / `CancelToken` 4 개 핵심 타입을 정의한다. 모든 타입은 GPUI 의존 없이 `cargo build -p moai-search` 가 통과해야 한다. | The system **shall** define `SearchSession`, `SearchHit`, `SearchOptions`, `CancelToken` in new crate `crates/moai-search/`, GPUI-free. |
| REQ-GS-002 | Ubiquitous | `SearchHit` 는 `{ workspace_id: String, rel_path: PathBuf, line: u32, col: u32, preview: String, match_start: u32, match_end: u32 }` 를 노출한다. preview 는 매칭 라인 전체 (max 200 chars + ellipsis). | `SearchHit` **shall** expose workspace_id, rel_path, line, col, preview, match_start, match_end fields. |
| REQ-GS-003 | Ubiquitous | `SearchOptions` 는 `{ query: String, case_sensitive: bool, max_per_file: u32, max_per_workspace: u32, max_total: u32 }` 를 노출하며 디폴트는 `case_sensitive=false, max_per_file=50, max_per_workspace=200, max_total=1000` 이다. | `SearchOptions` **shall** expose configuration with defaults: case_sensitive=false, max_per_file=50, max_per_workspace=200, max_total=1000. |
| REQ-GS-004 | Ubiquitous | `CancelToken` 은 내부에 `Arc<AtomicBool>` 를 보유하며 `cancel()` / `is_cancelled()` 두 메서드를 노출한다. clone 가능하여 worker 들 간 공유 가능. | `CancelToken` **shall** wrap `Arc<AtomicBool>` and expose `cancel()` / `is_cancelled()`, cloneable for worker sharing. |
| REQ-GS-005 | Ubiquitous | 시스템은 `crates/moai-search/src/walker.rs` 에 `walk_workspace(root: &Path, opts: &SearchOptions, cancel: &CancelToken) -> impl Iterator<Item=SearchHit>` 를 정의한다. 내부적으로 `ignore::WalkBuilder` 를 사용해 gitignore 처리. | The system **shall** provide `walk_workspace` in `walker.rs` using `ignore::WalkBuilder` internally. |
| REQ-GS-006 | Unwanted | 시스템은 `walk_workspace` 가 `read_dir` 권한 오류 / I/O 오류 / regex compile 오류로 panic 하지 않는다. 오류는 `Result<Iterator, SearchError>` 또는 iterator 내부 silent skip + tracing warn. | The system **shall not** panic on errors; errors are returned as `SearchError` or silently skipped with tracing warn. |

### RG-GS-2 — gitignore + custom exclude handling

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-010 | Ubiquitous | 시스템은 `WalkBuilder::standard_filters(true)` 를 활성하여 `.gitignore`, `.ignore`, hidden files (`.git/`), global git excludes 를 자동 제외한다. | The system **shall** enable `standard_filters(true)` to auto-exclude .gitignore, .ignore, hidden files, and global git excludes. |
| REQ-GS-011 | Ubiquitous | 시스템은 hardcoded exclude pattern 으로 `target/`, `node_modules/`, `dist/`, `build/`, `__pycache__/`, `.venv/`, `.moai/state/`, `.moai/cache/` 를 추가한다. 이들은 `OverrideBuilder::add` 또는 `WalkBuilder::add_custom_ignore_filename` 으로 합성. | The system **shall** add hardcoded excludes for target/, node_modules/, dist/, build/, __pycache__/, .venv/, .moai/state/, .moai/cache/. |
| REQ-GS-012 | Event-Driven | 검색 worker 가 파일을 read 하기 전, 시스템은 첫 8KB 에서 NUL byte (`\x00`) 를 검출하면 binary 로 판단하여 skip 한다. | When a worker is about to read a file, the system **shall** check the first 8KB for NUL byte and skip if detected (binary file heuristic). |

### RG-GS-3 — Multi-workspace parallel search

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-020 | Event-Driven | 사용자가 SearchPanel input 에 trim 후 비어있지 않은 query 를 입력할 때, 시스템은 `WorkspacesStore::list()` 의 모든 workspace 에 대해 worker 를 spawn 한다. worker 는 `std::thread::spawn` (또는 `tokio::task::spawn_blocking`) 으로 실행되며, 결과는 `mpsc::channel(1024)` 로 송신한다. | When the user enters a non-empty trimmed query, the system **shall** spawn one worker per active workspace via spawn_blocking, streaming results through mpsc channel (capacity 1024). |
| REQ-GS-021 | Ubiquitous | 시스템은 worker 시작 시 `CancelToken::new()` 를 1 회 생성하여 모든 worker 와 SearchPanel 에 clone 을 배포한다. session 종료 시 자동 cancel. | The system **shall** create a single `CancelToken` per session and clone it to all workers and the SearchPanel. |
| REQ-GS-022 | Event-Driven | 사용자가 query 를 변경하거나 cancel button 을 클릭하거나 SearchPanel 을 close 할 때, 시스템은 현재 session 의 `CancelToken::cancel()` 을 호출하여 모든 worker 가 다음 cancel check 에서 abort 하도록 한다. | When the user changes the query, clicks cancel, or closes the panel, the system **shall** call `cancel()` to abort all workers at next check. |
| REQ-GS-023 | State-Driven | worker 의 walk loop 진행 중에는, 시스템은 매 file entry 도착 시 + 매 line search 시작 시 `cancel.is_cancelled()` 를 polling 하여 true 면 즉시 break 한다. | While the walk loop progresses, the system **shall** poll `is_cancelled()` per file entry and per line search, breaking immediately on true. |
| REQ-GS-024 | Ubiquitous | 시스템은 결과 hit 의 cap 을 (a) per-file `max_per_file=50`, (b) per-workspace `max_per_workspace=200`, (c) total `max_total=1000` 으로 적용한다. cap 도달 시 worker 는 추가 hit 을 송신하지 않으며, total cap 도달 시 session 전체가 자동 cancel + "Too many results" 메시지가 SearchPanel 에 표시된다. | The system **shall** apply hit caps per-file (50), per-workspace (200), total (1000); on total cap, the session is auto-cancelled with a user message. |

### RG-GS-4 — SearchPanel UI (sidebar section)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-030 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/search/panel.rs` (신규) 에 `SearchPanel` GPUI Entity 를 정의한다. SearchPanel 은 (a) input field, (b) cancel button, (c) result list (workspace 별 grouped), (d) status line (진행 중/완료/cancel/cap reached) 4 영역을 가진다. | The system **shall** define `SearchPanel` GPUI Entity in `search/panel.rs` with 4 regions: input field, cancel button, result list (workspace-grouped), status line. |
| REQ-GS-031 | Event-Driven | 사용자가 ⌘⇧F (macOS) 또는 Ctrl+Shift+F (Linux/Windows) 를 누를 때, 시스템은 사이드바의 SearchPanel section 을 visible 로 토글하고 input field 에 focus 를 준다. 같은 단축키 재입력 시 hide 토글. | When the user presses Cmd+Shift+F (macOS) / Ctrl+Shift+F (other), the system **shall** toggle SearchPanel visibility and focus the input field; same key again hides. |
| REQ-GS-032 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/search/result_view.rs` (신규) 에 result row rendering 을 정의한다. 각 row 는 2-line layout: line 1 = workspace_name + " / " + rel_path + ":" + line, line 2 = preview text (matched substring 강조). | The system **shall** define result row rendering in `result_view.rs` with 2-line layout (workspace_name/rel_path:line + preview with match highlight). |
| REQ-GS-033 | State-Driven | session 진행 중이지만 결과가 0 인 동안, SearchPanel 은 "Searching..." status + spinner 를 표시한다. session 완료 + 결과 0 일 때는 "No matches" 표시. | While the session is in progress with zero results, the panel **shall** show "Searching..." with spinner; on completion with zero results, show "No matches". |
| REQ-GS-034 | Event-Driven | 검색 결과 stream 도착 시, 시스템은 1000ms 또는 100 hits 단위 batch 로 SearchPanel 의 result list 를 갱신한다 (over-render 방지). | When result stream arrives, the system **shall** batch-flush the result list every 1000ms or per 100 hits (over-render prevention). |
| REQ-GS-035 | Ubiquitous | 시스템은 SearchPanel 의 input field 가 빈 string (또는 trim 후 빈) 일 때 worker 를 spawn 하지 않고 result list 를 clear 한다. | The system **shall not** spawn workers if the input is empty (or empty after trim); instead clear the result list. |

### RG-GS-5 — Result navigation (click → workspace activate + tab open + line jump)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-040 | Event-Driven | 사용자가 result row 를 클릭할 때, 시스템은 (a) 해당 hit 의 `workspace_id` 가 현재 active workspace 가 아니면 `WorkspacesStore::touch(workspace_id)` 로 active 전환하고, (b) `TabContainer::new_tab(LeafKind::Code(rel_path))` 으로 새 tab 을 생성하며, (c) CodeViewer 에 `(line, col)` scroll 명령을 dispatch 한다. | When the user clicks a result row, the system **shall** (a) call `WorkspacesStore::touch(workspace_id)` to activate the workspace if not active, (b) call `TabContainer::new_tab(LeafKind::Code(rel_path))` to open a new tab, (c) dispatch scroll-to-(line,col) to CodeViewer. |
| REQ-GS-041 | Ubiquitous | navigation 동작은 SPEC-V3-LINK-001 의 `OpenCodeViewer { path, line, col }` struct 패턴을 재사용한다. 본 SPEC 은 search-side adapter (`SearchHit` → `OpenCodeViewer`) 만 제공한다. | The system **shall** reuse SPEC-V3-LINK-001's `OpenCodeViewer` pattern; this SPEC provides the `SearchHit` → `OpenCodeViewer` adapter only. |
| REQ-GS-042 | Unwanted | 시스템은 navigation 동작이 (a) workspace 전환 실패, (b) 파일 read 실패, (c) tab 생성 실패 어느 단계에서도 panic 하지 않는다. 실패는 status bar + tracing warn 으로 보고되며 SearchPanel 자체는 계속 visible. | The system **shall not** panic on navigation failures; failures are reported to status bar with tracing warn, and SearchPanel remains visible. |

### RG-GS-6 — Command Palette integration

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-050 | Event-Driven | 사용자가 Command Palette `⌘K` → `Search in Workspace` entry (palette/registry.rs:154 `workspace.search`) 를 선택할 때, 시스템은 SearchPanel 을 visible 로 활성화하고 input field 에 focus 를 준다. palette 자체에는 결과 표시 안 함. | When the user selects `Search in Workspace` from Command Palette, the system **shall** activate SearchPanel and focus the input; no results in the palette itself. |
| REQ-GS-051 | Ubiquitous | 시스템은 `palette/registry.rs:154` 의 기존 `workspace.search` entry 의 label 을 "Search in all workspaces" 로 갱신한다 (multi-workspace 명확화). entry id / category / keybinding 은 변경 없음. | The system **shall** update the existing `workspace.search` entry label to "Search in all workspaces"; id/category/keybinding remain unchanged. |
| REQ-GS-052 | Ubiquitous | 시스템은 `palette/registry.rs` 의 `workspace.search` entry 에 `keybinding: Some("Cmd+Shift+F")` 를 추가한다 (현재 None). | The system **shall** set `workspace.search` entry's keybinding to `Cmd+Shift+F`. |

### RG-GS-7 — Single-workspace fallback + edge cases

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-GS-060 | State-Driven | 활성 workspace 가 0 개인 동안, SearchPanel 의 input field 는 disabled 상태로 표시되며 placeholder "Open a workspace to search" 가 보인다. | While zero workspaces are active, the input field **shall** be disabled with placeholder "Open a workspace to search". |
| REQ-GS-061 | Ubiquitous | 활성 workspace 가 1 개일 때, SearchPanel 은 동일 UI 를 사용하되 결과 grouping 은 단일 그룹 (workspace name) 으로 표시한다. | When exactly 1 workspace is active, the panel **shall** use the same UI but render results under a single group. |
| REQ-GS-062 | Ubiquitous | 시스템은 query 가 정확히 ASCII letters/digits/underscore/dot/dash 만 포함할 때 literal substring match 를 사용하고, regex meta character (`*`, `+`, `?`, `(`, `[`, `\`, `|`, `{`, `^`, `$`) 를 포함할 때 regex 매칭으로 fallback 한다. regex compile 실패 시 literal 로 다시 fallback. | The system **shall** use literal substring match for ASCII-only queries and regex match for queries with meta characters; on regex compile failure, fall back to literal. |

---

## 6. 비기능 요구사항

### 6.1 성능

- P1. 5k 파일 monorepo 의 단일 workspace 검색 (literal "TODO") 은 첫 hit 도착까지 < 50ms (macOS local SSD 기준).
- P2. 5k 파일 monorepo 의 단일 workspace 전체 walk + 검색 완료까지 < 500ms.
- P3. 3 workspace 동시 검색 (각 5k 파일) 의 첫 hit 도착까지 < 100ms.
- P4. SearchPanel 의 result list batch flush (100 hits 또는 1000ms 단위) 가 60Hz 프레임 보존 (mutation + cx.notify() < 16ms).
- P5. cancel token poll 비용은 무시 가능 (`AtomicBool::load(Relaxed)` ~ 1ns).

### 6.2 보안

- S1. 검색 worker 는 절대 workspace root 외부의 파일을 읽지 않는다 — symlink 가 root 외부를 가리킬 때 follow 안 함 (`WalkBuilder::follow_links(false)` 디폴트 사용).
- S2. binary file 자동 skip — 의도치 않은 secret file (e.g. `.pem`) 의 raw 내용이 preview 에 노출되지 않도록 NUL byte heuristic 적용.
- S3. preview text 는 max 200 chars 로 truncate — 매우 긴 minified line 의 raw 노출 방지.

### 6.3 cross-platform

- C1. `WalkBuilder` 는 `ignore` crate 의 cross-platform 동작 (Windows path separator, case-insensitive filesystem) 에 위임.
- C2. Windows path 정규화는 `crates/moai-studio-ui/src/explorer/path.rs::normalize_for_display` (SPEC-V3-005 제공) 재사용.
- C3. Windows 빌드는 본 SPEC v1 에서 `cargo check` 까지만 검증, GPUI 환경 e2e 는 별 SPEC.

### 6.4 변경 금지 zone (R 제약)

- R1. `crates/moai-studio-terminal/**` 무변경 (SPEC-V3-002 carry).
- R2. `crates/moai-studio-ui/src/explorer/**` 무변경 — 본 SPEC 은 별 search 모듈 도입. explorer/search.rs 는 단일 workspace 트리 fuzzy filter (filename only) 용이며 본 SPEC (file content grep) 와 별개.
- R3. `crates/moai-studio-ui/src/lib.rs` `RootView` 는 새 필드 추가만 허용 (`search_panel: Option<Entity<SearchPanel>>`). 기존 필드 rename / delete 금지.
- R4. `crates/moai-studio-workspace/src/lib.rs` `WorkspacesStore` 시그니처 변경 금지. `list()` / `touch()` read-only 사용 (touch 는 navigation 용으로 호출만).
- R5. `crates/moai-studio-ui/src/palette/registry.rs` `workspace.search` entry 의 label / keybinding 만 갱신 허용 (REQ-GS-051/052). id / category / 기타 entry 변경 금지.
- R6. `crates/moai-studio-ui/src/tabs/container.rs` `TabContainer::new_tab` 공개 API read-only 사용.
- R7. 기존 SPEC-V3-LINK-001 의 `OpenCodeViewer` struct 시그니처 변경 금지. 본 SPEC 은 그것을 import 만 함.

---

## 7. USER-DECISION 게이트 (annotation iteration 1, 2026-05-02 RESOLVED)

본 SPEC v1.1.0 은 3 USER-DECISION 모두 권장 옵션 (a) 로 lock-in 됨. 아래는 결정 history 기록 — 추후 v1.x 에서 재검토하려면 새 USER-DECISION ID 발급.

### 7.1 USER-DECISION-A — 검색 엔진 선택 ✅ RESOLVED 2026-05-02 = (a)

[USER-DECISION-RESOLVED: search-engine-v0-2-0-global-search-001 = (a) pure Rust]

질문 (research §2 권장 = (a)):
- (a) ✅ **결정 채택**: pure Rust (`ignore = "0.4"` + `regex = "1"`). 외부 binary 의존 0, gitignore 정확 처리, in-process cancel. 신규 dependency: `ignore`, `regex` (둘 다 BurntSushi ecosystem, mature).
- (b) ripgrep crate (`grep-cli` + `grep-searcher` + `grep-printer` library set). 더 풍부한 feature (printer / color), 동일 in-process. 신규 dependency 더 무거움. **REJECTED**.
- (c) ripgrep subprocess (사용자 시스템 또는 번들 binary). UX 손실 또는 distribution size 비용. **REJECTED**.
- (d) tantivy + background indexing (v1 inappropriate, v0.3.0+ 검토). **REJECTED for v0.2.0**.

채택 근거: distribution 단순 (외부 binary 0), `ignore` crate 가 ripgrep 본체와 동일 walker 로 gitignore 정확 처리, in-process cancel 단순 (`Arc<AtomicBool>`), v1 scope (5k 파일 monorepo < 500ms) 에 적합.

영향: plan.md MS-1 task list 가 `ignore::WalkBuilder` 기준으로 작성됨. workspace `[dependencies]` 에 `ignore = "0.4"`, `regex = "1"` 추가.

### 7.2 USER-DECISION-B — 신규 crate `moai-search` vs ui 모듈 통합 ✅ RESOLVED 2026-05-02 = (a)

[USER-DECISION-RESOLVED: search-crate-isolation-v0-2-0-global-search-001 = (a) 신규 crate]

질문 (research §9 권장 = (a)):
- (a) ✅ **결정 채택**: 신규 crate `crates/moai-search/` (logic-only, GPUI-free). future tantivy migration 용이. SPEC-V3-005 의 Render Entity 분리 패턴 일관.
- (b) `moai-studio-ui::search/` 모듈로 통합. 단순, dependency 적음. **REJECTED**.

채택 근거: logic ↔ render 분리로 unit test 단순화 (GPUI 환경 의존 없이 `cargo test -p moai-search` 가능), v0.3.0+ tantivy 마이그레이션 시 logic crate 만 교체, SPEC-V3-005 패턴 일관성.

영향: plan.md MS-1 가 신규 crate workspace member 추가 task 포함 (`Cargo.toml` workspace `members` 항목 추가).

### 7.3 USER-DECISION-C — 결과 cap 디폴트 값 ✅ RESOLVED 2026-05-02 = (a)

[USER-DECISION-RESOLVED: result-cap-defaults-v0-2-0-global-search-001 = (a) 50/200/1000]

질문:
- (a) ✅ **결정 채택**: per-file 50 / per-workspace 200 / total 1000. VS Code Cmd+Shift+F 의 기본 비슷.
- (b) per-file 100 / per-workspace 500 / total 5000. 더 관대. **REJECTED**.
- (c) per-file 20 / per-workspace 100 / total 500. 더 보수적. **REJECTED**.

채택 근거: VS Code 패턴 친숙성, 5k 파일 monorepo 에서 합리적 cap (over-render 방지), settings 로 user-overridable 은 v0.2.1 carry.

영향: REQ-GS-024 의 `SearchOptions` default constants 가 (50, 200, 1000) 으로 lock-in. AC-GS-6 검증 케이스 (50 파일 × 100 매칭 → total 1000 cap) 사용.

### 7.4 추가 검토 항목 (initial draft 그대로 lock-in)

본 SPEC initial draft 가 이미 권장 옵션을 채택했으므로 별 USER-DECISION ID 미발급:

- **SearchPanel placement** = 사이드바 toggleable section + Command Palette `workspace.search` entry redirect (research §6 권장). 본 SPEC §1.2 + RG-GS-4 lock-in.
- **결과 click 시 항상 새 tab** (N13 비목표). 같은 path tab reuse 정책은 v0.2.1 carry.
- **활성 workspace 0 일 때 input disabled** (REQ-GS-060). placeholder "Open a workspace to search".
- **검색 진행 중 query 변경 시 자동 abort + 새 session** (REQ-GS-022 + US-GS4). debounce 없음 (즉시 cancel + new session).
- **결과 row 의 keyboard navigation** = MS-4 polish 단계 추가 (위험 §11 항목). v1 MS-2/3 단계는 mouse click only.

---

## 8. Acceptance Criteria

본 SPEC 의 AC 는 spec.md §8 단일 출처 (acceptance.md 별 파일 미생성, SPEC-V3-005 와 동일 형식). 각 AC 의 PASS 조건은 `cargo test -p moai-search`, `cargo test -p moai-studio-ui --lib`, `cargo test -p moai-studio-ui --test integration_search` (신규), `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` 의 5 게이트 모두 GREEN.

| AC ID | 검증 대상 | 검증 방법 | Definition of Done |
|-------|-----------|-----------|-------------------|
| AC-GS-1 | RG-GS-1 REQ-GS-001~005 (engine domain model) | 단위 테스트: `crates/moai-search/src/lib.rs` 의 4 핵심 타입 (`SearchSession`, `SearchHit`, `SearchOptions`, `CancelToken`) 모두 import 가능 + 디폴트 생성 + clone 검증 | `cargo test -p moai-search` PASS, `cargo build -p moai-search` GPUI 의존 0 (Cargo.toml 에 gpui 미포함) |
| AC-GS-2 | RG-GS-1 REQ-GS-005 (walk_workspace) | 단위 테스트: tempdir 에 3 파일 (`a.rs` "use std", `b.rs` "use tokio", `c.txt` "hello") 생성 → `walk_workspace(root, query="use", ..)` → 2 hits 검증 (a.rs, b.rs) | 단위 테스트 PASS |
| AC-GS-3 | RG-GS-2 REQ-GS-010/011 (gitignore + custom exclude) | 단위 테스트: tempdir 에 `.gitignore` ("target/\nnode_modules/\n*.log") + 파일 (`src/a.rs`, `target/b.rs`, `app.log`, `node_modules/c.js`, `dist/d.html`) → query "use" → src/a.rs 만 hit, target/node_modules/dist/log 모두 skip | 단위 테스트 PASS |
| AC-GS-4 | RG-GS-2 REQ-GS-012 (binary skip) | 단위 테스트: tempdir 에 binary (`bin.dat` 첫 8KB 에 NUL byte 포함) + text (`a.rs` "use") → query "use" 또는 query "any" → bin.dat skip, a.rs hit | 단위 테스트 PASS |
| AC-GS-5 | RG-GS-3 REQ-GS-020/023 (cancel) | 단위 테스트: 100 파일 tempdir + worker spawn → 50ms 후 `cancel.cancel()` → worker 의 hit count 가 cap 미만 + total walk 시간 < (full walk 시간) 검증 | 단위 테스트 PASS, cancel 후 thread leak 없음 |
| AC-GS-6 | RG-GS-3 REQ-GS-024 (cap) | 단위 테스트: tempdir 에 50 파일 × 각 file 100 매칭 → query "x" → 결과 hit 수 = max_per_file × 50 = 2500 → total cap (1000) 도달 → session auto-cancel + status "Too many results" | 단위 테스트 PASS |
| AC-GS-7 | RG-GS-4 REQ-GS-030/031 (SearchPanel 토글) | integration: `cx.new(\|cx\| SearchPanel::new())` + ⌘⇧F dispatch → visibility true + input focus, 두 번째 ⌘⇧F → visibility false 검증 | USER-DECISION-B (a) 채택 시 GPUI e2e, (b) 채택 시 logic-level fallback unit |
| AC-GS-8 | RG-GS-4 REQ-GS-032/034 (result row + batch flush) | integration: 100 hits stream → batch flush 시 SearchPanel.result_count = 100 검증, 각 row 의 2-line layout (workspace_name/rel_path:line + preview) 렌더 검증 | USER-DECISION-B 결과에 따라 PASS |
| AC-GS-9 | RG-GS-4 REQ-GS-033/035 (status + empty query) | 단위 테스트: query="" → workers spawn 안 됨 + result_count = 0 + status = "Empty"; query="xyz" + 0 hits → status = "Searching..." → 완료 후 "No matches" | 단위 테스트 PASS |
| AC-GS-10 | RG-GS-5 REQ-GS-040/041 (navigation) | integration: SearchHit 클릭 → (a) WorkspacesStore::touch 호출 검증 (mock store), (b) TabContainer::new_tab 호출 검증 (mock container), (c) OpenCodeViewer { path, line, col } dispatch 검증 | USER-DECISION-B 결과에 따라 PASS |
| AC-GS-11 | RG-GS-6 REQ-GS-050~052 (Command Palette) | 단위 테스트: `palette/registry.rs` 의 `workspace.search` entry label = "Search in all workspaces", keybinding = Some("Cmd+Shift+F") 검증; entry select dispatch → SearchPanel.is_visible() = true | 단위 테스트 PASS |
| AC-GS-12 | RG-GS-7 REQ-GS-060/061/062 (edge cases) | 단위 테스트: (a) 0 workspace → input disabled + placeholder "Open a workspace to search"; (b) 1 workspace → result grouping 단일 그룹; (c) query "use_*" (regex meta) → regex match + 정상 동작; (d) query "/[invalid" → regex compile fail → literal fallback | 단위 테스트 PASS |

---

## 9. 의존 SPEC 정리

| SPEC | 관계 | 차용 / 분담 |
|------|------|-------------|
| SPEC-V3-001 | precedent | 셸 4 영역 (TitleBar / Sidebar / Body / StatusBar) 기존 — 본 SPEC 이 sidebar 의 새 toggleable section 추가 |
| SPEC-V3-004 | precedent | tab_container Entity 패턴 + workspace switch persistence — 본 SPEC 의 navigation 이 의존 |
| SPEC-V3-005 | precedent | Render Entity 분리 패턴 차용 (logic ↔ render); `explorer/path.rs::normalize_for_display` 재사용; explorer/search.rs (single-workspace fuzzy filter, filename only) 와 별개 (본 SPEC 은 multi-workspace content grep) |
| SPEC-V3-006 | dependent | CodeViewer surface (`viewer/code/`) 의 line/col scroll 지원 가정 |
| SPEC-V3-LINK-001 | dependent | `OpenCodeViewer { path, line, col }` struct 재사용 (R7 제약, 시그니처 변경 금지) |
| SPEC-V0-2-0-PLUGIN-MGR-001 | parallel (별 SPEC) | 무관, 동시 진행 가능 |
| SPEC-V0-2-0-MISSION-CTRL-001 | parallel (별 SPEC) | 무관, 동시 진행 가능 |

---

## 10. Milestone 매핑 (plan.md 와 동기 — 후속 작성)

| Milestone | 핵심 산출 | 검증 AC |
|-----------|-----------|---------|
| MS-1 | `crates/moai-search/` 신규 crate (USER-DECISION-A/B 결정 후) — `SearchSession` / `SearchHit` / `SearchOptions` / `CancelToken` + `walker.rs` (ignore::WalkBuilder + custom excludes) + `matcher.rs` (regex/literal fallback) + `cancel.rs` + 단위 테스트 | AC-GS-1, AC-GS-2, AC-GS-3, AC-GS-4, AC-GS-5, AC-GS-6 |
| MS-2 | `crates/moai-studio-ui/src/search/` 모듈 — `panel.rs` (SearchPanel GPUI Entity) + `result_view.rs` (2-line row rendering) + `mod.rs` + 사이드바 section toggle wire (`RootView::search_panel` 필드 추가) + ⌘⇧F dispatch | AC-GS-7, AC-GS-8, AC-GS-9, AC-GS-12 (edge cases UI 측) |
| MS-3 | navigation wire — SearchHit click → `WorkspacesStore::touch` + `TabContainer::new_tab` + `OpenCodeViewer` dispatch (SPEC-V3-LINK-001 adapter) + Command Palette `workspace.search` entry handler dispatch + label/keybinding 갱신 | AC-GS-10, AC-GS-11 |
| MS-4 | polish — backpressure (1000 hits cap → auto-cancel + message), progress indicator (per-workspace spinner), keyboard navigation (↑↓ result row), match highlight in preview text, integration test `tests/integration_search.rs` | 모든 AC 의 final regression sweep |

자세한 task 분해는 plan.md 후속 작성.

---

## 11. 위험 / 미해결 항목

- **Spike 0** (USER-DECISION-A 의 (a) 채택 시): `ignore = "0.4"` + `regex = "1"` workspace 추가 후 `cargo build -p moai-search` 통과 검증.
- **Spike 1** (USER-DECISION-B 의 (a) 채택 시): 신규 crate `crates/moai-search/` workspace member 등록 후 빌드 검증.
- **Spike 2** (gpui test-support, SPEC-V3-004/V3-005 의 carry-over 결정 일관성): SearchPanel 의 GPUI e2e AC (AC-GS-7/8/10) 가 GPUI 환경 검증 vs logic-level fallback 결정 — 본 SPEC 은 SPEC-V3-005 의 결정 (logic-level fallback) 을 따름.
- **위험 1** — `ignore::WalkBuilder` 의 cross-platform Windows path 동작 검증. Windows runner CI 는 `cargo check` 까지만 검증, e2e 는 별 SPEC.
- **위험 2** — 큰 monorepo (10k+ 파일) 검색 성능. P2 (< 500ms) 미달 시 MS-4 polish 단계에서 `WalkBuilder::threads(N)` 튜닝 또는 parallel walker option 검토.
- **위험 3** — workspace 전환 시 panes/tabs 잃어버림 (D-3 PARTIAL). v0.1.2 PR #64 의 round-trip 강화에 의존. 만약 잃어버림 사례 발견 시 본 SPEC 책임 외 (D-3 별 SPEC).
- **결과 row keyboard navigation** (REQ-GS-040 외 추가 요구): MS-4 polish 단계에서 ↑↓ keyboard 이동 + Enter open + Esc close 추가.
- **검색 history**: v1 비목표 (N7). v0.2.1 carry.

---

## 12. 영문 보조 요약 (Executive Summary)

SPEC-V0-2-0-GLOBAL-SEARCH-001 introduces multi-workspace global file content search for moai-studio v0.2.0, addressing audit Top 1 demo-visible feature D-4 (`Global search across workspaces`). Recommended search engine is **pure Rust** (`ignore = "0.4"` + `regex = "1"`), avoiding external binary dependency while matching ripgrep's gitignore accuracy. Indexing is **lazy** — `ignore::WalkBuilder` provides multi-thread walk; cancel via `Arc<AtomicBool>` token. A new crate `crates/moai-search/` (logic-only, GPUI-free) holds the engine; `crates/moai-studio-ui/src/search/` provides the GPUI **SearchPanel** mounted as a sidebar toggleable section (Cmd+Shift+F or Cmd+K → `Search in all workspaces`). Result click resolves via existing patterns: `WorkspacesStore::touch` activates workspace, `TabContainer::new_tab` opens file, SPEC-V3-LINK-001's `OpenCodeViewer { path, line, col }` jumps to line. Three USER-DECISION gates: search engine (pure Rust recommended), crate isolation (new crate recommended), result caps (per-file 50 / per-workspace 200 / total 1000 recommended). 7 EARS requirement groups, 12 acceptance criteria, 4 milestones (engine / UI / navigation / polish). Excluded from v1: tantivy index, eager indexing, regex/case-sensitivity toggles, glob include patterns, replace, history, cross-workspace single tab, drag-drop result, ANSI color highlight, tab reuse policy, Windows GPUI e2e, new design tokens.

---

작성 완료: 2026-05-01 sess 8
다음 산출: plan.md (milestone × task × file × AC 매핑) — 본 위임 scope 외, 후속 위임에서 작성.
