# SPEC-V0-2-0-GLOBAL-SEARCH-001 Research — Global Search Across Workspaces

작성: MoAI (manager-spec, 2026-05-01 sess 8)
브랜치 베이스: `main` HEAD `585ac8e` (v0.1.2 GA `1ce6b01d` 후속)
선행: SPEC-V3-001 (셸 4 영역), SPEC-V3-004 (workspace switcher D-2 skeleton), SPEC-V3-005 (File Explorer, fuzzy match 패턴 차용)
병행: SPEC-V0-2-0-PLUGIN-MGR-001 (별 SPEC, 본 SPEC 와 무관), SPEC-V3-009 (SPEC list, navigation 출력은 본 SPEC 의 click target 후보)
범위: design v3 spec.md `D-4 Global search across workspaces` 의 v1 구현. 전체 활성 workspace 의 파일 트리를 ⌘⇧F (macOS) / Ctrl+Shift+F (other) 로 검색, 결과 클릭 → workspace 활성화 + 파일 tab 점프.

audit reference: `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier D, §4 Top 8 #1 (Priority 1 ⭐⭐⭐⭐⭐), §10 carry table.

---

## 1. 동기 — D-4 가 v0.2.0 critical demo 인 이유

### 1.1 design v3 D-4 정의

`.moai/design/v3/spec.md` v3.1.0 Tier D Multi-Project Workspace `D-4 Global search across workspaces` (Priority **High**):

> 전체 활성 workspace 의 파일 트리를 단일 검색 창에서 검색. 결과 클릭 시 해당 workspace 활성화 + 파일 open + 라인 점프.

### 1.2 audit §4 Top 8 #1 등급

`.moai/specs/RELEASE-V0.2.0/feature-audit.md` §4 Priority 1 #1:

> **D-4 Global search across workspaces** ⭐⭐⭐⭐⭐
> - **Why**: MoAI 의 multi-project 차별화 핵심. ⌘⇧F → 전체 workspace 검색 → 결과 클릭 → tab 으로 점프. VS Code Cmd+Shift+F 와 동등 UX.
> - **Demo visibility**: HIGH (사이드바 + Command Palette 양쪽에서 진입).
> - **Scope**: ripgrep / tantivy 통합 + 결과 SearchResultView GPUI + click-to-navigate. 추정 ~800 LOC.
> - **SPEC**: SPEC-V0-2-0-GLOBAL-SEARCH-001 (신규).
> - **Risk**: 중간 (인덱싱 전략 결정 필요).

본 research 는 그 위 Risk 의 **인덱싱 전략 결정** 과 **검색 엔진 선택** 의 두 핵심 결정 포인트를 정리한다.

### 1.3 v0.1.2 GA 시점 격차 — D-4 NONE

audit §2 Tier D 매트릭스 `D-4 | NONE | (no implementation). v0.2.0 critical demo 후보`. 코드베이스 grep:

| 자산 | 위치 | 상태 |
|------|------|------|
| `WorkspacesStore::list()` | `crates/moai-studio-workspace/src/lib.rs:181` | ✅ active workspace 의 `&[Workspace]` 반환 — 본 SPEC 의 검색 도메인 source |
| `Workspace::project_path` | 동 lib.rs:34 | ✅ 각 workspace 의 root path |
| `crates/moai-studio-ui/src/explorer/search.rs` | 11.7KB `fuzzy_match()` + `apply_filter()` | ✅ 단일 workspace 트리의 fuzzy filter — **파일 이름** 매칭 만, 본 SPEC 은 **파일 내용** 매칭 |
| `crates/moai-studio-ui/src/palette/registry.rs` | 11.7KB `CommandRegistry` 30+ commands | ✅ `workspace.search` entry 이미 등록 (line 154 `Search in Workspace`) — 본 SPEC 의 Command Palette 진입점 |
| `crates/moai-studio-ui/src/tabs/container.rs` | 31KB `TabContainer` | ✅ tab CRUD API — 본 SPEC 의 결과 click → 새 tab open target |
| Search Engine integration | — | ❌ 미존재 — 본 SPEC 도입 |
| Multi-workspace parallel walk | — | ❌ 미존재 |
| SearchPanel sidebar surface | — | ❌ 미존재 |
| 결과 click → workspace activate + tab open + line jump | — | ❌ 미존재 |

격차는 4 갈래 — (a) 검색 엔진, (b) 다중 workspace 동시 walk + cancel, (c) GPUI SearchPanel 렌더, (d) 결과 navigation (workspace switch + new tab + line jump).

### 1.4 사용자 가시 정의 (escape hatch)

본 SPEC PASS 시 `cargo run -p moai-studio-app` 으로 직접 관찰 가능:

1. ⌘⇧F (macOS) 또는 Ctrl+Shift+F (Linux/Windows) 누름 → 사이드바에 SearchPanel 등장 + input focus.
2. "TODO" 입력 → 모든 active workspace 의 파일에서 "TODO" 매칭 결과가 workspace 별로 grouped 표시.
3. `.git/`, `target/`, `node_modules/`, `.moai/state/` 등은 결과에서 제외됨.
4. 결과 entry "moai-studio-A / src/main.rs:42 — `// TODO: implement`" 클릭 → workspace A 가 active 로 전환 + `src/main.rs` 가 새 tab 으로 open + 42 line 으로 점프.
5. 검색 진행 중 cancel button 클릭 → 모든 worker abort + 현재까지 결과는 유지.
6. Command Palette `⌘K` → "Search in all workspaces" entry 선택 → 동일 SearchPanel 활성.

