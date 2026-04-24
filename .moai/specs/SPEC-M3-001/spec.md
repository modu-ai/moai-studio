# SPEC-M3-001: M3 Code Viewer — SwiftTreeSitter + LSP 진단 + @MX 거터 + Tri-pane Diff + Time-travel (ARCHIVED — v2 Swift design)

> **⚠️ SUPERSEDED (2026-04-24)**: 본 SPEC 은 Swift/AppKit + SwiftTreeSitter 기반 v2 아키텍처를 전제한다. 2026-04-21 v3 pivot (GPUI + Rust) 으로 모든 참조 경로 (`app/Sources/Surfaces/Code/`, `core/crates/moai-lsp-bridge/`, `core/crates/moai-mx/`) 가 `archive/swift-legacy/` 로 이관되었으며, Code Viewer 기능 자체는 v3 아키텍처에서 **TBD — 후속 SPEC (SPEC-V3-CODE-VIEWER 등) 로 재발행** 예정.
>
> **주요 미정 결정**: (i) syntax highlighting 라이브러리 (tree-sitter-rust vs Zed 's gpui-syntax), (ii) LSP 클라이언트 경로 (lsp-types crate vs tower-lsp), (iii) tri-pane diff 구조 (diff-match-patch-rs 등), (iv) time-travel (git 통합 수준).
>
> **후속 조치**: (b) `status: archived-v2-design` 로 동결 채택. v3 기반 재설계 SPEC 은 SPEC-V3-003 MS-3 완료 이후 시점에 착수.

---
id: SPEC-M3-001
version: 1.1.0-archived
status: archived-v2-design
created_at: 2026-04-16
updated_at: 2026-04-24
superseded_by: TBD (v3 기반 Code Viewer SPEC 은 SPEC-V3-003 MS-3 완료 후 재발행)
author: MoAI (manager-spec)
priority: High
issue_number: 0
labels: [archived, v2-swift, code-viewer, lsp, superseded]
revision: v1.1.0-archived (plan-auditor 2026-04-24 감사 FAILED — v3 pivot 으로 archive 처리)
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.0.0 | 2026-04-16 | 초안 작성. M2 Conditional GO v1.2.0 (339 tests) 기반. DESIGN.v4 §5.2 / §7.1 / §11.1 / 로드맵 M3 (3주) 기준. P-1 (SPEC-M2-002 MS-2) 완료를 선행 조건으로 명시. |
| 1.1.0-archived | 2026-04-24 | plan-auditor 감사 FAILED — C-001 (archive 경로 drift 전면: SwiftTreeSitter, `core/crates/moai-lsp-bridge/`, `core/crates/moai-mx/` 모두 archive). v3 pivot 으로 v2 design 전제 모두 무효. status `draft → archived-v2-design`. v3 기반 재발행은 SPEC-V3-003 MS-3 완료 이후. |

---

## 1. 개요

MoAI Studio 의 "Code Viewer" 마일스톤. M2 에서 달성한 5개 Viewer (Terminal/FileTree/Markdown/Image/Browser) + Pane/Tab/Command Palette 구조 위에 **개발자가 실제로 코드를 편집하고 검토할 수 있는 Code Viewer surface** 를 추가한다. M3 를 통해 MoAI Studio 는 **읽기 전용 뷰어에서 개발 환경으로 승격**한다.

**성공 기준**: 개발자가 단일 MoAI Studio 세션에서 다음을 끊김 없이 수행할 수 있다.

1. FileTree 에서 `.go` / `.rs` / `.py` / `.ts` / `.c` / `.swift` 파일을 더블클릭 -> Code Viewer 에 열림 (Syntax highlighting 적용, <100ms)
2. 해당 파일의 LSP 진단 (error/warning/info) 이 거터와 줄 마커로 표시되고, hover tooltip 에 진단 메시지 노출
3. `@MX:NOTE`, `@MX:WARN`, `@MX:ANCHOR`, `@MX:TODO` 태그가 거터 아이콘으로 표시되고 클릭 시 popover (REASON 링크 + fan_in 카운트) 노출
4. Tri-pane diff 모드로 전환 -> 좌(HEAD git) / 중(working tree) / 우(pending) 3 열이 동기 스크롤되며 변경 블록 네비게이션 가능
5. Time-travel 슬라이더로 git log 를 스크럽 -> 해당 시점의 파일 내용이 Code Viewer 에 표시됨

**선행 조건**:

- [HARD] **P-1 (SPEC-M2-002 MS-2 완료 필수)**: TerminalSurface 가 실제 GhosttyHost 와 연결되어야 Code Viewer 와 Terminal 이 같은 pane tree 에서 공존 가능하다. P-1 미완료 시 M3 MS-6 (Surface 통합) 진행 불가.
- (권장) SPEC-M2-003 완료: Surface `state_json` 스키마 안정화 (파일 경로/뷰 모드/scroll offset 영속화 포맷 고정).
- SPEC-M2-001 MS-7 완료 (M2 Conditional GO v1.2.0, 339 tests PASS).

**참조 문서**:

- `DESIGN.v4.md` §5.2 Code Viewer Surface (렌더러, LSP 진단, @MX 거터, tri-pane diff, time-travel)
- `DESIGN.v4.md` §7.1 Swift Shell 스택 (`SwiftTreeSitter`)
- `DESIGN.v4.md` §11.1 성능 목표 (Tree-sitter incremental parse 1MB < 100ms)
- `DESIGN.v4.md` §8 로드맵 M3 (3주)
- `SPEC-M2-001/spec.md` (M2 산출물 기준)
- `.moai/project/product.md` §4-3 (Code Viewer 핵심 기능 정의)
- `.moai/project/tech.md` §2 (SwiftTreeSitter M3 상태)

---

## 2. 요구사항 그룹 (EARS 형식)

### RG-M3-1: Syntax Highlighting (SwiftTreeSitter)

**[Ubiquitous]** Swift 측 `app/Sources/Surfaces/Code/` 디렉토리에 `NSTextView` 를 서브클래싱한 `CodeEditorView` 를 **구현해야 한다** (shall implement). SwiftUI `NSViewRepresentable` wrapper 로 노출한다.

**[Ubiquitous]** Code Viewer 는 **SwiftTreeSitter** (tree-sitter 바인딩) 를 사용하여 구문 분석을 **수행해야 한다** (shall perform). 자체 파서 구현은 금지한다.

**[Ubiquitous]** 시스템은 다음 6개 언어의 tree-sitter grammar 를 번들에 **포함해야 한다** (shall include): `tree-sitter-go`, `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-typescript` (+ `tsx`), `tree-sitter-c`, `tree-sitter-swift`.

**[Event-Driven]** 파일이 Code Viewer 에서 열리면 (When), 시스템은 파일 확장자 기반으로 grammar 를 선택하여 초기 파싱을 **수행해야 한다** (shall perform). 확장자 매핑: `.go` -> go, `.rs` -> rust, `.py` -> python, `.ts|.tsx|.js|.jsx` -> typescript, `.c|.h|.cpp|.hpp` -> c, `.swift` -> swift.

**[Event-Driven]** 사용자가 편집하여 버퍼가 변경되면 (When), 시스템은 변경 영역에 대해서만 **incremental reparsing 을 수행해야 한다** (shall perform incremental reparsing). 전체 reparse 는 금지한다 (fallback 제외).

**[Ubiquitous]** 시스템은 다음 테마 토큰을 **렌더링해야 한다** (shall render): `keyword`, `type`, `string`, `comment`, `function`, `variable`, `number`, `operator`, `punctuation`. 다크/라이트 테마 쌍은 macOS 시스템 설정에 연동한다.

**[If]** SwiftTreeSitter incremental reparsing 이 실패하면 (If failure detected), 시스템은 **전체 full reparse 로 fallback 해야 한다** (shall fall back to full reparse). 실패 원인은 `tracing` 로그에 기록한다.

**[Optional]** 심볼 인덱스 (Go to Definition 지원용) 를 **제공할 수 있다** (may provide). M3 범위 내에서는 동일 파일 내 jump 만 지원.

**산출물**: `app/Sources/Surfaces/Code/CodeEditorView.swift`, `app/Sources/Surfaces/Code/TreeSitterHighlighter.swift`, grammar 6종 번들 (SPM dependency 또는 xcframework).

---

### RG-M3-2: LSP 진단 (6개 언어)

**[Ubiquitous]** 시스템은 6개 언어 각각의 공식 LSP 서버와 통신할 수 있어야 한다 (shall be able to communicate): `gopls` (go), `rust-analyzer` (rust), `pyright` (python), `typescript-language-server` (typescript/javascript), `clangd` (c/cpp), `sourcekit-lsp` (swift).

**[Ubiquitous]** 진단 수집 경로는 **`mcp__ide__getDiagnostics` tool 호출** 을 PRIMARY 로 **한다** (shall use as primary). Claude Code 가 plugin `.lsp.json` 으로 spawn 한 LSP 서버 경유. 자체 LSP 클라이언트 구현은 금지한다 (tech.md §7 근거).

**[Ubiquitous]** 진단 수집 경로 SECONDARY 는 **moai-lsp MCP bridge** 로 **한다** (shall use as secondary). Claude subprocess 가 비활성 상태일 때 Rust core 가 직접 LSP 서버와 통신하는 보조 경로.

**[Event-Driven]** Code Viewer 에서 파일이 활성화되면 (When), 시스템은 해당 파일의 진단을 PRIMARY 경로 우선으로 **요청해야 한다** (shall request). PRIMARY 실패 시 SECONDARY 로 자동 전환한다.

**[Ubiquitous]** 시스템은 각 진단을 거터에 아이콘으로 **표시해야 한다** (shall display in gutter): error -> 빨강 원, warning -> 노랑 삼각형, info -> 파랑 정보 아이콘, hint -> 회색 점.

**[Ubiquitous]** 시스템은 해당 줄에 squiggly underline 을 **렌더링해야 한다** (shall render): error -> 빨강, warning -> 노랑, info -> 파랑.

**[Event-Driven]** 사용자가 진단 아이콘 또는 squiggly underline 에 hover 하면 (When), 시스템은 tooltip 으로 진단 메시지, severity, source (LSP 서버명), code 를 **표시해야 한다** (shall show).

**[State-Driven]** 파일 내용이 편집되는 동안 (While editing), 시스템은 LSP 진단을 debounce (500ms) 후 **재요청해야 한다** (shall re-request). 타이핑 중 과도한 요청을 방지한다.

**[Ubiquitous]** 시스템은 `.moai/config/sections/quality.yaml` 임계값과 실시간 에러 카운트를 비교하여 **LSP gate overlay 를 표시해야 한다** (shall display). 임계값 초과 시 에디터 상단에 붉은 배너로 "LSP errors exceed threshold" 노출 (DESIGN.v4 §5.2).

**[If]** LSP 서버가 설치되지 않았거나 spawn 실패 시 (If LSP unavailable), 시스템은 해당 언어에 대해 **진단 기능을 비활성화하고** (shall disable) 상태 바에 "LSP: {language} unavailable" 를 **표시해야 한다** (shall indicate).

**산출물**: `app/Sources/Surfaces/Code/DiagnosticsOverlay.swift`, `core/crates/moai-lsp-bridge/` (SECONDARY 경로), FFI `fetch_diagnostics(file_path)`.

---

### RG-M3-3: @MX 거터 (gutter annotations)

**[Ubiquitous]** Rust core 는 `@MX:NOTE`, `@MX:WARN`, `@MX:ANCHOR`, `@MX:TODO` 4종 태그를 파일에서 스캔하여 `mx_tags` 테이블에 **캐시해야 한다** (shall cache). 스키마: `id INTEGER PK, path TEXT, line INTEGER, kind TEXT, body TEXT, reason TEXT NULL, fan_in INTEGER DEFAULT 0, spec_id TEXT NULL, updated_at INTEGER`.

**[Ubiquitous]** Rust core 는 `/moai mx --dry --json` 결과 또는 등가 내부 API 로 태그 스캔을 **수행해야 한다** (shall perform). 파일 저장 시 그리고 파일 로드 시 양쪽에서 갱신한다.

**[Event-Driven]** Code Viewer 에서 파일이 활성화되면 (When), 시스템은 `mx_tags` 테이블에서 해당 파일의 태그를 조회하여 거터에 아이콘으로 **표시해야 한다** (shall render). 아이콘 매핑: `@MX:ANCHOR` -> ★ (금색), `@MX:WARN` -> ⚠ (주황), `@MX:NOTE` -> ℹ (파랑), `@MX:TODO` -> ☐ (회색).

**[Event-Driven]** 사용자가 거터 아이콘을 클릭하면 (When), 시스템은 popover 를 **표시해야 한다** (shall show). popover 는 다음을 포함한다:
- 태그 kind 와 body
- `@MX:WARN` 의 경우 `REASON` 링크 (없으면 "REASON required" 경고)
- `@MX:ANCHOR` 의 경우 fan_in 카운트 (예: "Called by 7 functions")
- `spec_id` 가 있으면 "Jump to SPEC-{ID}" 링크 (클릭 시 Markdown surface 에 해당 SPEC 오픈)

**[State-Driven]** 파일 시스템에 변경이 발생하는 동안 (While fs events), 시스템은 moai-fs (notify) 이벤트를 구독하여 `mx_tags` 캐시를 **갱신하고 UI 를 재렌더해야 한다** (shall refresh).

**[Ubiquitous]** fan_in 계산은 symbol call graph 기반으로 **수행해야 한다** (shall compute). M3 범위에서는 정적 분석이 어려운 경우 "N/A" 표기를 허용한다 (M4 이후 정확도 개선).

**[Optional]** `@MX:WARN` 에 `REASON` 이 누락된 경우 거터 아이콘을 **깜빡이게 표시할 수 있다** (may blink). 시각적 어포던스.

**산출물**: `core/crates/moai-store/migrations/v4__mx_tags.sql` (또는 기존 v3 확장), `core/crates/moai-mx/` (태그 스캐너), FFI `fetch_mx_tags(file_path)`, `app/Sources/Surfaces/Code/MxGutter.swift`.

---

### RG-M3-4: Tri-pane Diff (HEAD | working | pending)

**[Ubiquitous]** Code Viewer 는 "Tri-pane diff" 모드를 **지원해야 한다** (shall support). 모드 활성화 시 상단 툴바에서 "Diff" 토글 버튼으로 전환한다.

**[Ubiquitous]** Tri-pane 레이아웃은 3 열로 **구성해야 한다** (shall compose):
- 좌측 열: HEAD (git2 로 읽은 `HEAD:<path>` 스냅샷)
- 중앙 열: working tree (현재 디스크 상의 파일)
- 우측 열: pending (`/moai run` 결과물 혹은 unstaged 변경. M3 범위에서는 unstaged 변경으로 한정)

**[Ubiquitous]** 3 열은 **줄 단위 동기 스크롤을 지원해야 한다** (shall support synchronized scrolling). 한 열에서 스크롤 시 나머지 2 열이 대응 줄에 맞춰 스크롤된다.

**[Ubiquitous]** 시스템은 **공통 라인 하이라이트** 를 **수행해야 한다** (shall perform): 3 열 공통 줄은 일반 배경, 변경 블록은 색상 (added=초록, removed=빨강, modified=파랑) 으로 구분한다. Diff 알고리즘은 Myers diff (git2 내장) 를 사용한다.

**[Event-Driven]** 사용자가 변경 블록 네비게이션 단축키 (F7 / Shift+F7) 를 누르면 (When), 시스템은 다음/이전 변경 블록으로 커서를 **이동해야 한다** (shall jump).

**[Event-Driven]** 사용자가 Accept 또는 Revert 버튼을 클릭하면 (When), 시스템은 해당 변경 블록을 working tree 에 반영하거나 HEAD 로 되돌려야 한다 (shall apply or revert) (DESIGN.v4 §5.2). M3 범위: Accept/Revert 는 **read-only 모드에서는 비활성화**, edit 모드 진입 후에만 활성화.

**[Ubiquitous]** Edit mode 진입 시 시스템은 **자동 git stash 스냅샷을 생성해야 한다** (shall create auto-stash). 변경 취소 시 복구 가능.

**[State-Driven]** pending 열이 비어있는 동안 (While no pending), 시스템은 해당 열에 "No pending changes" placeholder 를 **표시해야 한다** (shall show).

**산출물**: `app/Sources/Surfaces/Code/TriPaneDiffView.swift`, `core/crates/moai-git/` diff API 확장, FFI `compute_tri_diff(path)`.

---

### RG-M3-5: Time-travel 리뷰

**[Ubiquitous]** Code Viewer 는 "Time-travel" 모드를 **지원해야 한다** (shall support). 모드 활성화 시 상단에 git log 기반 커밋 히스토리 슬라이더가 노출된다.

**[Ubiquitous]** 슬라이더는 현재 파일을 touch 한 커밋 히스토리를 **표시해야 한다** (shall display). 각 슬라이더 스텝은 하나의 커밋 (short SHA + subject + author + relative time) 에 대응한다.

**[Event-Driven]** 사용자가 슬라이더를 이동하면 (When), 시스템은 해당 시점 커밋의 파일 내용을 읽어 Code Viewer 에 **표시해야 한다** (shall show). git2 `find_blob` 을 사용한다.

**[Event-Driven]** 사용자가 슬라이더를 이동하면 (When), 시스템은 해당 시점에 존재한 파일 트리 스냅샷도 사이드바에 **갱신해야 한다** (shall refresh). 사이드바는 FileTree surface 와 공유한다.

**[Ubiquitous]** Time-travel 모드에서는 편집이 **비활성화되어야 한다** (shall disable editing). read-only 모드로 강제된다.

**[State-Driven]** Time-travel 모드가 활성화된 동안 (While active), 시스템은 에디터 상단에 "Time-travel: {short_sha} ({author}, {date})" 배너를 **표시해야 한다** (shall show).

**[Ubiquitous]** 슬라이더 최우측은 working tree (현재 상태) 에 **해당해야 한다** (shall correspond). 슬라이더를 최우측으로 이동하면 Time-travel 모드가 자동 종료된다.

**[Optional]** 각 시점의 task-metric bar chart (`.moai/logs/task-metrics.jsonl` 또는 `task_metrics_mirror` 테이블) 를 슬라이더 하단에 **표시할 수 있다** (may display) (DESIGN.v4 §5.2).

**[Event-Driven]** 사용자가 `/moai run` hook event stream 연동을 요청하면 (When), 시스템은 **M5 범위임을 안내해야 한다** (shall notify). M3 범위에서는 git 기반 time-travel 로 **한정한다** (shall limit). Agent run stream 과의 통합은 **M5 Agent Run Viewer 에서 처리한다** (M5 boundary).

**산출물**: `app/Sources/Surfaces/Code/TimeTravelSlider.swift`, `core/crates/moai-git/` `list_commits_for_path()`, FFI `fetch_blob_at_commit(path, sha)`.

---

### RG-M3-6: Code Viewer Surface 통합 (Pane tree, Tab UI 호환)

**[Ubiquitous]** Code Viewer 는 M2 의 `SurfaceProtocol` 을 **구현해야 한다** (shall conform). `surfaceKind = .code`.

**[Ubiquitous]** Code Viewer 는 **Pane tree 및 Tab UI 와 호환되어야 한다** (shall be compatible). 즉, NSSplitView leaf 의 tab bar 에 code surface 인스턴스가 탭으로 추가되며, 다른 surface 와 동일한 lifecycle (init, activate, deactivate, destroy) 을 따른다.

**[Ubiquitous]** Code Viewer 의 상태 (현재 파일 경로, scroll offset, cursor position, view mode (normal/diff/time-travel), diff 시점 SHA) 는 `surfaces.state_json` BLOB 에 **영속해야 한다** (shall persist). 앱 재시작 시 정확히 복원되어야 한다.

**[Event-Driven]** FileTree surface 에서 사용자가 지원 확장자 파일을 더블클릭하면 (When), 시스템은 활성 pane 에 새 Code Viewer 탭을 **생성하고 파일을 열어야 한다** (shall open). RG-M2-4 의 "파일 확장자에 따라 적절한 surface" 규칙을 확장한다.

**[Event-Driven]** Command Palette 에서 "Open in Code Viewer" 명령이 선택되면 (When), 시스템은 활성 pane 에 Code Viewer 탭을 **생성해야 한다** (shall create). RG-M2-3 확장.

**[Ubiquitous]** Code Viewer 는 toolbar 에 다음 아이템을 **포함해야 한다** (shall include): 파일 경로 breadcrumb, View mode 토글 (Normal/Diff/Time-travel), LSP gate 배너 슬롯, Edit mode 토글 (read-only 기본), Reveal in FileTree 버튼.

**[State-Driven]** Code Viewer 가 비활성 (탭 비활성화) 상태인 동안 (While deactivated), 시스템은 tree-sitter 파서 스레드를 **일시정지해야 한다** (shall pause). 메모리 누수 방지. 활성화 시 재개한다.

**산출물**: `SurfaceProtocol` 준수, `app/Sources/Surfaces/Code/CodeSurface.swift`, state_json 스키마 확장.

---

## 3. 수용 기준 (Acceptance Criteria)

### AC-M3-1.x (Syntax Highlighting)

- **AC-M3-1.1**: 6개 언어 각각의 샘플 파일 (1000줄) 을 열 때 syntax highlighting 이 적용되고 렌더링 완료 시간 <100ms.
- **AC-M3-1.2**: 편집 후 incremental reparse 완료 시간 <16ms (1000줄 파일, 10줄 변경 기준).
- **AC-M3-1.3**: 9종 테마 토큰 (keyword/type/string/comment/function/variable/number/operator/punctuation) 모두 다크/라이트 테마에서 시각적으로 구분된다.
- **AC-M3-1.4**: incremental reparse 실패를 강제 주입한 테스트에서 full reparse fallback 이 동작하고 `tracing::warn!` 로그가 출력된다.

### AC-M3-2.x (LSP 진단)

- **AC-M3-2.1**: PRIMARY 경로 (`mcp__ide__getDiagnostics`) 가 성공하는 환경에서 6개 언어 각각에 대해 error/warning/info/hint 4종 진단이 거터와 줄 마커로 표시된다.
- **AC-M3-2.2**: PRIMARY 실패 시 SECONDARY (moai-lsp bridge) 로 자동 전환되고, 사용자에게는 변경 없이 진단이 노출된다.
- **AC-M3-2.3**: 편집 중 500ms debounce 후 LSP 재요청이 수행되고, 연속 타이핑 시에는 추가 요청이 억제된다.
- **AC-M3-2.4**: LSP gate overlay 가 quality.yaml 임계값 초과 시에만 배너로 노출된다.
- **AC-M3-2.5**: LSP 서버 미설치 상황에서 앱이 크래시하지 않고 "LSP: {language} unavailable" 상태 바에 표시된다.

### AC-M3-3.x (@MX 거터)

- **AC-M3-3.1**: 4종 태그 (ANCHOR/WARN/NOTE/TODO) 각각의 거터 아이콘이 올바른 색상과 모양으로 표시된다.
- **AC-M3-3.2**: `@MX:WARN` 에 `REASON` 이 없는 경우 popover 에 "REASON required" 경고가 표시된다.
- **AC-M3-3.3**: `@MX:ANCHOR` popover 에 fan_in 카운트가 표시된다 (정적 분석 가능 시 정확한 값, 불가 시 "N/A").
- **AC-M3-3.4**: `@MX:ANCHOR SPEC-XXX-NNN` 태그의 popover 에서 "Jump to SPEC" 링크 클릭 시 Markdown surface 가 해당 SPEC 파일을 연다.
- **AC-M3-3.5**: 파일 편집 후 저장 시 `mx_tags` 캐시가 갱신되고 거터 아이콘이 반영된다.

### AC-M3-4.x (Tri-pane Diff)

- **AC-M3-4.1**: Diff 모드 토글 시 3 열 레이아웃이 표시되고 좌=HEAD / 중=working / 우=pending 배치가 맞다.
- **AC-M3-4.2**: 한 열에서 스크롤 시 나머지 2 열이 대응 줄에 동기화되어 스크롤된다.
- **AC-M3-4.3**: added/removed/modified 블록이 초록/빨강/파랑으로 시각적으로 구분된다.
- **AC-M3-4.4**: F7 / Shift+F7 단축키로 다음/이전 변경 블록으로 이동한다.
- **AC-M3-4.5**: pending 열이 비어있는 파일에서는 "No pending changes" placeholder 가 표시된다.
- **AC-M3-4.6**: Edit mode 진입 시 git stash 스냅샷이 자동 생성되고 `git stash list` 에서 확인 가능하다.

### AC-M3-5.x (Time-travel)

- **AC-M3-5.1**: Time-travel 슬라이더가 현재 파일을 touch 한 커밋만 스텝으로 노출한다.
- **AC-M3-5.2**: 슬라이더 이동 시 해당 시점 파일 내용이 Code Viewer 에 표시되며 편집은 비활성화된다.
- **AC-M3-5.3**: 에디터 상단 배너에 "Time-travel: {short_sha} ({author}, {date})" 가 정확히 표시된다.
- **AC-M3-5.4**: 슬라이더 최우측 이동 시 Time-travel 모드가 자동 종료되고 working tree 상태로 복귀한다.
- **AC-M3-5.5**: Agent run stream 연동 요청 시 "M5 범위" 안내 메시지가 노출된다 (M5 boundary 유지).

### AC-M3-6.x (Surface 통합)

- **AC-M3-6.1**: Code Viewer 가 `SurfaceProtocol` 을 완전히 구현하여 다른 surface 와 동일한 lifecycle 로 작동한다.
- **AC-M3-6.2**: FileTree 에서 `.rs` 파일 더블클릭 -> 활성 pane 에 Code Viewer 탭 생성 -> 파일 표시까지 <500ms.
- **AC-M3-6.3**: 앱 종료 후 재시작 시 Code Viewer 의 파일 경로, scroll offset, cursor position, view mode, diff 시점 SHA 가 정확히 복원된다.
- **AC-M3-6.4**: Code Viewer 탭 비활성 시 tree-sitter 파서 스레드가 일시정지되고 (Instruments 로 확인), 활성화 시 즉시 재개된다.

### AC-M3-7.x (E2E 통합 시나리오 — RG-M3-1~6 통합)

- **AC-M3-7.1**: 개발자 시나리오 E2E — FileTree 에서 `.go` 파일 더블클릭 -> Code Viewer 열림 (syntax highlight) -> LSP 진단 표시 -> @MX 태그 거터 확인 -> Diff 모드 전환 -> 변경 블록 네비게이션 -> Time-travel 로 3 커밋 전으로 이동 -> 복귀 -> 모든 단계 <3s.
- **AC-M3-7.2**: 8 pane × 8 tab 환경에서 Code Viewer 가 4개 이상 열려 있을 때 RSS <700MB 유지.

---

## 4. 마일스톤

우선순위 기반. 시간 추정은 사용하지 않는다.

### MS-1 (High): SwiftTreeSitter 통합 + 6개 언어 grammar

- SwiftTreeSitter SPM dependency 또는 xcframework 통합
- 6개 언어 grammar 번들 포함 (go, rust, python, typescript, c, swift)
- `TreeSitterHighlighter` 구현 + 9종 토큰 테마
- Incremental reparse 구현 + full reparse fallback
- 단위 테스트: grammar 선택, 토큰 매핑, incremental 정확성
- 성능 벤치마크: 10,000 줄 초기 렌더 <100ms, incremental <16ms

### MS-2 (High): LSP 프로토콜 brokering

- PRIMARY 경로: `mcp__ide__getDiagnostics` 호출 래퍼 구현
- SECONDARY 경로: `core/crates/moai-lsp-bridge/` 최소 LSP 클라이언트 (initialize, textDocument/publishDiagnostics 구독)
- PRIMARY -> SECONDARY 자동 전환 로직
- `DiagnosticsOverlay` UI (거터 아이콘 + squiggly underline + hover tooltip)
- LSP gate overlay (quality.yaml 임계값 비교)
- 미설치 LSP 서버 graceful degradation

### MS-3 (High): @MX 거터 렌더링

- `mx_tags` 테이블 마이그레이션 (V4 또는 V3 확장)
- `core/crates/moai-mx/` 스캐너 (`/moai mx --dry --json` 결과 또는 내부 API 재사용)
- FFI `fetch_mx_tags(file_path)` + `moai-fs` 이벤트 구독
- `MxGutter` UI (4종 아이콘 + popover + REASON/fan_in/spec_id)
- Markdown surface "Jump to SPEC" 링크 통합

### MS-4 (Medium): Tri-pane Diff 엔진

- git2 기반 HEAD blob 읽기, Myers diff 계산
- 3 열 `TriPaneDiffView` (동기 스크롤, 블록 하이라이트)
- 변경 블록 네비게이션 단축키 (F7 / Shift+F7)
- Edit mode 진입 시 git stash auto-snapshot
- pending 열 placeholder 처리

### MS-5 (Medium): Time-travel UX

- git2 기반 `list_commits_for_path()` + `find_blob(sha, path)`
- `TimeTravelSlider` UI (커밋 히스토리 슬라이더, 상단 배너)
- read-only 강제 + working tree 복귀 시 모드 자동 종료
- M5 경계 안내 메시지

### MS-6 (High): Code Viewer Surface 통합 + E2E

- `CodeSurface` 클래스 (`SurfaceProtocol` 준수)
- `surfaces.state_json` 스키마 확장 (파일 경로, scroll offset, cursor, view mode, diff SHA)
- FileTree 더블클릭 / Command Palette "Open in Code Viewer" 연동
- Toolbar (breadcrumb, view mode 토글, LSP gate 슬롯, Edit 토글, Reveal in FileTree)
- 비활성 시 파서 스레드 일시정지
- E2E 시나리오 테스트 (AC-M3-7.x)

---

## 5. 비기능 요구사항 (NFR)

| 항목 | 목표 | 측정 방법 |
|------|------|-----------|
| 10,000 줄 파일 초기 렌더 | <100ms | 파일 open -> 첫 번째 페인트 완료 (XCTest Performance) |
| Incremental reparse (10줄 변경) | <16ms | 편집 이벤트 -> 토큰 업데이트 완료 (Instruments time profiler) |
| LSP 진단 첫 수신 | <500ms | 파일 열기 -> 거터 아이콘 표시 (PRIMARY 경로) |
| Tri-pane diff 초기 계산 | <200ms (10,000 줄 파일) | Diff 토글 -> 3 열 렌더 완료 |
| Time-travel 스텝 전환 | <100ms | 슬라이더 이동 -> 파일 내용 교체 완료 |
| Code Viewer 탭 전환 | <50ms | 탭 클릭 -> 활성화 완료 (M2 NFR 승계) |
| 앱 재시작 후 Code Viewer 상태 복원 | 100% 일치 | 파일/scroll/cursor/view mode/SHA 동일 |
| LSP 서버당 RSS | <200MB | `ps -o rss` 샘플링 |
| 전체 RSS (8 pane × 8 tab, Code Viewer 4개 포함) | <700MB | Instruments |
| SwiftTreeSitter incremental 실패 시 fallback | 100% 복구 | 강제 주입 테스트 |
| VoiceOver 접근성 | 전체 UI 읽기 가능 | macOS Accessibility Inspector |
| 거터 구분 | 색상 + 아이콘 병용 | 색맹 시뮬레이션 (Sim Daltonism) |

---

## 6. 의존성 및 제약

### 외부 의존성

| 항목 | 제공 주체 | 비고 |
|------|----------|------|
| SwiftTreeSitter | SPM package | tree-sitter Swift binding |
| tree-sitter-go | git submodule or SPM | grammar |
| tree-sitter-rust | git submodule or SPM | grammar |
| tree-sitter-python | git submodule or SPM | grammar |
| tree-sitter-typescript | git submodule or SPM | grammar (+ tsx) |
| tree-sitter-c | git submodule or SPM | grammar |
| tree-sitter-swift | git submodule or SPM | grammar |
| gopls | 사용자 환경 PATH | LSP |
| rust-analyzer | 사용자 환경 PATH | LSP |
| pyright | 사용자 환경 PATH | LSP (pyright-langserver) |
| typescript-language-server | 사용자 환경 PATH | LSP |
| clangd | 사용자 환경 PATH | LSP |
| sourcekit-lsp | Xcode 번들 | LSP |

### 선행 SPEC (HARD)

- **P-1 (HARD)**: `SPEC-M2-002 MS-2` 완료 — TerminalSurface <-> GhosttyHost 연결. P-1 미완료 시 M3 MS-6 (Surface 통합) 블록.
- **권장**: `SPEC-M2-003` 완료 — `surfaces.state_json` 스키마 안정화.

### 설계 문서 준수

- DESIGN.v4 §5.2 Code Viewer Surface — 본 SPEC 의 원천 설계
- DESIGN.v4 §7.1 Swift Shell 스택 — SwiftTreeSitter 사용 확정
- DESIGN.v4 §11.1 성능 목표 — Tree-sitter incremental parse 1MB <100ms 승계
- DESIGN.v4 §8 로드맵 M3 (3주) — 본 SPEC 의 범위 경계

### 금지 사항

- 자체 LSP 클라이언트 구현 금지 (tech.md §7 근거, `mcp__ide__getDiagnostics` PRIMARY)
- 자체 syntax 파서 구현 금지 (SwiftTreeSitter 경유 필수)
- 전체 reparse 기본 사용 금지 (incremental 필수, 실패 시만 fallback)

---

## 7. 테스트 전략

| 레벨 | 도구 | 대상 | 대표 케이스 |
|------|------|------|-----------|
| Rust unit | `cargo test` | `moai-mx` 태그 스캐너, `moai-lsp-bridge` RPC 인코딩, `moai-git` diff/blob/commit_list | 6개 언어 샘플 파일, 태그 4종 파싱 |
| Rust integration | `cargo test --features mock-claude` | `mcp__ide__getDiagnostics` 왕복, SECONDARY LSP 동작 | mock LSP 서버 fixture |
| Swift unit | **Swift Testing** | `TreeSitterHighlighter` 토큰 매핑, `DiagnosticsOverlay` 상태, `MxGutter` popover | 9종 토큰 스냅샷 |
| Swift UI | **XCUITest** + swift-snapshot-testing | Code Viewer 렌더, Tri-pane diff 레이아웃, Time-travel 슬라이더 | 다크/라이트 테마 각 3 케이스 |
| 성능 | **XCTest Performance** | 10K 줄 초기 렌더, incremental reparse, diff 계산 | 벤치마크 baseline 기록 |
| E2E | 개발자 시나리오 스크립트 | AC-M3-7.1 (full flow), AC-M3-7.2 (RSS) | Instruments + ps 샘플링 |
| 접근성 | macOS Accessibility Inspector | VoiceOver 라벨, 대비비, 색맹 시뮬레이션 | 6개 surface 공통 체크리스트 |

### 커버리지 목표

- Rust: 85%+ (cargo tarpaulin 또는 llvm-cov)
- Swift: 85%+ (xcov)
- 6개 언어 각각 최소 1개의 샘플 파일 테스트 (grammar 정확성)
- LSP 6개 서버 각각 최소 1개의 진단 fixture

---

## 8. 참조 문서

- **DESIGN.v4.md** §5.2 Code Viewer Surface (원천 설계)
- **DESIGN.v4.md** §7.1 Swift Shell 스택 (SwiftTreeSitter)
- **DESIGN.v4.md** §11.1 성능 목표
- **DESIGN.v4.md** §8 로드맵 M3 (3주)
- **SPEC-M2-001/spec.md** — M2 산출물 (SurfaceProtocol, Pane tree, Tab UI, Command Palette, FileTree 더블클릭 규칙)
- **SPEC-M2-002** (존재 시) — TerminalSurface <-> GhosttyHost 연결 (P-1 선행)
- **.moai/project/product.md** §4-3 (Code Viewer 핵심 기능 정의), §7 (비기능 요구사항)
- **.moai/project/tech.md** §2 (SwiftTreeSitter M3 상태), §7 (LSP plugin feature), §11 (성능 목표)

---

## 9. Exclusions (What NOT to Build)

M3 범위에서 **명시적으로 제외** 되는 항목. 향후 마일스톤 경계 유지.

1. **Agent Run Viewer 통합 Time-travel** — M5 범위. M3 의 Time-travel 은 **git log 기반으로 한정**. `/moai run` hook event stream 과의 연동 (세션 단위 스크럽, cost bar) 은 **M5 Agent Run Viewer 에서 처리**.
2. **Kanban 보드** — M5 범위. SPEC <-> worktree <-> `/moai run` 자동 연동.
3. **Memory Viewer / InstructionsGraph** — M5 범위.
4. **Plugin 자동 설치 (`~/.claude/plugins/moai-studio@local/` drop) + `.lsp.json` 자동 wiring** — M4 범위. M3 범위에서는 **사용자가 수동으로 LSP 서버를 PATH 에 설치** 한다고 전제한다.
5. **Native Permission Dialog** — M4 범위.
6. **Go to Definition / Find References / Rename** (cross-file symbol navigation) — M4 이후. M3 의 Optional 심볼 인덱스는 **동일 파일 내 jump 로 한정**.
7. **Accept/Revert 의 pending 열 `/moai run` artifact 연동** — M5 범위. M3 의 pending 열은 **unstaged 변경으로 한정**.
8. **정확한 fan_in 계산 (call graph)** — M4 이후. M3 에서는 정적 분석 불가 시 "N/A" 허용.
9. **16+ 동시 Code Viewer 탭 스트레스** — M6 범위.
10. **Auto-update (Sparkle)** — M6 범위.
11. **터미널 내 커서 위치 -> Code Viewer 파일 자동 열기** — M4+ 범위.
12. **Code Viewer <-> Browser surface 통신** (예: 브라우저 DevTools 소스맵 연동) — M6+ 범위.

---

## 10. 용어 정의

| 용어 | 정의 |
|------|------|
| Code Viewer | `SurfaceProtocol` 구현체 중 하나. `surfaceKind = .code`. 파일 단위 편집/리뷰 UI |
| SwiftTreeSitter | tree-sitter 파서의 Swift binding. 6개 언어 grammar 의 incremental parse 를 제공 |
| Incremental reparse | 전체 파일이 아닌 **변경된 영역만** 재파싱하는 tree-sitter 기능. 성능 핵심 |
| LSP PRIMARY | `mcp__ide__getDiagnostics` tool 경유 진단 수집. 기본 경로 |
| LSP SECONDARY | `moai-lsp-bridge` 경유 진단 수집. Claude subprocess 비활성 시 대체 경로 |
| LSP gate overlay | quality.yaml 임계값 초과 시 에디터 상단에 표시되는 붉은 배너 |
| @MX 거터 | 파일 좌측 거터에 표시되는 `@MX:*` 태그 아이콘 영역 |
| fan_in | 해당 함수를 호출하는 다른 함수의 수. `@MX:ANCHOR` 대상 판단 기준 (>= 3) |
| Tri-pane diff | 좌(HEAD) / 중(working tree) / 우(pending) 3 열 diff 레이아웃 |
| pending | `/moai run` 결과물 또는 unstaged 변경. M3 범위에서는 unstaged 로 한정 |
| Time-travel | git log 기반 커밋 히스토리 슬라이더로 파일 내용을 과거 시점으로 전환하는 read-only 모드 |
| Edit mode | Code Viewer 의 read-only 기본 상태를 해제하여 편집을 허용하는 모드. 진입 시 git stash auto-snapshot |