---

## 2. 검색 엔진 선택 — 4 후보 비교

본 SPEC 의 핵심 결정. 4 후보의 장단점:

### Option A: ripgrep (subprocess wrapper)

`ripgrep` binary 를 subprocess 로 spawn 하여 결과 stdout 을 stream parse.

**장점**:
- 가장 빠른 grep (Rust 진영 표준, 99% 의 케이스에서 grep / silver-searcher 보다 빠름)
- `.gitignore` / `.ignore` 자동 처리 (--no-ignore 로 비활성 가능)
- mature regex (PCRE2 옵션, 기본은 Rust regex)
- JSON output mode (`--json`) 로 stream parse 용이
- 사용자가 이미 ripgrep 설치된 경우 추가 의존 없음

**단점**:
- 외부 binary 의존 — 사용자 시스템에 `rg` 가 없으면 동작 안함 (또는 번들)
- 번들 시 macOS .app / Linux AppImage / Windows .exe 에 ~5MB binary 추가
- subprocess 관리 (spawn, cancel via SIGTERM, parse JSON stream) 복잡도

**번들 전략 후보**:
- (a) ripgrep crate 로 in-process 사용 (`grep-cli` + `grep-searcher` + `grep-printer` library) — binary 없이 link
- (b) subprocess + 사용자가 별도 설치 요구 — 의존성 0 이지만 UX 손실
- (c) subprocess + 번들 binary — 가장 단순, distribution size 비용

### Option B: tantivy (Rust native full-text index)

Lucene-like full-text search index. workspace 추가 시 background indexing.

**장점**:
- Rust pure (외부 binary 0)
- 인덱스 구축 후 sub-ms query — 큰 monorepo 에서도 빠름
- highlight / faceted search / scoring 등 풍부한 feature
- gitignore 처리 별도 구현 필요

**단점**:
- 인덱스 구축 비용 (큰 monorepo 의 첫 인덱싱은 수 분)
- 인덱스 storage (~/.moai/studio/index/{workspace_id}/) 관리 — disk 비용 + invalidation
- file watch 와 인덱스 동기화 (notify event → 부분 reindex) 복잡도
- v1 단순 검색에 over-engineering — D-4 의 v1 목표는 "VS Code Cmd+Shift+F 와 동등" 이지 "Algolia 수준" 은 아님

### Option C: GNU grep (subprocess)

전통적 grep binary subprocess.

**장점**: 가장 보편적, 의존성 0 (모든 Unix 에 존재).
**단점**: ripgrep 대비 느림, gitignore 미처리, Windows 부재 (별도 grep 필요), regex 변종 다양 (BSD grep vs GNU grep).

### Option D: Pure Rust (ignore + walkdir + regex + memchr)

`ignore` crate (BurntSushi, ripgrep 의 walker) + `regex` crate + `memchr` 직접 조합.

**장점**:
- Rust pure, 외부 binary 0
- `ignore` crate 가 gitignore / .ignore / exclude pattern 모두 처리 (ripgrep 본체와 동일 walker)
- regex crate 사용 → Rust regex 와 모든 곳 동일 syntax
- 의존성 추가 작음 (`ignore` 주요, 나머지는 std 또는 기존 사용)
- in-process 이므로 cancel 은 단순 `Arc<AtomicBool>` cancel flag

**단점**:
- ripgrep 본체 만큼 빠르진 않음 (단일 파일 검색은 비슷, 큰 trees 에서는 ripgrep 이 우세 — 단 v1 scale 에서는 차이 미미)
- 자체 검색 loop 구현 필요 (~150 LOC)

### 권장: **Option D (ignore + walkdir + regex + memchr)**

근거:
1. **외부 binary 의존 0** — distribution 단순. macOS .app / Windows .exe / Linux AppImage 모두 추가 binary 불필요.
2. **`ignore` crate** 가 ripgrep 본체의 walker 와 동일 로직 — gitignore / .ignore / hidden file / max depth / file type filter 모두 처리. ripgrep 와 동등 정확도.
3. **in-process** — cancel 은 단순한 Rust pattern (`CancelToken` / `Arc<AtomicBool>`), subprocess 관리 복잡도 0.
4. **scope 적합** — D-4 v1 은 monorepo (10k 파일) 검색 < 500ms 가 목표. `ignore::WalkBuilder + regex::Regex::find_iter` 로 충분히 달성 가능.
5. **Rust regex** 동일 syntax — terminal link.rs 의 regex 와 일관성.
6. **번들 의존성**: `ignore = "0.4"`, `regex = "1"`, `walkdir = "2"` (이미 plugin-installer 에 존재). `memchr` 는 regex 가 내부 사용 → 직접 추가 불필요.

대안 검토:
- ripgrep crate (`grep-cli` / `grep-searcher`) 는 더 풍부한 기능 (printer, color highlight) 제공. v2 cycle 에서 성능 병목 발견 시 마이그레이션 후보. v1 scope 외.
- tantivy 는 v0.3.0+ 에서 "프로젝트 인덱싱" 옵션 추가 시 검토.

---

## 3. 인덱싱 전략 — 3 옵션

### Option A: lazy (on-demand walk)

검색 시점에 모든 active workspace 의 파일을 walk + grep. 인덱스 0.

**장점**: 단순, storage 0, watcher 불필요.
**단점**: 큰 monorepo (5k+ 파일) 에서 매 검색마다 walk → 사용자 가시 지연.

### Option B: eager (background index)

workspace 추가 시 background indexing. tantivy 또는 자체 hashmap.

**장점**: 검색 sub-ms.
**단점**: 인덱싱 비용 + storage + watcher 동기화 복잡도.

### Option C: hybrid

lazy default + 옵션으로 eager. 사용자가 큰 workspace 에서 toggle.

### 권장: **Option A (lazy)**

근거:
1. v1 scope — VS Code Cmd+Shift+F 도 lazy 동작 (인덱스 없음, ripgrep 호출).
2. `ignore::WalkBuilder` 의 multi-thread walker 는 병렬 traversal — 5k 파일 monorepo 도 수십 ms 내 walk 완료.
3. tantivy 인덱스 도입은 invalidation 복잡도 (notify event → partial reindex) 가 v1 목표 대비 과다.
4. v0.3.0+ 에서 "프로젝트 인덱싱" feature toggle 로 eager 옵션 추가 가능 (미래 SPEC).

---

## 4. 다중 workspace 동시성 패턴

### 4.1 worker per workspace

각 active workspace 별 1 worker (tokio task). 결과는 `mpsc::channel(SearchHit)` 로 GPUI thread 로 stream.

```text
┌─ worker(ws-A) ─► WalkBuilder + regex ─► tx.send(hit) ─┐
├─ worker(ws-B) ─► WalkBuilder + regex ─► tx.send(hit) ─┤── rx ── GPUI ─► SearchPanel re-render
└─ worker(ws-C) ─► WalkBuilder + regex ─► tx.send(hit) ─┘
                                            ▲
                                       cancel_token
                                       (Arc<AtomicBool>)
```

### 4.2 cancel 패턴

- 사용자가 새 query 입력 → 이전 search session abort.
- cancel token = `Arc<AtomicBool>`. worker 의 walk 루프에서 매 entry / 매 line 마다 `cancel_token.load(Ordering::Relaxed)` 체크 → true 면 즉시 break.
- cancel button 클릭 → 동일 mechanism.
- workspace 전환 시 자동 cancel.

### 4.3 backpressure

- mpsc channel capacity 1024.
- 결과 폭주 시 (e.g. "the" 같은 매우 일반적 단어) hard cap 1000 hits 후 cancel + "Too many results — narrow your query" message.
- per-workspace cap: 200 hits.
- per-file cap: 50 hits (extreme cases 방지).

### 4.4 결과 ordering

- workspace order: WorkspacesStore.list() 순서 유지 (= last_active 정렬).
- 같은 workspace 내: file path lexical order.
- 같은 file 내: line number 순.
- streaming 이므로 incremental — 매 hit 도착 시 sort + re-render 는 비용 — 1000ms 마다 batch flush 또는 100 hits 마다 flush.

### 4.5 Rayon vs tokio

- `ignore::WalkBuilder::build_parallel` 는 자체 thread pool 사용 (rayon-style).
- 본 SPEC 는 `WalkBuilder::build_parallel(visit_fn)` 를 사용해 walker 자체가 multi-thread. 각 worker 는 std::thread::spawn (또는 tokio::task::spawn_blocking) 으로 walk + 결과는 std::sync::mpsc 로 → GPUI bridge.
- async tokio 채택 안 함 — search 는 CPU bound (fs I/O + regex) 이며 async 의 이득 작음. spawn_blocking 이 적합.

---

## 5. 결과 click navigation

### 5.1 navigation 단계

1. `SearchHit { workspace_id, rel_path, line, col, preview }` click.
2. `WorkspacesStore::touch(workspace_id)` → last_active 갱신 + active workspace 전환.
3. RootView 에 active workspace 변경 통지 → tab_container reload (workspace 전환 시 기존 panes / tabs persistence 적용).
4. `TabContainer::new_tab` (또는 기존 open 된 tab 재사용) 으로 `LeafKind::Code(rel_path)` tab 생성.
5. CodeViewer 에 line/col scroll 명령 dispatch.

### 5.2 기존 SPEC 와의 시너지

- **SPEC-V3-LINK-001**: terminal click → CodeViewer 점프 패턴 이미 존재 (`OpenCodeViewer { path, line, col }` struct). 본 SPEC 의 navigation 도 동일 path 사용.
- **SPEC-V3-004**: tab_container 의 active workspace 전환 시 panes/tabs 복원 패턴 존재 (PaneLayoutV1.active_tab_idx round-trip).
- **SPEC-V3-006 / V3-005**: CodeViewer surface (`viewer/code/`) 가 line/col scroll 지원.

### 5.3 신규 결정 포인트

- 결과 click 이 (a) 새 tab 으로 open vs (b) 기존 tab 재사용 (같은 path 가 이미 열려 있다면) — VS Code 는 (b) 우선. 본 SPEC v1 도 (b) 우선, fallback (a).
- 다른 workspace 결과 click 시 (a) workspace 전환 후 새 tab vs (b) 현재 workspace 의 별 tab 으로 다른 workspace 의 파일 open — VS Code multi-root 는 (b) 도 가능. 본 SPEC v1 은 (a) 만 — workspace 격리 (D-3 state preserve) 정합성.

---

## 6. SearchPanel UI placement

design v3 §7 IA `Sidebar (260pt, toggleable)` 에 `Workspace Switcher / Current Project Section / [+ New Pane]` 만 정의됨. SearchPanel 추가 위치:

### Option A: Sidebar 의 toggleable section (VS Code Cmd+Shift+F 패턴)

기존 sidebar 의 sections (Workspace Switcher / Panes Tree / SPECs / Worktrees / Recent Files) 옆 또는 위에 SearchPanel section toggle.

**장점**: VS Code 와 동등 UX — 사용자가 익숙. side-by-side 사용 (검색 + 트리 동시).
**단점**: 사이드바 폭 (260pt) 부족 — preview text + path 렌더 좁음.

### Option B: 별 Surface (검색 결과를 새 tab 으로)

검색 결과를 `LeafKind::Search` 로 새 tab 에 mount. main pane area 사용.

**장점**: 폭 넉넉, 결과 풍부 표시 가능.
**단점**: tab 생성 → 사용자가 검색 후 tab close 필요. VS Code 와 다른 UX.

### Option C: Command Palette dropdown

⌘⇧F → palette overlay 에 결과 표시 (max 10-20).

**장점**: 단순.
**단점**: 큰 결과 셋 부적합. preview 좁음.

### 권장: **Option A + Option C 양립**

- 메인 진입: Option A — 사이드바 SearchPanel section toggle (⌘⇧F 단축키).
- 보조 진입: Option C — Command Palette `Search in all workspaces` entry → SearchPanel 활성 (직접 결과 표시 안 함, panel 으로 redirect).

사이드바 폭 부족은 결과 row 의 2-line layout (line 1: workspace + path, line 2: preview, padding) 으로 해결.

---

## 7. gitignore 처리

`ignore` crate 의 `WalkBuilder` 가 자동 처리:
- `.gitignore` (모든 parent + 현재 dir + global git config core.excludesfile)
- `.ignore` (ripgrep 의 자체 ignore)
- hidden files (`.git/`, `.DS_Store`)
- `WalkBuilder::standard_filters(true)` 가 위 3 가지 모두 활성

추가 hardcoded exclude (SPEC default):
- `target/` (Rust)
- `node_modules/` (JS/TS)
- `dist/`, `build/` (general)
- `__pycache__/`, `.venv/` (Python)
- `.moai/state/`, `.moai/cache/` (MoAI 자체)

이들은 `WalkBuilder::add_custom_ignore_filename` 로 합성, 또는 `OverrideBuilder` 로 명시적 add.

binary file 제외:
- `WalkBuilder::types(types_builder.add_defaults())` + 기본 text type 만 포함, 또는
- 검색 read 시 첫 8KB 에 NUL byte 발견 시 binary 로 판단 + skip (ripgrep 알고리즘).

---

## 8. 위험 평가

### 8.1 Spike — `ignore` crate 의 WalkBuilder 빌드 검증

`ignore = "0.4"` workspace 추가 후 `cargo build -p moai-studio-ui` 통과 검증. 의존성 conflict 가능성 낮음 — `ignore` 는 BurntSushi ecosystem (regex / globset 등) 와 동일 maintainer.

### 8.2 Spike — gitignore 매칭 정확도

unit test: tempdir 에 `.gitignore` (`target/\nnode_modules/\n*.log`) + 파일 (`src/a.rs`, `target/b.rs`, `app.log`, `node_modules/c.js`) → 검색 query "use" → src/a.rs 만 매칭 검증.

### 8.3 위험 — 큰 monorepo 첫 검색 지연

5k 파일 monorepo (e.g. moai-adk-go) 에서 첫 검색이 100ms 초과 시 사용자 가시 지연. mitigations:
- streaming 결과 (첫 hit 부터 즉시 표시)
- progress indicator (검색 진행 중 spinner + 검색 중인 workspace 이름)
- per-workspace progress bar

### 8.4 위험 — 다중 workspace 동시 walk → CPU 폭주

3+ workspace 동시 검색 시 ignore::WalkBuilder 내부 thread pool × N workspace = 많은 thread. mitigations:
- worker per workspace 가 아니라 **sequential per workspace** (walker 가 자체 multi-thread 이므로 단일 worker 만으로 CPU 활용 충분)
- 또는 `WalkBuilder::threads(2)` 로 worker 당 thread 수 제한

### 8.5 위험 — 결과 click → workspace 전환 시 panes/tabs 잃어버림

D-3 (state preserve on project switch) 가 PARTIAL 상태. workspace 전환 시 panes/tabs 가 자동 복원 가정. v0.1.2 PR #64 가 round-trip 강화 → 본 SPEC 은 그것을 신뢰. 만약 잃어버림 사례 발견 시 본 SPEC 의 책임 외 (D-3 별 SPEC).

### 8.6 위험 — preview 길이 / encoding

매 hit 의 preview line 이 매우 길 수 있음 (e.g. minified JS) → preview 는 max 200 chars + ellipsis. binary 로 판단되어도 첫 line 만 fallback preview.

---

## 9. 의존성 추가 결정

| Crate | 버전 | 용도 | 추가 위치 |
|-------|------|------|----------|
| `ignore` | `"0.4"` | gitignore 처리 + WalkBuilder | `crates/moai-studio-ui/Cargo.toml` (또는 별 `moai-search` crate) |
| `regex` | `"1"` | search query → Regex compile (case sensitivity option, word boundary option) | 동 |
| `walkdir` | `"2"` | (이미 plugin-installer 에 있음 — `ignore` 가 내부 사용 가능성) | indirect |

신규 crate 분리 검토:
- (a) `crates/moai-search/` 신규 crate 생성 — search engine 분리, future tantivy 마이그레이션 용이.
- (b) `moai-studio-ui::search/` 모듈로 통합 — UI 와 결합 단순.

권장: **(a) 신규 crate `crates/moai-search/`** — search engine 은 GPUI 의존 없이 logic-only 가능. SPEC-V3-005 의 Render Entity 패턴 (logic ↔ render 분리) 과 동일.

`crates/moai-search/` 구조:
- `src/lib.rs` — public API (`SearchSession`, `SearchHit`, `SearchOptions`)
- `src/walker.rs` — ignore::WalkBuilder wrapper
- `src/matcher.rs` — regex compile + line search
- `src/cancel.rs` — CancelToken
- `src/error.rs` — SearchError

`moai-studio-ui::search/` 모듈:
- `src/search/mod.rs` — module entry
- `src/search/panel.rs` — SearchPanel GPUI Entity
- `src/search/result_view.rs` — SearchResult row rendering
- `src/search/registry.rs` — SearchSession registry (active sessions)

---

## 10. milestone 분할 후보 (plan.md 와 동기화)

3 milestone 권장:

- **MS-1 — search engine + worker** — `crates/moai-search/` 신규 crate, WalkBuilder + regex matcher + cancel token + 단위 테스트. UI 0.
- **MS-2 — SearchPanel UI** — `moai-studio-ui::search/panel.rs` GPUI Entity, sidebar section toggle, input field + result list + cancel button. logic 만, navigation 0.
- **MS-3 — navigation + Command Palette entry** — 결과 click → workspace activate + new tab + line jump. Command Palette `workspace.search` entry 동작 wire (already in registry). ⌘⇧F 단축키 dispatch.
- **MS-4 — polish + a11y + 결과 cap + progress** (시간 허용 시) — backpressure / progress indicator / keyboard navigation (↑↓ result list).

---

## 11. 변경 금지 zone (R 제약)

- R1. `crates/moai-studio-terminal/**` 무변경 (SPEC-V3-002 carry).
- R2. `crates/moai-studio-ui/src/explorer/**` 무변경 — 본 SPEC 은 별 search 모듈 (`moai-studio-ui::search/`) 도입. explorer/search.rs 는 단일 workspace 트리 fuzzy filter 용이며 본 SPEC 와 별개.
- R3. `crates/moai-studio-ui/src/lib.rs` `RootView` 새 필드 추가만 허용 (`search_panel: Option<Entity<SearchPanel>>`).
- R4. `crates/moai-studio-workspace/src/lib.rs` `WorkspacesStore` 시그니처 변경 금지. `list()` read-only 사용.
- R5. `crates/moai-studio-ui/src/palette/registry.rs` `workspace.search` entry 이미 존재 — handler dispatch 만 wire, registry 변경 0.
- R6. `crates/moai-studio-ui/src/tabs/container.rs` `TabContainer::new_tab` 공개 API read-only 사용.

---

## 12. 미해결 결정 포인트 (annotation cycle 에서 확정)

다음 항목은 plan.md 작성 시 USER-DECISION 게이트로 분리 또는 기본 가정으로 확정:

1. **검색 엔진 선택 확정**: research 권장 = Option D (ignore + regex). 사용자 동의 필요 — 만약 사용자가 ripgrep crate 를 선호 시 MS-1 scope 변경.
2. **신규 crate `moai-search` vs ui 모듈 통합**: 권장 = 신규 crate. 사용자 결정.
3. **인덱싱 전략 v1**: 권장 = lazy. eager 옵션 v0.3.0+ deferred.
4. **결과 cap**: 권장 per-workspace 200 / per-file 50 / total 1000. 조정 가능.
5. **case sensitivity / regex / word boundary toggles**: v1 에 포함 vs v0.2.1 carry. 권장 = v1 default case-insensitive + 추후 toggle.
6. **glob include 패턴 (e.g. `*.rs` 만 검색)**: v1 vs v0.2.1. 권장 = v0.2.1 carry (단순화).

---

## 13. 영문 보조 요약 (Executive Summary)

SPEC-V0-2-0-GLOBAL-SEARCH-001 introduces multi-workspace global file content search for moai-studio v0.2.0, addressing audit Top 1 priority D-4. The recommended engine is **pure Rust** (`ignore` + `regex` crates), avoiding external binary dependency while matching ripgrep's gitignore accuracy. Indexing strategy is **lazy** (no background index in v1) — `ignore::WalkBuilder::build_parallel` provides sufficient throughput for typical monorepos. Each active workspace gets one worker (sequential per workspace, internal multi-thread walker) with `Arc<AtomicBool>` cancel token. Results stream via `mpsc::channel` to a new GPUI **SearchPanel** mounted in the sidebar (VS Code-style, toggleable via Cmd+Shift+F or Command Palette `workspace.search` entry — already registered in `palette/registry.rs:154`). Result click resolves via existing patterns: `WorkspacesStore::touch` to activate workspace, `TabContainer::new_tab` to open file, scroll to line/col. A new `crates/moai-search/` crate isolates the search engine (logic-only, GPUI-free, future tantivy-migration-ready). Three milestones: engine, UI, navigation. Excluded from v1: tantivy index, eager indexing, regex/case-sensitivity toggles, glob include patterns, drag-and-drop result.

---

작성 완료: 2026-05-01 sess 8
다음 산출: spec.md (EARS requirements + 9+ acceptance criteria + milestone × USER-DECISION 매핑).
