---
id: SPEC-V3-003
version: 1.1.0
status: run-in-progress-ms2-complete
created_at: 2026-04-24
updated_at: 2026-04-24
approved_at: 2026-04-24
author: MoAI (manager-spec, v1.0.0 annotation cycle approval)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, terminal, panes, tabs, persistence]
revision: v1.1.0 (plan-auditor 2026-04-24 감사 — MS-1/MS-2 완료, MS-3 대기; status 세분화 + §7.7 TabBar abstract 추가)
milestone_status:
  MS-1: completed (2026-04-24, 14 AC FULL 13 / PARTIAL 0 / DEFERRED 1 → AC-P-5 MS-3 재승계)
  MS-2: completed (2026-04-24, 10 primary + 2 carry-over, FULL 11 / DEFERRED 1 AC-P-5)
  MS-3: pending (T12 persistence + T13 E2E + T14 CI, contract.md §12)
---

# SPEC-V3-003: Tab / Pane Split — 다중 탭 + 이진 트리 pane split

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 0.1.0-draft | 2026-04-24 | research.md (625 LOC) 완료 후 사용자의 7개 scope 질문 확정 답변을 반영해 초안 작성. Q1 다중 탭 + 탭당 pane tree, Q2 깊이 무제한 + 최소 pane 크기, Q3 direct modifier 키 바인딩, Q4 open pane 구조만 JSON 복원, Q5 단일 SPEC + 3 milestones, Q6 추상 인터페이스만 정의 후 plan spike 에서 결정, Q7 MS-1 Pane core → MS-2 Tabs → MS-3 Persistence. gpui-component 의존 여부 및 divider drag API 확인은 plan 단계로 연기. |
| 0.2.0-draft | 2026-04-24 | plan-auditor iter 1 감사 FAIL 대응 iter 2 재작업. (C-1) §1.3 / §13.1 의 `system.md:422-439` 참조를 실존 경로 `.moai/design/v3/spec.md:420-438` 로 교정. (C-2) 사용자 확정 (2026-04-24): macOS Cmd ↔ Linux Ctrl 이원화를 RG-P-4 표 두 컬럼 구조로 반영, G3 / G7 / §6.4 / AC-P-9 에 양 플랫폼 검증 명시. (C-3) Horizontal = 좌/우 배치 (수직 divider), Vertical = 상/하 배치 (수평 divider) 를 §7.1 / §15 / RG-P-1 / US-1 / AC-P-4 에 확정. PaneTree 필드 `left/right` → `first/second` 로 방향 중립 재명명. (MP-1) 전체 EARS 문장에 REQ-P-NNN ID 부여. (MP-2) spec.md:113, :137, :159 의 EARS 레이블 오분류 재분류. (MP-3) frontmatter 를 H1 앞으로 이동, `created_at` / `updated_at` / `labels` / `revision` 추가. (M-2) PaneConstraints 를 `impl` associated const 로 단일화. (M-4) 최소 6건의 [Unwanted] / [Ubiquitous] REQ 에 negative/positive assertion AC 추가 (AC-P-20 ~ AC-P-25). (M-5) 탭 바 시각 스펙을 "bold active indicator + color" 최소값으로 직접 규정. (M-6) cwd fallback REQ-P-057 + AC-P-13a 신규. (M-7) 접근성 §6.3 에 VoiceOver / Orca / pane role / tab title source 명시. (M-8) 성능 목표 각 숫자에 근거 링크 주석. (M-9) §14 Exclusions 를 §3.2 참조 + 고유 3건 (#11~#13) 만 유지로 축약. (m-1) horizontal 용어 검증을 plan spike #3 에 추가. (m-3) AC-P-18 / AC-P-19 Requirement Group 컬럼을 `§6.1` / `§6.2` 로 교정. |
| 1.0.0 | 2026-04-24 | plan-auditor iter 2 CONDITIONAL PASS 후 annotation cycle 승인 (v1.0.0). NM-1/Nm-1/Nm-2 해소 반영. (NM-1) RG-P-6 내 REQ-P-057 (cwd fallback) 을 **REQ-P-056** 으로 rename — §5 RG-P-6 / §8 MS-3 / §10 AC-P-13a 의 Requirement Group 컬럼 갱신. (Nm-1) REQ-P-034 (Optional, pane 내부 tmux 중첩 시 OS-level 우선 처리) 에 대한 **AC-P-26** 신규 추가 (§10, MS-2). (Nm-2) REQ-P-044 (State-Driven, tab bar active 탭 시각 구분) 에 대한 **AC-P-27** 신규 추가 (§10, MS-2). (Nm-3) §6.4 Linux 포터빌리티 문단에 Ctrl+D (shell EOF) / Ctrl+W (readline word-delete-backward) / Ctrl+\\ (SIGQUIT) 의 shell 관례 충돌을 명시하고, R-9 신규 추가 및 plan Spike 4 (Linux shell 관례 실제 UX 검증) 로 연기. AC 총수 27 → 29. status draft → approved. |
| 1.1.0 | 2026-04-24 | plan-auditor (2026-04-24 전 SPEC 감사) 결과 반영. (C-001) `status: approved` → `run-in-progress-ms2-complete` 세분화 — MS-1/MS-2 run 완료, MS-3 대기. frontmatter 에 `milestone_status` 필드 추가. (M-001) §7.7 에 `TabBar` / `TabBarStyle` / `FontWeight` abstract API 추가 (T10 구현체 대응). (M-002) §6.4 "60 tests baseline" 주석 명확화 — 본 SPEC 시작 시점 (2026-04-24 pre-T1) 기준. (m-001) REQ-P-057 rename 잔재 주석 유지 (historical clarity). (m-002) §11.1 Plan Spikes 표에 Spike 4 (Linux shell 관례) 엔트리 명시 추가. (m-003) §6.2 `60 × 18 = 1080 MB` 숫자 일치 교정. |

---

## 1. 개요

### 1.1 목적

MoAI Studio v3 의 **Tab / Pane split** 레이어 (`crates/moai-studio-ui/src/panes`, `crates/moai-studio-ui/src/tabs` 신규) 를 구축한다. 단일 `TerminalSurface` 만 렌더하던 현재 RootView 를 **탭 + 이진 트리 pane tree** 구조로 확장하여, 사용자가 macOS 와 Linux 에서 플랫폼 네이티브 관례 (macOS 는 Cmd, Linux 는 Ctrl 기반) 를 통해 다중 터미널을 하나의 창 안에 배치하고 재시작 후에도 배치를 복원할 수 있게 한다.

본 SPEC 은 3 개의 milestone 으로 분할된다:

- **MS-1 Pane core**: 단일 탭 내부 binary tree pane split, 분할/닫기/focus/resize, 최소 pane 크기 제약.
- **MS-2 Tabs**: 탭 바 UI + 탭 생성/전환/닫기, 각 탭이 독립된 `PaneTree` 소유.
- **MS-3 Persistence**: 종료 시 JSON 저장, 시작 시 복원. Pane tree + tab 목록 + cwd + focus 복원. Scrollback 복원은 제외.

성공 기준: macOS 14 + Ubuntu 22.04 runner 상에서 `cargo run -p moai-studio-app` 을 실행하여 사용자가 플랫폼별 split 단축키 (macOS: Cmd+\\ / Cmd+Shift+\\, Linux: Ctrl+\\ / Ctrl+Shift+\\) 로 3-level 이상의 pane split 을 만들고, 탭 생성 단축키 (macOS: Cmd+T, Linux: Ctrl+T) 로 9개의 탭을 열어 각 탭마다 독립된 pane tree 를 유지하며, 앱을 종료 후 재시작했을 때 pane tree 구조 + 탭 목록 + 각 pane 의 cwd + focus 가 그대로 복원되는 end-to-end 동작.

### 1.2 SPEC-V3-001 / SPEC-V3-002 와의 관계

본 SPEC 은 **SPEC-V3-002 Terminal Core 의 공개 API (`Pty` trait, `PtyWorker`, `VtState`, `PtyEvent`, `TerminalSurface`) 를 수정 없이 재사용** 하는 것을 대원칙으로 한다. Pane 당 `PtyWorker` + `VtState` + `TerminalSurface` 를 독립 인스턴스로 생성하며, Terminal Core 는 pane 의 존재를 알지 못한다.

SPEC-V3-001 의 scaffold 산출물 (`RootView`, `Workspace`, `WorkspacesStore`) 중 `RootView::terminal: Option<Entity<TerminalSurface>>` 필드는 본 SPEC 에서 **pane tree entity 로 대체된다**. `Workspace` 구조체 자체는 변경 없이 유지되며, persistence 는 별도 파일 경로 (`~/.moai/studio/panes-{ws-id}.json`) 를 사용해 기존 `workspaces.json` schema 와 분리한다.

### 1.3 근거 문서

- Research findings: `.moai/specs/SPEC-V3-003/research.md` (625 LOC, 6 경쟁 레퍼런스 + 코드베이스 경계 + 기술 후보 + 위험 분석)
- 확정 scope: research.md §8 의 7개 질문에 대한 사용자 답변 (2026-04-24 AskUserQuestion 세션)
- 전제 구현: `.moai/specs/SPEC-V3-002/spec.md` v1.1.0 (completed)
- 디자인 참조: `.moai/design/v3/spec.md:420-438` (플랫폼별 키 바인딩 — macOS / Windows+Linux 이원 표)

---

## 2. 배경 및 동기

본 섹션의 상세 논거는 `.moai/specs/SPEC-V3-003/research.md` 에 있다. 여기서는 SPEC 독자가 요구사항을 읽기 전에 필요한 최소한의 맥락만 요약한다.

- **경쟁 매트릭스 수렴점** (research §1.7): cmux / Zed / WezTerm / iTerm2 4 개 제품이 모두 "workspace 당 N 탭, 탭마다 binary tree pane" + "direct-modifier 키 바인딩 (prefix 없음)" 을 채택. tmux 의 prefix key 철학은 pane 내부에서 tmux 를 실행하는 시나리오와 충돌하므로 호스트 앱에서는 기각.
- **현재 UI 구조의 한계** (research §2.2): `crates/moai-studio-ui/src/lib.rs:75` 의 `terminal: Option<Entity<TerminalSurface>>` 는 단일 터미널만 허용한다. 다중 pane 을 지원하려면 `content_area` (`lib.rs:410-444`) 의 분기 로직과 `main_body` 로의 전달 경로 (`lib.rs:184`, `lib.rs:290-299`) 를 재설계해야 한다.
- **Terminal Core 의 무변경 원칙** (research §2.1): SPEC-V3-002 의 pty/worker/vt 모듈은 pane 개념을 모른다. Pane ID 와 focus 관리는 UI 계층에서 부여한다. 이는 Terminal Core 의 74 passed test suite 를 보호하는 대원칙이다.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. 단일 탭 내부에서 pane 을 수평/수직으로 재귀 분할할 수 있다 (이진 트리).
- G2. 탭 바 UI 를 통해 N 개의 탭을 생성/전환/닫기할 수 있으며, 각 탭은 독립된 `PaneTree` 를 가진다.
- G3. 사용자가 플랫폼 로컬 modifier 조합 (macOS: Cmd, Linux: Ctrl) 의 직접 매핑 키 바인딩으로 모든 pane/tab 조작을 수행할 수 있다. Prefix key (tmux 스타일) 는 채택하지 않는다.
- G4. 앱을 종료 후 재시작했을 때 pane tree + 탭 목록 + 각 pane 의 cwd + 마지막 focus 가 복원된다 (scrollback 제외).
- G5. Pane 최소 크기 제약 (40 cols × 10 rows) 이 강제되며, 분할 불가 시 split 요청은 거부된다.
- G6. SPEC-V3-002 Terminal Core 의 공개 API 를 수정하지 않고 재사용한다.
- G7. macOS 14+ 와 Ubuntu 22.04+ 양쪽 플랫폼에서 동일한 UX (동등한 키 조합 + 동등한 AC) 를 보장한다. Windows 는 본 SPEC 범위 밖 (Non-Goal N10).

### 3.2 비목표 (Non-Goals)

- N1. **Named layout** (tmux even-horizontal / main-vertical / tiled 등) — Phase 3.1 이후.
- N2. **Pane zoom** (tmux `C-b z` 스타일 임시 최대화) — Phase 3.1 이후.
- N3. **Drag-and-drop pane 재배치** — MVP 이후.
- N4. **Scrollback 복원** — libghostty-vt terminal state 직렬화 spike 가 선행되어야 하며, 본 SPEC 범위 밖.
- N5. **Shell session 복원 (cmux 방식)** — Unix socket 서버 + agent env var 필요. 별도 SPEC 으로 이관.
- N6. **탭 reordering (drag)** — Phase 3.1 이후.
- N7. **탭 detach/reattach to separate window** — 다중 윈도우 SPEC 이 선행되어야 함.
- N8. **Pane 간 텍스트 broadcast / sync scroll** — Phase 3.1 이후.
- N9. **탭 이름 편집 / 색상 지정** — nice-to-have, 별도 SPEC.
- N10. **Windows 빌드** — Phase 7 (GPUI Windows GA 대기, SPEC-V3-002 와 동일 정책).

---

## 4. 사용자 스토리

- **US-1**: 개발자가 한 창에서 좌측에 코드 빌드, 우측 상단에 테스트 watcher, 우측 하단에 로그 tail 을 동시에 띄우고 싶다 → Horizontal split (좌/우 배치, 수직 divider) 단축키로 좌우 분할, 우측 pane 에서 Vertical split (상/하 배치, 수평 divider) 단축키로 상/하 분할. (macOS: Cmd+\\ / Cmd+Shift+\\, Linux: Ctrl+\\ / Ctrl+Shift+\\).
- **US-2**: 여러 프로젝트를 탭으로 구분하여 작업하되 각 탭마다 독립된 pane 배치를 유지하고 싶다 → 새 탭 단축키 (macOS: Cmd+T, Linux: Ctrl+T) 로 탭 생성, 탭 번호 단축키 (macOS: Cmd+1~9, Linux: Ctrl+1~9) 로 탭 전환, 각 탭의 pane tree 가 탭 간에 영향받지 않음.
- **US-3**: 앱을 실수로 닫거나 OS 가 재부팅되어도 어제의 pane 배치로 이어서 작업하고 싶다 → MS-3 에서 pane tree + cwd 복원 (shell 은 새 세션).
- **US-4**: pane 을 너무 작게 만들어 코드를 읽지 못하게 되는 일을 방지하고 싶다 → 최소 40 cols × 10 rows 제약, 조건 미달 시 split 거부.
- **US-5**: 키보드만으로 pane 간 focus 이동을 빠르게 수행하고 싶다 → prev/next pane focus 단축키 (macOS: Cmd+Shift+[/], Linux: Ctrl+Shift+[/]).
- **US-6**: pane 분리선을 마우스로 drag 하여 비율을 조정하고 싶다 → divider drag 로 sibling 간 비율 변경, 윈도우 resize 시 비율 유지.

---

## 5. 기능 요구사항 (EARS)

본 섹션의 요구사항은 7 개 그룹 (RG-P-1 ~ RG-P-7) 으로 조직되며, 각 그룹은 MS-1 / MS-2 / MS-3 중 하나 이상에 매핑된다 (§8 Milestone 정의 참조). 각 EARS 문장은 **REQ-P-NNN** ID 를 부여받아 AC 와 개별 매핑된다.

**Split 방향 용어 정의 (C-3 해소)**:

- `SplitDirection::Horizontal` = 두 child pane 이 **좌/우** 로 배치됨. Divider 는 **수직선** (세로). `first` = 왼쪽 pane, `second` = 오른쪽 pane.
- `SplitDirection::Vertical` = 두 child pane 이 **상/하** 로 배치됨. Divider 는 **수평선** (가로). `first` = 위쪽 pane, `second` = 아래쪽 pane.
- 정식 정의는 §7.1 과 §15 에 중복 기재. AC (특히 AC-P-4) 는 이 정의에 기반한다.

### RG-P-1: Pane 자료구조 및 Split 연산 (MS-1)

**REQ-P-001** [Ubiquitous] (group: RG-P-1) `moai-studio-ui::panes` 모듈은 이진 트리 구조의 `PaneTree` 자료형을 **제공해야 한다**. Leaf 는 단일 `TerminalSurface` 엔티티를 참조하고, Split 은 direction (`Horizontal` / `Vertical`) + ratio (`f32`, 0.0..1.0, 경계 제외) + `first` / `second` 두 child 를 가진다. `Horizontal` 에서 `first` = 왼쪽, `second` = 오른쪽; `Vertical` 에서 `first` = 위쪽, `second` = 아래쪽.

**REQ-P-002** [Event-Driven] (group: RG-P-1) 사용자가 현재 focused leaf pane 에 대해 horizontal split (좌/우 분할) 을 요청하면, 시스템은 해당 leaf 를 `Split { direction: Horizontal, ratio: 0.5, first, second }` 노드로 교체하고 새 `TerminalSurface` 엔티티를 `second` (오른쪽) child 로 생성**해야 한다**. Vertical split (상/하 분할) 도 동일하게 동작하되 direction 만 다르다 (새 pane 은 아래쪽).

**REQ-P-003** [Event-Driven] (group: RG-P-1) 사용자가 현재 focused leaf pane 에 대해 close 를 요청하면, 시스템은 sibling leaf 를 parent 위치로 승격 (tree rotation) 하고 PTY reader thread + VtState + TerminalSurface entity 를 1 초 이내에 정리**해야 한다**.

**REQ-P-004** [Event-Driven] (group: RG-P-1) PaneTree 에 leaf 가 단 하나만 남은 상태에서 사용자가 close 를 요청하면, 시스템은 pane 을 닫지 않고 **경고 없이 무시해야 한다** (현재 탭의 유일한 pane 은 보존). 검증: AC-P-3. *(iter 1 의 [State-Driven] 레이블을 Event-Driven 으로 재분류 — MP-2/M-1 a: `close 를 요청하면` 은 event trigger 이며 지속 조건 (while) 이 없음.)*

**REQ-P-005** [Unwanted] (group: RG-P-1) 시스템은 Split 노드의 ratio 를 `0.0` 또는 `1.0` 으로 설정**해서는 안 된다**. Drag 또는 프로그램적 조작으로 ratio 가 최소 pane 크기 제약 (RG-P-2) 을 위반하거나 경계 (0.0 / 1.0) 에 도달하는 값으로 세팅되려는 경우, 세팅은 거부되며 기존 ratio 가 유지된다. 검증: AC-P-6, AC-P-20.

### RG-P-2: Pane 최소 크기 및 Resize (MS-1)

**REQ-P-010** [Ubiquitous] (group: RG-P-2) 시스템은 모든 pane 의 최소 크기를 **40 cols × 10 rows** 로 강제**해야 한다**. 이는 `impl PaneConstraints { pub const MIN_COLS: u16 = 40; pub const MIN_ROWS: u16 = 10; }` 형태의 **associated const 공개 값** 으로 정의되며 runtime 에 변경 불가한 불변 상수이다 *(M-2: iter 1 의 struct field + 상수 병존 모호성 해소)*.

**REQ-P-011** [Event-Driven] (group: RG-P-2) 사용자가 split 요청 시 결과 pane 중 하나라도 최소 크기 조건을 충족할 수 없다면, 시스템은 split 을 거부하고 `tracing::warn!("split rejected: pane size constraint violated")` 를 **기록해야 한다**. AC-P-4 의 경계 판정은 `< MIN_COLS` 또는 `< MIN_ROWS` (strict less-than) 일 때만 거부로 한다 (정확히 40 cols 가 되면 허용).

**REQ-P-012** [Event-Driven] (group: RG-P-2) 사용자가 divider 를 drag 하여 ratio 변경을 시도할 때 두 sibling 중 하나가 최소 크기 미만이 되면, 시스템은 clamp 된 최대 허용 ratio 로 제한**해야 한다** (drag 자체는 계속 허용, 단 이동 한계에 도달).

**REQ-P-013** [State-Driven] (group: RG-P-2) 윈도우 resize 가 진행되는 동안 (while resizing) 시스템은 각 pane 의 실제 cols × rows 를 재계산하여 `PtyWorker::resize(cols, rows)` 를 호출**해야 한다**. 윈도우가 너무 작아져 전체 pane tree 가 최소 크기 제약을 충족할 수 없는 경우, 시스템은 가장 깊은 split 부터 pane 을 시각적으로 숨기되 자료구조는 유지**해야 한다**.

**REQ-P-014** [Unwanted] (group: RG-P-2) 시스템은 최소 크기 제약을 사용자 설정으로 낮추는 API 를 MS-1 범위에서 노출**해서는 안 된다** (Phase 3.1 이후 고려). 검증: AC-P-21.

### RG-P-3: Focus Routing (MS-1, MS-2)

**REQ-P-020** [Ubiquitous] (group: RG-P-3) 시스템은 workspace 당 단 **하나의 active pane** 을 유지**해야 한다**. Active pane 은 GPUI `FocusHandle` 을 소유하며, `TerminalSurface::handle_key_down` 은 active pane 에서만 호출된다.

**REQ-P-021** [Event-Driven] (group: RG-P-3) 사용자가 prev pane (macOS: Cmd+Shift+\[, Linux: Ctrl+Shift+\[) 또는 next pane (macOS: Cmd+Shift+\], Linux: Ctrl+Shift+\]) 단축키를 누르면, 시스템은 현재 탭의 `PaneTree` 를 in-order 순회한 순서로 focus 를 이동**해야 한다**.

**REQ-P-022** [Event-Driven] (group: RG-P-3) 사용자가 마우스로 특정 pane 영역을 클릭하면, 시스템은 해당 pane 을 active 로 전환하고 GPUI `FocusHandle` 을 이관**해야 한다**.

**REQ-P-023** [Complex] (group: RG-P-3) 탭이 2 개 이상 존재하는 상태에서 (while) 사용자가 탭 전환 단축키를 입력하면 (when, MS-2 활성), 시스템은 전환 대상 탭의 `last_focused_pane: Option<PaneId>` 를 복원**해야 한다**. 해당 pane 이 닫혔거나 존재하지 않으면 pane tree in-order 첫 leaf 를 focus 한다. 검증: AC-P-8. *(iter 1 의 [State-Driven] 레이블을 Complex 로 재분류 — MP-2/M-1 b: state precondition "탭 2 개 이상 존재" + event trigger "탭을 전환하면" 이 복합.)*

**REQ-P-024** [Unwanted] (group: RG-P-3) 시스템은 두 개 이상의 pane 이 동시에 focused 상태가 되도록 허용**해서는 안 된다** (GPUI FocusHandle 의 단일성과 일치). 검증: AC-P-22.

### RG-P-4: 키 바인딩 (MS-1 + MS-2)

**REQ-P-030** [Ubiquitous] (group: RG-P-4) 시스템은 다음 키 조합을 기본 바인딩으로 **제공해야 한다**. 모든 조합은 direct modifier (prefix key 없음) 방식이며, 플랫폼별로 macOS 는 Command (⌘), Linux 는 Control (Ctrl) 을 기본 modifier 로 사용한다. 이는 `.moai/design/v3/spec.md:420-438` 의 "macOS | Windows/Linux" 이원 표와 정합한다 (Windows 는 Non-Goal N10).

| 동작 | macOS | Linux | Milestone |
|------|-------|-------|-----------|
| Horizontal split (좌/우 배치, 수직 divider) | Cmd+\\ | Ctrl+\\ | MS-1 |
| Vertical split (상/하 배치, 수평 divider) | Cmd+Shift+\\ | Ctrl+Shift+\\ | MS-1 |
| Horizontal split (iTerm2 관례 대안) | Cmd+D | Ctrl+D | MS-1 |
| 현재 focused pane close (탭의 유일 pane 이면 무시) | Cmd+W | Ctrl+W | MS-1 |
| 이전 pane 으로 focus 이동 | Cmd+Shift+\[ | Ctrl+Shift+\[ | MS-1 |
| 다음 pane 으로 focus 이동 | Cmd+Shift+\] | Ctrl+Shift+\] | MS-1 |
| 새 탭 생성 (신규 탭의 pane tree 는 단일 leaf) | Cmd+T | Ctrl+T | MS-2 |
| 현재 탭 close (탭이 하나 남으면 무시) | Cmd+Shift+W | Ctrl+Shift+W | MS-2 |
| N 번째 탭으로 전환 (1-based, 10번째 이상은 미지원) | Cmd+1 ~ Cmd+9 | Ctrl+1 ~ Ctrl+9 | MS-2 |
| 이전 탭으로 전환 | Cmd+\{ | Ctrl+\{ | MS-2 |
| 다음 탭으로 전환 | Cmd+\} | Ctrl+\} | MS-2 |

**플랫폼별 주의**: Linux 에서 `Ctrl+C` / `Ctrl+V` / `Ctrl+X` 는 터미널 내부 관례 (SIGINT, paste buffer, interrupt) 와 충돌하므로 본 SPEC 의 pane/tab 조작에는 사용하지 않는다. 본 표의 Linux 컬럼은 해당 조합을 **포함하지 않는다** (Ctrl+C/V/X 는 pane 내부 앱 / OS 에 그대로 전달됨).

**REQ-P-031** [Event-Driven] (group: RG-P-4) 위 표의 조합 중 하나가 눌리면, 시스템은 해당 조작 (split / close / focus / tab 전환 등) 을 수행**해야 한다**. 검증: AC-P-9.

**REQ-P-032** [Unwanted] (group: RG-P-4) 위 표의 조합 중 하나가 눌리면, 시스템은 해당 keystroke 를 pane 내부 `TerminalSurface::handle_key_down` 에 전달**해서는 안 된다** (키 이벤트 소비). *(iter 1 의 단일 [Event-Driven] 에서 positive/negative shall 이 혼재했던 것을 MP-2/M-1 c 에 따라 REQ-P-031 positive + REQ-P-032 negative 로 분리.)*

**REQ-P-033** [Unwanted] (group: RG-P-4) 시스템은 tmux 스타일 prefix key (Ctrl-B 등) 를 기본 바인딩으로 도입**해서는 안 된다**. 사용자가 pane 내부에서 tmux 를 실행하는 것은 보장되어야 한다. 검증: AC-P-23.

**REQ-P-034** [Optional] (group: RG-P-4) 사용자가 pane 내부에서 tmux 또는 screen 을 실행하여 동일한 modifier 조합을 가로채려는 경우, 시스템은 해당 pane 이 focused 인 한 OS / GPUI 레벨에서 우선 처리하되 pane 내부 앱에는 전달하지 않는다 (REQ-P-032 와 동일 경로). 이는 macOS / Linux 네이티브 앱 관례에 부합한다.

### RG-P-5: Tab 자료구조 및 탭 바 UI (MS-2)

**REQ-P-040** [Ubiquitous] (group: RG-P-5) `moai-studio-ui::tabs` 모듈은 `TabContainer` 자료형을 **제공해야 한다**. `TabContainer` 는 `tabs: Vec<Tab>` 과 `active_tab_idx: usize` 를 가지며, 각 `Tab` 은 `id: TabId`, `title: String`, `pane_tree: Entity<PaneTree>`, `last_focused_pane: Option<PaneId>` 를 소유한다. `title` 의 초기값은 해당 탭의 첫 leaf pane 의 `cwd.file_name()` (없을 경우 `"untitled"`) 로 설정된다. 사용자 편집은 본 SPEC 범위 밖 (Non-Goal N9).

**REQ-P-041** [Ubiquitous] (group: RG-P-5) RootView 는 `TabContainer` 의 렌더 결과를 `content_area` 의 유일한 내용물로 표시**해야 한다**. Empty State CTA 는 `TabContainer` 가 비어있을 때 (`tabs.is_empty()`) 만 표시된다. 검증: AC-P-24.

**REQ-P-042** [Event-Driven] (group: RG-P-5) 사용자가 새 탭 단축키 (macOS: Cmd+T, Linux: Ctrl+T) 를 누르면, 시스템은 새 `Tab` 을 생성하고 `tabs` 의 끝에 추가한 뒤 `active_tab_idx` 를 새 탭으로 변경**해야 한다**. 새 탭의 `pane_tree` 는 단일 leaf `TerminalSurface` 로 초기화된다.

**REQ-P-043** [Event-Driven] (group: RG-P-5) 사용자가 탭 바의 close 버튼을 클릭하거나 탭 close 단축키 (macOS: Cmd+Shift+W, Linux: Ctrl+Shift+W) 를 누르면, 시스템은 해당 탭의 모든 pane (PtyWorker + VtState + TerminalSurface) 을 1 초 이내에 정리하고 탭을 제거**해야 한다**. `active_tab_idx` 는 인접 탭으로 이동한다.

**REQ-P-044** [State-Driven] (group: RG-P-5) 탭이 N 개 존재하는 동안 탭 바는 각 탭의 `title` 을 표시하고 `active_tab_idx` 와 일치하는 탭을 시각적으로 강조**해야 한다**. 강조 수단의 최소 스펙은 (a) **별도 배경색 (design token `toolbar.tab.active.background`)** + (b) **텍스트 weight `bold`** 의 **두 조건을 모두 충족** 하는 것으로 한다. `toolbar.tab.active.background` 의 정확한 색상 값은 plan 단계에서 `.moai/design/v3/system.md` Toolbar 토큰에 추가된다 *(M-5 해소: iter 1 의 "plan 에서 확정" 미확정 표현 제거, 테스트 가능한 최소 스펙을 SPEC 본문에 직접 규정)*.

**REQ-P-045** [Unwanted] (group: RG-P-5) 시스템은 탭 개수 상한을 하드코딩**해서는 안 된다**. 단, 탭 번호 단축키 (Cmd/Ctrl+1~9) 는 1-based 9번째 탭까지만 커버하며, 10번째 이상 탭은 마우스 또는 prev/next 탭 단축키 (Cmd/Ctrl+\{/\}) 로만 접근 가능하다. 검증: AC-P-25.

### RG-P-6: Persistence (MS-3)

**REQ-P-050** [Ubiquitous] (group: RG-P-6) 시스템은 `~/.moai/studio/panes-{ws-id}.json` 경로에 workspace 별 pane 상태를 **저장해야 한다**. 파일 schema version 은 `"moai-studio/panes-v1"` 으로 명시되고, 기존 `workspaces.json` (`"moai-studio/workspace-v1"`) 과 분리된다.

**REQ-P-051** [Ubiquitous] (group: RG-P-6) 저장 데이터는 다음을 포함**해야 한다**: (a) 탭 목록 (순서 유지) — 각 탭의 `id`, `title`, `last_focused_pane`, (b) 각 탭의 `PaneTree` 직렬화 — Split 노드의 direction/ratio, Leaf 의 pane metadata (cwd 포함), (c) `active_tab_idx`, (d) workspace 전역의 `schema_version` 필드.

**REQ-P-052** [Event-Driven] (group: RG-P-6) 앱이 정상 종료 시퀀스에 진입하면 (예: 윈도우 close, macOS Cmd+Q, Linux 세션 종료), 시스템은 저장 경로에 JSON 을 atomic write (임시 파일 + rename) 로 기록**해야 한다**. 비정상 종료 (crash, SIGKILL) 에 대한 저장 보장은 제공하지 않는다 (별도 SPEC).

**REQ-P-053** [Event-Driven] (group: RG-P-6) 앱 시작 시 workspace 가 활성화되면, 시스템은 `panes-{ws-id}.json` 을 읽어 탭 목록 + 각 탭의 PaneTree 구조를 복원**해야 한다**. 각 leaf pane 에 대해 새 `PtyWorker` 를 cwd 값을 작업 디렉터리로 삼아 spawn 한다. Scrollback 은 복원되지 않는다 (새 shell 세션).

**REQ-P-054** [Event-Driven] (group: RG-P-6) 저장 파일이 존재하지 않거나 schema version 이 일치하지 않거나 JSON parse 에 실패하면, 시스템은 단일 탭 + 단일 leaf pane 상태 (기본값) 로 **fallback 해야 한다**. `tracing::warn!` 로 복원 실패 원인을 기록하고, parse 실패의 경우 원본 파일을 `.corrupt` suffix 로 move 한다.

**REQ-P-055** [Unwanted] (group: RG-P-6) 시스템은 scrollback buffer, selection, cursor position 같은 VT state 를 persistence JSON 에 직렬화**해서는 안 된다**. 본 MS-3 의 복원 범위는 구조 + cwd + focus 까지로 한정된다. 검증: AC-P-12 negative assertion.

**REQ-P-056** [Event-Driven] (group: RG-P-6) 앱 시작 시 persistence 에서 읽어들인 leaf pane 의 cwd 경로가 (a) 존재하지 않거나, (b) 접근 권한이 없거나, (c) 디렉터리가 아닌 경우, 시스템은 해당 pane 의 cwd 를 `$HOME` (환경변수 또는 OS 기본 home 경로) 으로 fallback 하고 `tracing::warn!("pane cwd fallback: {saved} → $HOME (reason: {not_found|permission_denied|not_a_dir})")` 를 기록**해야 한다**. 복원 자체는 실패로 간주하지 않고 계속 진행한다. 검증: AC-P-13a *(M-6 해소: iter 1 의 §6.5 warn 로그 계획 대비 요구 문장 + AC 쌍 신규. v1.0.0: NM-1 해소로 REQ-P-057 → REQ-P-056 rename)*.

### RG-P-7: Terminal Core 호환성 및 추상 인터페이스 (전체 MS)

**REQ-P-060** [Ubiquitous] (group: RG-P-7) 시스템은 SPEC-V3-002 의 공개 API (`Pty` trait, `PtyWorker`, `VtState`, `PtyEvent`, `TerminalSurface`) 를 변경 없이 **재사용해야 한다**. Pane ID, focus, tab 개념은 UI 계층에서 부여하며 Terminal Core 는 이를 알지 못한다. 검증: AC-P-16.

**REQ-P-061** [Ubiquitous] (group: RG-P-7) `moai-studio-ui::panes` 모듈은 `PaneSplitter` 추상 trait 을 **제공해야 한다**. 해당 trait 은 `split_horizontal`, `split_vertical`, `close_pane`, `focus_pane` 메서드를 정의하며, 구체 구현체의 채택 (gpui-component 의 Resizable Panel vs. 자체 구현) 은 plan 단계의 spike 결과로 결정된다. 검증: AC-P-17.

**REQ-P-062** [Ubiquitous] (group: RG-P-7) `moai-studio-ui::panes` 모듈은 `ResizableDivider` 추상 trait 을 **제공해야 한다**. 해당 trait 은 drag 이벤트를 받아 sibling ratio 를 업데이트하는 책임을 가진다. 구체 구현체는 plan 단계에서 결정된다.

**REQ-P-063** [Unwanted] (group: RG-P-7) 시스템은 gpui-component crate 에 대한 의존성을 spec 단계에서 확정**해서는 안 된다**. 본 SPEC 의 acceptance criteria 는 `PaneSplitter` / `ResizableDivider` trait 기반으로 검증 가능해야 하며, 직접 구현 / gpui-component 채택 어느 쪽으로 plan 결정이 내려져도 AC 재작성이 필요 없어야 한다.

---

## 6. 비기능 요구사항

### 6.1 성능

각 성능 목표의 근거 (M-8 해소):

- Pane split 요청부터 새 `TerminalSurface` 의 첫 프레임 paint 까지 **≤ 200 ms**. 근거: SPEC-V3-002 §5.1 (초기화 latency 동일 기준, 기존 AC-T 벤치마크 상속).
- Divider drag 중 프레임 간격 **≤ 16.67 ms** (60 fps 표준 refresh), ratio 갱신 → `PtyWorker::resize` 호출 → re-paint 왕복 latency **≤ 33 ms** (2 frame). 근거: 60 fps UI 표준 + GPUI render loop 관례.
- 탭 전환 소요 시간 **≤ 50 ms** (visible frame 기준). 근거: iTerm2 / Zed 경쟁 벤치마크의 체감 상한 (research §4 및 일반적 human perception threshold ~100ms 대비 여유).
- 9 tab × 평균 2 pane = 18 pane 동시 활성 상태에서 idle CPU **≤ 3%** (전체 프로세스 기준). 근거: SPEC-V3-002 단일 pane idle < 0.3% × 18 ≈ 5.4% 를 3% 로 타이트하게 설정하여 PtyWorker wake-up batch 최적화 유도. 실측은 plan 단계 benchmark 신설 (§11.2 CI 게이트 연계).

### 6.2 메모리

- Pane 당 RSS 증분 **≤ 60 MB** (SPEC-V3-002 §5.2 기준 재활용, scrollback 10K rows 포함). 18 pane 기준 약 1.1 GB 상한 (MBP 16GB 기준 허용 범위).
- PaneTree / TabContainer 자료구조 자체의 오버헤드 **≤ 1 MB / 100 pane**. 근거: `PaneTree` enum (Box + f32 + direction enum) ≈ 32 B + `Tab` struct ≈ 80 B × 100 < 50 KB (20× safety margin).
- Persistence JSON 파일 크기 **≤ 64 KB / workspace**. 근거: 18 pane × ~300 B/leaf (cwd path + pane_id + ratio) + 9 tab × ~100 B ≈ 6 KB. 10× safety margin.

### 6.3 접근성

- 모든 pane/tab 조작은 키보드 전용으로 가능해야 한다 (RG-P-4).
- 탭 바의 active tab indicator 는 색상 외에도 **텍스트 weight `bold`** 로도 구분**해야 한다** (REQ-P-044 의 (a)+(b) 두 조건 동시 요구. 색맹 사용자 배려 + 색 + 비-색 이중 표시).
- Divider 는 최소 **4 pt** 의 drag hit area 를 가진다.
- Minimum pane size 위반 시 UI 피드백은 시각적 **shake 애니메이션 (200ms duration, ±3px horizontal translation)** 과 `tracing::warn!` 로그 두 가지를 병행하며, 사운드 경고는 제공하지 않는다.
- Screen reader 지원 (M-7 해소):
  - macOS VoiceOver 및 Linux Orca 에 대해, 각 pane 은 GPUI accessibility role `pane` 을 전달해야 한다 (GPUI 0.2.2 의 accessibility API 가 해당 role 을 미지원하는 경우 가장 근접한 `group` role 로 fallback).
  - 탭 바는 role `tab_list`, 각 탭은 role `tab` 을 전달한다. Active tab 은 `aria-selected=true` 의미에 해당하는 속성을 설정한다.
  - 각 pane / tab 의 accessible label 은 해당 `title` 필드 값이다.
  - VoiceOver / Orca 호환 수준은 "focus 변경이 음성으로 안내되는 수준" 까지를 본 SPEC 의 최소 보장으로 한정한다 (고급 landmark navigation 은 별도 SPEC).
- RTL (아랍어 / 히브리어) locale 에서의 pane 좌/우 배치는 시스템 기본 LTR 방향을 유지한다 (전용 RTL 지원은 별도 SPEC).

### 6.4 이식성 / 안정성

- 기존 `moai-studio-terminal` 74 tests + `moai-studio-ui` 60 tests 의 regression 0.
- **macOS 14+ / Ubuntu 22.04+ 양 플랫폼 대칭 (G7)**: RG-P-4 의 모든 키 바인딩은 두 플랫폼에서 등가 동작을 제공해야 하며, 각 AC 는 두 플랫폼에서 각각 검증된다 (특히 AC-P-9, AC-P-12 / AC-P-13). Windows 는 범위 밖 (Non-Goal N10).
- **Linux 터미널 관례와 충돌하는 host 바인딩 (v1.0.0 Nm-3 해소)**: RG-P-4 표의 Linux 컬럼에는 `Ctrl+D`, `Ctrl+W`, `Ctrl+\\` 이 포함되어 있으나, 이 조합들은 Unix shell 의 장기 관례와 충돌한다 — 구체적으로 **Ctrl+D = shell EOF (`exit` 트리거)**, **Ctrl+W = readline `unix-word-rubout` (단어 단위 backspace)**, **Ctrl+\\ = SIGQUIT (core dump 포함 프로세스 강제 종료)**. 본 SPEC 은 디자인 원천 (`.moai/design/v3/spec.md:420-438`) 을 따라 이들을 host 바인딩으로 수용하되, **plan Spike 4** 에서 `Ctrl+Shift+...` 계열로 이중 modifier shift 가능성을 실제 Linux 셸 세션 UX 로 검증한 뒤 최종 결정한다. 최종 결정 전까지 Linux 빌드의 해당 바인딩은 **기본 활성화** 하되, 사용자 설정 파일 (향후 Shortcut Customization SPEC, Exclusion #12) 에서 개별 비활성화 또는 shift-escalation 이 가능하도록 설계 여지를 남긴다. 본 SPEC 의 AC-P-9b 는 이 trade-off 를 전제로 한 검증이며, Spike 4 이 (a) 현행 유지 경로를 선택한 경우 AC 무변경, (b) shift-escalation 경로를 선택한 경우 plan 단계에서 RG-P-4 Linux 컬럼과 AC-P-9b 가 동기화된다.
- SPEC-V3-002 의 MSRV (1.93) 를 유지, 상향 없음.

### 6.5 관측성

- `tracing` crate 사용, 최소 로그 level:
  - `info!`: pane split / close, tab 생성 / close, persistence save / load.
  - `warn!`: split rejected (size constraint), persistence schema mismatch, persistence parse failure, cwd recovery failure (REQ-P-056).
  - `debug!`: divider drag ratio update, focus transition.

---

## 7. 아키텍처 (추상 인터페이스)

본 섹션은 추상 trait + 자료구조의 최소 형식만 제시한다. 실제 구현 코드, 필드 이름 세부 사항, 오류 타입 정의 등은 plan.md 의 task 분해에서 확정된다.

### 7.1 PaneTree

**Split 방향 정의 (C-3 해소, §15 와 일치)**:

- `SplitDirection::Horizontal` → 두 child 가 **좌/우** 로 배치됨. Divider 는 **수직선**. `first` = 왼쪽 pane, `second` = 오른쪽 pane.
- `SplitDirection::Vertical` → 두 child 가 **상/하** 로 배치됨. Divider 는 **수평선**. `first` = 위쪽 pane, `second` = 아래쪽 pane.

```rust
// crates/moai-studio-ui/src/panes/tree.rs
pub enum PaneTree {
    Leaf(Entity<TerminalSurface>),
    Split {
        direction: SplitDirection,
        ratio: f32, // 0.0 < ratio < 1.0, 경계 제외 (REQ-P-005), 최소 크기 clamp (RG-P-2)
        first: Box<PaneTree>,   // Horizontal -> 왼쪽, Vertical -> 위쪽
        second: Box<PaneTree>,  // Horizontal -> 오른쪽, Vertical -> 아래쪽
    },
}

pub enum SplitDirection {
    Horizontal, // 좌/우 배치, 수직 divider
    Vertical,   // 상/하 배치, 수평 divider
}
```

*(iter 1 의 `left` / `right` 필드명은 Vertical split 에서 의미가 상실되므로 방향 중립의 `first` / `second` 로 재명명 — C-3 해소.)*

### 7.2 PaneSplitter (추상 trait, 구현체 미정)

```rust
pub trait PaneSplitter {
    fn split_horizontal(&mut self, target: PaneId) -> Result<PaneId, SplitError>;
    fn split_vertical(&mut self, target: PaneId) -> Result<PaneId, SplitError>;
    fn close_pane(&mut self, target: PaneId) -> Result<(), CloseError>;
    fn focus_pane(&mut self, target: PaneId);
}
```

구현체 후보: (a) 자체 구현 (GPUI mouse event + flex basis), (b) `longbridge/gpui-component` 의 Resizable Panel wrapper. 선택은 plan 단계의 2 시간 spike 결과에 따른다.

### 7.3 ResizableDivider (추상 trait, 구현체 미정)

```rust
pub trait ResizableDivider {
    fn on_drag(&mut self, delta_px: f32, total_px: f32) -> f32; // new ratio, clamped
    fn min_ratio_for(&self, sibling_px: f32) -> f32;
    // PaneConstraints::MIN_COLS / MIN_ROWS associated const 를 직접 참조
}
```

### 7.4 TabContainer

```rust
// crates/moai-studio-ui/src/tabs/container.rs
pub struct TabContainer {
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
}

pub struct Tab {
    pub id: TabId,
    pub title: String,           // 초기값: cwd.file_name() 또는 "untitled" (REQ-P-040)
    pub pane_tree: Entity<PaneTree>,
    pub last_focused_pane: Option<PaneId>,
}
```

### 7.5 PersistenceSchema

```jsonc
// ~/.moai/studio/panes-{ws-id}.json
{
  "$schema": "moai-studio/panes-v1",
  "workspace_id": "ws-abcdef",
  "active_tab_idx": 0,
  "tabs": [
    {
      "id": "tab-0001",
      "title": "main",
      "last_focused_pane": "pane-0003",
      "pane_tree": {
        "Split": {
          "direction": "Horizontal",
          "ratio": 0.5,
          "first":  { "Leaf": { "pane_id": "pane-0001", "cwd": "/Users/goos/proj" } },
          "second": {
            "Split": {
              "direction": "Vertical",
              "ratio": 0.6,
              "first":  { "Leaf": { "pane_id": "pane-0002", "cwd": "/Users/goos/proj" } },
              "second": { "Leaf": { "pane_id": "pane-0003", "cwd": "/tmp" } }
            }
          }
        }
      }
    }
  ]
}
```

### 7.6 PaneConstraints

```rust
pub struct PaneConstraints;

impl PaneConstraints {
    pub const MIN_COLS: u16 = 40;
    pub const MIN_ROWS: u16 = 10;
}
```

*(M-2 해소: iter 1 의 struct field 와 상수 병존 모호성을 제거. 불변 associated const 단일화.)*

### 7.7 TabBar / TabBarStyle / FontWeight (v1.1.0 신규)

T10 구현체 (`crates/moai-studio-ui/src/tabs/bar.rs`, commit 4428e93) 의 공개 API 를 spec 추상 형태로 명시. plan-auditor 2026-04-24 감사 M-001 대응.

```rust
pub enum FontWeight {
    Normal,
    Medium,
    Bold,
}

pub struct TabBarStyle {
    pub active_bg: u32,              // = tokens::TOOLBAR_TAB_ACTIVE_BG (= BG_SURFACE_3 = 0x232327)
    pub inactive_bg: u32,            // = tokens::BG_SURFACE
    pub active_font_weight: FontWeight,    // Bold (AC-P-27 직접 근거)
    pub inactive_font_weight: FontWeight,  // Normal 또는 Medium
    pub active_fg: u32,              // FG_PRIMARY
    pub inactive_fg: u32,            // FG_SECONDARY
}

pub struct TabBar<L: Clone + 'static> { /* ... */ }

impl<L: Clone + 'static> TabBar<L> {
    pub fn style_for(idx: usize, active_idx: usize) -> TabBarStyle;
    pub fn is_active(idx: usize, active_idx: usize) -> bool;
    // 실제 GPUI 렌더 래퍼 메서드는 thin wrapper — 테스트 영역 외.
}
```

USER-DECISION (design-token-color-value): **(a) BG_SURFACE_3** 확정 (2026-04-24). `TOOLBAR_TAB_ACTIVE_BG` alias 는 `lib.rs::tokens` 에서 노출되며 `BG_SURFACE_3` (0x232327) 과 동일. sidebar 의 active workspace row 색상과 일관성 유지.

AC-P-27 (v1.0.0 Nm-2) 직접 검증: `tabs::bar::tests::active_tab_is_bold` + `active_tab_uses_bg_surface_3` + `inactive_tab_is_not_bold` (T10 산출, 8 unit tests).

---

## 8. Milestone 정의

각 milestone 은 독립 실행 가능한 Run phase 진입 단위이다. MS-1 완료 시점에 단일 탭 + pane 기본 동작이 end-to-end 로 시연 가능해야 하며, MS-2 는 MS-1 의 결과 위에 탭 레이어를 추가, MS-3 는 MS-1 + MS-2 전제로 persistence 만 추가한다.

### MS-1: Pane core

- **범위**: 단일 탭 내부 binary tree pane split. 분할 / 닫기 / focus / drag resize / 최소 pane 크기.
- **포함 요구사항**: RG-P-1 전체 (REQ-P-001 ~ REQ-P-005), RG-P-2 전체 (REQ-P-010 ~ REQ-P-014), RG-P-3 의 REQ-P-020 / REQ-P-021 / REQ-P-022 / REQ-P-024, RG-P-4 의 MS-1 바인딩 6 건 + REQ-P-030 ~ REQ-P-034 (MS-1 부분), RG-P-7 전체.
- **제외**: 탭 개념, persistence, 탭 관련 키 바인딩.
- **시연 가능 상태**: 사용자가 horizontal split / vertical split 단축키 (플랫폼별) 로 3 level 이상의 pane split 을 만들고, close 단축키로 pane 을 닫으며 (단일 pane 으로 축소되면 무시), divider 를 drag 하여 resize 할 수 있다.

### MS-2: Tabs

- **범위**: 탭 바 UI + 탭 생성 / 전환 / 닫기. 각 탭이 독립 `PaneTree` 소유.
- **포함 요구사항**: REQ-P-023 (탭 전환 시 last-focused-pane 복원), RG-P-4 의 MS-2 바인딩 5 건 (Cmd/Ctrl+T / Cmd/Ctrl+Shift+W / Cmd/Ctrl+1~9 / Cmd/Ctrl+\{ / Cmd/Ctrl+\}), RG-P-5 전체 (REQ-P-040 ~ REQ-P-045).
- **제외**: 탭 reordering (drag), 탭 이름 편집, persistence.
- **시연 가능 상태**: 사용자가 새 탭 단축키로 9 개 탭을 생성하여 각 탭에서 독립된 pane split 구조를 만들고, 탭 번호 단축키 (Cmd/Ctrl+1~9) 로 전환 시 각 탭의 pane tree + last focus 가 보존된다.

### MS-3: Persistence

- **범위**: 종료 시 JSON 저장, 시작 시 복원. Pane tree + tab 목록 + cwd + focus 복원. Scrollback 제외.
- **포함 요구사항**: RG-P-6 전체 (REQ-P-050 ~ REQ-P-056). 기존 MS-1 / MS-2 요구사항 regression 없음.
- **제외**: Shell session 복원, scrollback 복원, 실시간 checkpoint.
- **시연 가능 상태**: 사용자가 3 탭 × 각 2 pane 상태로 앱을 종료 후 재시작했을 때 동일한 탭 / pane 구조가 복원되고 (shell 은 새 세션), 각 pane 의 cwd 가 저장 시점과 일치한다. 저장된 cwd 가 재시작 시 삭제되어 있을 경우 $HOME 으로 fallback (REQ-P-056) 된다.

---

## 9. 파일 레이아웃 (canonical)

### 9.1 신규

- `crates/moai-studio-ui/src/panes/mod.rs` — `PaneTree`, `PaneId`, `SplitDirection`, `PaneConstraints` 노출.
- `crates/moai-studio-ui/src/panes/tree.rs` — `PaneTree` enum + in-order iterator + split/close 알고리즘.
- `crates/moai-studio-ui/src/panes/splitter.rs` — `PaneSplitter` trait (MS-1), 구현체는 plan 후 결정.
- `crates/moai-studio-ui/src/panes/divider.rs` — `ResizableDivider` trait (MS-1), drag event handler.
- `crates/moai-studio-ui/src/panes/focus.rs` — focus routing (prev/next pane 단축키, mouse click).
- `crates/moai-studio-ui/src/tabs/mod.rs` — `TabContainer`, `Tab`, `TabId` 노출 (MS-2).
- `crates/moai-studio-ui/src/tabs/container.rs` — `TabContainer` 구현 + 탭 전환 로직.
- `crates/moai-studio-ui/src/tabs/bar.rs` — 탭 바 GPUI element (MS-2).

### 9.2 수정

- `crates/moai-studio-ui/src/lib.rs:75` — `terminal: Option<Entity<TerminalSurface>>` → `tab_container: Option<Entity<TabContainer>>` 로 교체.
- `crates/moai-studio-ui/src/lib.rs:184` — `main_body` 에 전달하는 타입을 `Option<Entity<TabContainer>>` 로 변경.
- `crates/moai-studio-ui/src/lib.rs:290-299` — `main_body(...)` 파라미터 시그니처 확장.
- `crates/moai-studio-ui/src/lib.rs:410-444` — `content_area` 분기를 (tab_container.is_some) 기준으로 변경, Empty State 는 `tabs.is_empty()` 일 때만.
- `crates/moai-studio-workspace/src/persistence.rs` 신규 또는 확장 — `~/.moai/studio/panes-{ws-id}.json` 읽기/쓰기 함수 (MS-3).

### 9.3 변경 금지 (Terminal Core)

- `crates/moai-studio-terminal/**` — 전체 변경 금지. 재사용만 허용.
- `crates/moai-studio-terminal/src/pty/mod.rs` — `Pty` trait 유지.
- `crates/moai-studio-terminal/src/worker.rs` — `PtyWorker` 유지.
- `crates/moai-studio-terminal/src/vt.rs` — `VtState` 유지.
- `crates/moai-studio-terminal/src/events.rs` — `PtyEvent` 유지.
- `crates/moai-studio-terminal/src/libghostty_ffi.rs` — FFI boundary 유지.

---

## 10. Acceptance Criteria

| AC ID | Requirement Group | Milestone | Given | When | Then | 검증 수단 |
|-------|-------------------|-----------|-------|------|------|-----------|
| AC-P-1 | RG-P-1 (REQ-P-002) | MS-1 | 단일 탭 + 단일 leaf pane | 사용자가 horizontal split 단축키 입력 (macOS: Cmd+\\, Linux: Ctrl+\\) | PaneTree 가 `Split { direction: Horizontal, ratio: 0.5, first, second }` 로 교체되고 `second` (오른쪽) leaf 에 새 TerminalSurface + PtyWorker 가 spawn 된다 | cargo test `panes::tree::tests::split_horizontal_from_leaf` |
| AC-P-2 | RG-P-1 (REQ-P-003) | MS-1 | 3 level split 된 PaneTree (8 leaf) | 사용자가 한 leaf 에서 close 단축키 입력 | sibling 이 parent 위치로 승격되어 7 leaf 가 되며, 닫힌 pane 의 PtyWorker + VtState 는 1 초 이내 drop 된다 | cargo test + lsof FD count assert |
| AC-P-3 | RG-P-1 (REQ-P-004) | MS-1 | 단일 leaf pane (탭의 유일 pane) | 사용자가 close 단축키 입력 | 상태 변경 없음, 경고 로그 없음, pane 은 유지된다 | cargo test `panes::tree::tests::close_last_leaf_is_noop` |
| AC-P-4 | RG-P-2 (REQ-P-011) | MS-1 | 윈도우 크기가 60 cols × 20 rows, 단일 leaf | 사용자가 horizontal split 단축키 입력 (horizontal = 좌/우 분할 → 결과: 좌 30 cols / 우 30 cols, 세로는 20 rows 유지). 경계 판정은 strict `< MIN_COLS`: 30 < 40 이므로 거부 | split 요청 거부, `tracing::warn!("split rejected: ...")` 로그 1건 기록, PaneTree 는 변경 없음 | cargo test + tracing test subscriber |
| AC-P-5 | RG-P-2 (REQ-P-013) | MS-1 | 윈도우를 축소하여 일부 pane 이 최소 크기 미달 | 윈도우 resize 진행 중 | 가장 깊은 pane 부터 시각적으로 숨기되 PaneTree 구조는 유지, 윈도우가 다시 커지면 복원 | integration test (headless resize simulation) |
| AC-P-6 | RG-P-2 (REQ-P-012) + RG-P-1 (REQ-P-005) | MS-1 | 이웃한 2 leaf sibling | 사용자가 divider drag 시도 | ratio 는 최소 pane 크기 제약 내로 clamp 되며, 양 pane 모두 최소 40 cols × 10 rows 이상 유지, ratio 는 `0.0 < ratio < 1.0` 경계 내에 머문다 | cargo test `panes::divider::tests::drag_clamps_ratio` + manual |
| AC-P-7 | RG-P-3 (REQ-P-021) | MS-1 | 3 leaf 가 있는 PaneTree, pane-A 가 focused | 사용자가 next pane 단축키 입력 (macOS: Cmd+Shift+\], Linux: Ctrl+Shift+\]) | Focus 가 in-order 다음 leaf 로 이동, `TerminalSurface::handle_key_down` 은 새 focused pane 에서만 호출 | cargo test + GPUI FocusHandle assertion |
| AC-P-8 | RG-P-3 (REQ-P-023) | MS-2 | Tab 2 개, 각 탭이 2 pane, tab A 의 pane-2 에 focus | 사용자가 Cmd/Ctrl+2 로 탭 B 로 전환 후 다시 Cmd/Ctrl+1 로 탭 A 복귀 | 탭 A 의 last_focused_pane (pane-2) 이 복원됨 | cargo test `tabs::container::tests::tab_switch_restores_focus` |
| AC-P-9a | RG-P-4 (REQ-P-030, REQ-P-031, REQ-P-032) | MS-1 + MS-2 (macOS) | macOS 14 runner, 단일 탭 + 단일 pane | 사용자가 Cmd+T → Cmd+\\ → Cmd+Shift+\\ → Cmd+W → Cmd+Shift+\[ 순차 입력 | 각 조합이 RG-P-4 표의 macOS 컬럼 동작 수행, keystroke 는 pane 내부 `TerminalSurface::handle_key_down` 에 전달되지 않음 | cargo test + macOS CI job |
| AC-P-9b | RG-P-4 (REQ-P-030, REQ-P-031, REQ-P-032) | MS-1 + MS-2 (Linux) | Ubuntu 22.04 runner, 단일 탭 + 단일 pane | 사용자가 Ctrl+T → Ctrl+\\ → Ctrl+Shift+\\ → Ctrl+W → Ctrl+Shift+\[ 순차 입력 | 각 조합이 RG-P-4 표의 Linux 컬럼 동작 수행, keystroke 는 pane 내부 `TerminalSurface::handle_key_down` 에 전달되지 않음 | cargo test + Linux CI job *(C-2 해소)* |
| AC-P-10 | RG-P-5 (REQ-P-042) | MS-2 | Workspace 활성, 탭 없음 | 사용자가 새 탭 단축키를 9 회 반복 | 9 개 탭 생성, 각 탭의 pane tree 는 단일 leaf, `active_tab_idx` 는 8 (0-based 마지막) | cargo test `tabs::container::tests::create_nine_tabs` |
| AC-P-11 | RG-P-5 (REQ-P-041) | MS-2 | 9 개 탭, 각 탭이 독립 pane 구조 (탭 A 는 2 pane, 탭 B 는 4 pane …) | 사용자가 Cmd/Ctrl+1~9 를 임의 순서로 전환 | 각 탭의 PaneTree 는 전환 후 구조 / TerminalSurface 동일성 보존 | cargo test + integration (GPUI headless 범위) |
| AC-P-12 | RG-P-6 (REQ-P-050, REQ-P-051, REQ-P-052, REQ-P-055) | MS-3 | 3 탭 × 각 2 pane 상태 | 앱 정상 종료 (윈도우 close, macOS 와 Linux 각각) | `~/.moai/studio/panes-{ws-id}.json` 이 atomic write 로 생성, `$schema = "moai-studio/panes-v1"`, 모든 탭 / pane / cwd 가 기록됨. **Negative assertion**: JSON 에 `scrollback`, `selection`, `cursor_position`, `vt_state` 키가 존재하지 않아야 한다 (REQ-P-055) | integration test (tempdir + fs assertion) — macOS 와 Linux 각각 실행 |
| AC-P-13 | RG-P-6 (REQ-P-053) | MS-3 | AC-P-12 에서 생성된 JSON 존재, 모든 cwd 가 유효한 디렉터리 | 앱 재시작 (macOS / Linux 각각) | 탭 / pane tree / cwd / active_tab_idx / last_focused_pane 이 저장 시점과 일치하도록 복원, 각 pane 의 shell 은 저장된 cwd 를 작업 디렉터리로 새로 spawn | integration test |
| AC-P-13a | RG-P-6 (REQ-P-056) | MS-3 | panes-{ws-id}.json 의 한 leaf cwd 가 저장 시점 이후 삭제 (rm -rf) 됨 | 앱 재시작 | 해당 pane 은 `$HOME` 을 cwd 로 spawn 되며, `tracing::warn!("pane cwd fallback: ... (reason: not_found)")` 로그 1건 기록. 다른 pane 의 복원은 영향받지 않음 | cargo test + tempdir + subscribe tracing *(M-6 해소 신규 AC. v1.0.0: NM-1 해소로 REQ-P-057 → REQ-P-056)* |
| AC-P-14 | RG-P-6 (REQ-P-054) | MS-3 | panes-{ws-id}.json 의 schema_version 이 `"moai-studio/panes-v2"` (미래 버전) | 앱 시작 | 단일 탭 + 단일 leaf fallback, `tracing::warn!("... schema version mismatch")` 로그 기록 | cargo test |
| AC-P-15 | RG-P-6 (REQ-P-054) | MS-3 | panes-{ws-id}.json 파일이 손상되어 JSON parse 실패 | 앱 시작 | 단일 탭 + 단일 leaf fallback, `tracing::warn!("... panes file parse failed")` 로그 기록, 기존 파일은 `.corrupt` suffix 로 move | cargo test |
| AC-P-16 | RG-P-7 (REQ-P-060) | 전체 | Terminal Core crate | MS-1 / MS-2 / MS-3 구현 완료 후 `cargo test -p moai-studio-terminal` 실행 | SPEC-V3-002 의 74 tests 모두 통과, 기존 API 변경 없음 | CI gate |
| AC-P-17 | RG-P-7 (REQ-P-061) | MS-1 | `panes` 모듈 | cargo build | `PaneSplitter`, `ResizableDivider` 추상 trait 정의가 존재, 구체 구현체 선택 없이도 컴파일 가능 (mock 구현체가 있음) | cargo check + doc test |
| AC-P-18 | §6.1 (성능) | MS-1 | 9-leaf PaneTree | 1 개 pane 에서 horizontal split 단축키 수행 | 새 TerminalSurface 첫 프레임 paint ≤ 200 ms (criterion benchmark) | benches/pane_split.rs *(m-3 해소: Requirement Group 을 `§6.1` 로 교정)* |
| AC-P-19 | §6.1 (성능 — 탭 전환 ≤ 50ms) | MS-2 | 9 탭, 각 2 pane | Cmd/Ctrl+1 ↔ Cmd/Ctrl+9 50 회 왕복 | 평균 탭 전환 visible frame ≤ 50 ms | benches/tab_switch.rs *(m-3 해소: Requirement Group 을 `§6.1` 로 교정)* |
| AC-P-20 | RG-P-1 (REQ-P-005) | MS-1 | PaneTree 의 한 Split 노드 | 프로그램적으로 `set_ratio(0.0)` 또는 `set_ratio(1.0)` 호출 (테스트 API) | 세팅 거부, 기존 ratio 유지, 오류 Result 반환 | cargo test `panes::tree::tests::ratio_boundary_rejected` *(M-4 해소 신규 negative AC)* |
| AC-P-21 | RG-P-2 (REQ-P-014) | MS-1 | 공개 `PaneConstraints` API | cargo doc / public API surface 검사 | `PaneConstraints::MIN_COLS` 와 `MIN_ROWS` 는 `pub const` 로만 노출. `set_min_cols`, `set_min_rows`, `PaneConstraints::new(...)` 같은 가변 API 가 공개 API 에 존재하지 않음 | `cargo public-api` 또는 수동 rustdoc 검사 *(M-4 해소 신규 negative AC)* |
| AC-P-22 | RG-P-3 (REQ-P-024) | MS-1 | 3 leaf PaneTree | 임의 순서의 focus 전환 시퀀스 (Cmd/Ctrl+Shift+\[/\] + mouse click) | 매 시점 active focused pane 수는 정확히 1 개. `assert_eq!(focused_panes.len(), 1)` 가 모든 시퀀스 단계에서 성립 | cargo test `panes::focus::tests::single_focus_invariant` *(M-4 해소 신규 negative AC)* |
| AC-P-23 | RG-P-4 (REQ-P-033) | MS-1 | pane 내부에서 `tmux` 실행 중, pane focused | 사용자가 `Ctrl+B` 입력 (tmux prefix) | `Ctrl+B` 는 pane 내부 `TerminalSurface::handle_key_down` 에 그대로 전달됨 (OS/GPUI 는 가로채지 않음). tmux 가 정상적으로 prefix 를 수신하여 다음 키를 대기 | manual test + pty echo 검증 *(M-4 해소 신규 AC, tmux 호환성 보증)* |
| AC-P-24 | RG-P-5 (REQ-P-041) | MS-2 | 새로 생성된 workspace, `TabContainer.tabs.is_empty() == true` | RootView 렌더 | Empty State CTA 만 표시, 탭 바는 렌더되지 않음 (또는 0 탭으로 빈 바). 첫 Cmd/Ctrl+T 입력 시 탭 바 출현 | cargo test + GPUI headless render *(M-4 해소 신규 AC)* |
| AC-P-25 | RG-P-5 (REQ-P-045) | MS-2 | 12 개 탭 생성 상태 | Cmd/Ctrl+1~9 로 각각 전환 시도 + 마우스로 10번째 탭 클릭 + Cmd/Ctrl+\} 로 탐색 | Cmd/Ctrl+1~9 는 1~9번째 탭만 활성화 (10번째는 영향 없음). 마우스 클릭과 Cmd/Ctrl+\} 는 10번째 이상 탭도 정상 전환 | cargo test + integration *(M-4 해소 신규 AC)* |
| AC-P-26 | RG-P-4 (REQ-P-034) | MS-2 | 한 pane 의 `$SHELL` 내부에서 `tmux new-session` 으로 중첩 tmux 가 실행 중이고 해당 pane 이 focused | 사용자가 새 탭 단축키 입력 (macOS: Cmd+T, Linux: Ctrl+T) | (a) pane 내부 tmux 는 해당 key event 를 **수신하지 않는다** — PTY echo 로그 / pty master 로 쓰여진 raw byte stream 에 Cmd+T / Ctrl+T 에 대응하는 escape sequence (예: `\x1b[...T`) 가 **부재** 함을 byte-level assertion 으로 검증. **AND** (b) host 앱은 새 탭을 생성하고 `active_tab_idx` 가 1 증가한다 | integration test (macOS + Linux 양 플랫폼), PTY feed 기록 검사 *(v1.0.0 Nm-1 해소 신규 AC: REQ-P-034 Optional 의 "OS / GPUI 레벨 우선 처리" 를 관측 가능한 검증 기준으로 고정)* |
| AC-P-27 | RG-P-5 (REQ-P-044) | MS-2 | `TabContainer.tabs.len() >= 2` 이고 `active_tab_idx = 0` 인 상태 | RootView 가 탭 바를 렌더 | 탭 index 0 에 해당하는 GPUI element 는 동시에 (a) background color = design token `toolbar.tab.active.background` (정확히 일치), **AND** (b) text font-weight = `bold` 두 속성을 모두 가진다. 비활성 탭 (index >= 1) 은 둘 중 어느 조건도 만족하지 않는다 | GPUI snapshot test 또는 unit test — styled element 속성 추출 assert (`assert_eq!(tab[0].bg, active_bg); assert_eq!(tab[0].font_weight, FontWeight::Bold)`) *(v1.0.0 Nm-2 해소 신규 AC: REQ-P-044 의 "(a) + (b) 동시 충족" 을 실행 가능한 테스트로 고정)* |

---

## 11. 의존성 및 제약

### 11.1 외부 의존성

| Crate | 버전 / 상태 | 비고 |
|-------|-------------|------|
| `gpui` | 0.2.2 (SPEC-V3-002 와 동일) | 변경 없음 |
| `gpui-component` (longbridge) | **미정** | plan 단계의 2 시간 spike 결과로 결정. 도입 시 버전 확정. |
| `serde` / `serde_json` | workspace 기본 | Persistence JSON 직렬화 (MS-3) |
| `uuid` 또는 기존 ID 생성 로직 | 선택 | PaneId / TabId 생성. `format!("pane-{:04x}", …)` 패턴으로 자체 구현도 가능. 기존 workspace ID 생성 패턴과의 정합성은 plan 단계 확인 (A-3 연기). |

C-1. gpui-component 도입 여부는 plan 단계에서 GPUI 0.2.2 의 divider drag API 존재 여부 + longbridge/gpui-component 안정성 각 1 시간 spike 후 결정한다. 본 spec 의 AC 는 양쪽 모두에서 검증 가능한 추상 trait 기반이다.

**Plan Spikes (1~3 시간씩)**:
1. GPUI 0.2.2 divider drag API 검증 (1h).
2. longbridge/gpui-component Resizable Panel 안정성 평가 (1h).
3. iTerm2 Cmd+D / Cmd+Shift+D 의 공식 명칭 (`horizontal` vs `vertical`) 검증 (30min) *(m-1 해소: research.md §2 전제의 외부 검증)*.

### 11.2 내부 의존성

- `crates/moai-studio-terminal` (SPEC-V3-002 완료) — 공개 API 무수정 재사용.
- `crates/moai-studio-ui` (SPEC-V3-001/002 산출) — `RootView` / `content_area` / `main_body` 수정 대상.
- `crates/moai-studio-workspace` (SPEC-V3-001 완료) — `Workspace` 구조체 무변경. Persistence 는 별도 파일로 분리.

### 11.3 시스템/도구 제약

- **Rust stable 1.93+** (SPEC-V3-002 와 동일).
- **macOS 14+ / Ubuntu 22.04+** — Windows 는 본 SPEC 범위 밖.
- 기존 `mlugg/setup-zig@v2` CI 스텝 유지 (Terminal Core 링크 시 필요).
- MS-1 → MS-2 → MS-3 milestone 전이 시 regression CI gate: 이전 milestone 의 AC 전체가 새 milestone commit 에서 통과해야 함 *(A-2 반영, plan 단계에서 CI yaml 정의)*.

### 11.4 Git / Branch 제약

- 본 SPEC 구현은 `feat/v3-scaffold` 브랜치 또는 이로부터 파생된 milestone 별 서브 브랜치에서 진행.
- `main` 직접 커밋 금지.
- 각 milestone (MS-1, MS-2, MS-3) 은 독립 PR 또는 독립 커밋 그룹으로 분리.

---

## 12. 위험 및 완화

상세 분석은 `.moai/specs/SPEC-V3-003/research.md` §4 참조. 본 섹션은 spec 독자를 위한 요약만 제공한다.

| ID | 위험 | 영향 | 완화 전략 | research 참조 |
|----|------|------|-----------|---------------|
| R-1 | GPUI 0.2.2 의 divider drag API 미확인 | 구현 불가 또는 자체 구현 부담 | plan 단계 spike 1 시간 + gpui-component 평가 1 시간 | research §4.1, §5.1, §5.3 |
| R-2 | 다중 pane 으로 인한 FD 압박 / 메모리 spike | 16 pane 기준 ~960 MB RSS, FD 32 개 | pane 당 60 MB/10K rows 설계 고정, scrollback 상한 설정 노출은 별도 SPEC | research §4.2, §4.3 |
| R-3 | SPEC-V3-002 API 변경 유혹 | 74 tests regression | RG-P-7 로 무변경 강제, CI gate (AC-P-16) | research §4.4 |
| R-4 | GPUI FocusHandle 과 active pane 관리 괴리 | 키 입력이 엉뚱한 pane 에 전달 | Zed 의 `FocusHandle + last_focus_handle_by_item` 패턴 차용 | research §4.5 |
| R-5 | 탭 바 디자인 토큰 부재 | 일관성 없는 UI | 최소 스펙 (bold active + background color) 을 REQ-P-044 에 직접 규정, 정확한 색상 값은 plan 단계에서 Toolbar 토큰에 추가 | research §4.6 |
| R-6 | Persistence schema 와 기존 workspaces.json 역호환 | 기존 workspace 설정 깨짐 | 별도 파일 (`panes-{ws-id}.json`) + `"moai-studio/panes-v1"` schema 분리 | research §4.7 |
| R-7 | gpui-component 의존 도입 시 유지비 | 버전 호환 리스크 | plan 단계 spike 미통과 시 자체 구현 fallback 경로 유지 | research §4.1, §5.3 |
| R-8 | Linux modifier 관례 (Ctrl vs Super) 의 사용자 기대 충돌 | 일부 GNOME / KDE 사용자가 Super 키 기대 | 본 SPEC 은 Ctrl 기반으로 고정 (design 원천 `.moai/design/v3/spec.md:422-438` 와 일치). Super 키 커스터마이징은 Phase 5 Shortcut Customization SPEC 으로 연기. | 신규 (C-2 대응) |
| R-9 | Linux Ctrl+D / Ctrl+W / Ctrl+\\ 의 shell 관례 (EOF / readline word-delete / SIGQUIT) 와 host 바인딩 충돌 | pane 내부 shell 사용자가 `exit` / 단어 삭제 / SIGQUIT 기능을 잃음 | 본 SPEC 에서는 디자인 원천 유지 (기본 활성). Plan **Spike 4** 에서 실제 Linux 셸 세션 UX 를 측정하여 (a) 현행 유지 + 설정 파일 override 제공, (b) Ctrl+Shift+... 로 이중 modifier shift 중 택1. 최종 결정 전까지 Linux 빌드는 host 바인딩 우선. | 신규 (v1.0.0 Nm-3 대응) |

---

## 13. 참조 문서

### 13.1 본 레포 내

- `.moai/specs/SPEC-V3-003/research.md` — 경쟁 분석, 코드베이스 경계, 기술 후보, 위험 상세, 사용자 확정 scope 질문 원본.
- `.moai/specs/SPEC-V3-001/spec.md` + `progress.md` — scaffold 전제 및 Phase 3 후보 명시.
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 공개 API, Exclusions #1 "Tab UI / Pane split — SPEC-V3-003 예정".
- `.moai/design/v3/spec.md:180-206` — Tier A Terminal Core 기능 A-1 "멀티 pane 터미널 (binary tree split)" Critical.
- `.moai/design/v3/system.md:240-250` — 3-Pane Body NSSplitView binary tree 언급.
- `.moai/design/v3/spec.md:420-438` — **플랫폼별 키 바인딩 (macOS | Windows/Linux 이원 표)**. 본 SPEC 의 RG-P-4 의 원천. *(C-1 해소: iter 1 의 `system.md:422-439` 오인용 교정.)*
- `crates/moai-studio-ui/src/lib.rs:69-76` — RootView 의 단일 `terminal: Option<Entity<TerminalSurface>>` 필드.
- `crates/moai-studio-ui/src/lib.rs:410-444` — `content_area` 현재 분기 (단일 terminal vs empty state).
- `crates/moai-studio-ui/src/terminal/mod.rs:101-115` — `TerminalSurface` 구조 (pane leaf content 로 재활용).
- `crates/moai-studio-terminal/src/worker.rs:114-174` — `PtyWorker` (pane 당 독립 spawn 대상).
- `crates/moai-studio-workspace/src/lib.rs:89-91` — `"moai-studio/workspace-v1"` schema 버전.

### 13.2 외부 참조

- [Zed pane.rs](https://github.com/zed-industries/zed/blob/main/crates/workspace/src/pane.rs) — ItemHandle / PaneGroup / FocusHandle 패턴.
- [WezTerm SplitPane](https://wezterm.org/config/lua/keyassignment/SplitPane.html) — binary tree + direction 4 종.
- [cmux](https://cmux.com) — `.moai/design/v3/research.md:9-31` 의 직접 참조 제품.
- [iTerm2 Preferences Keys](https://iterm2.com/documentation-preferences-keys.html) — macOS 사용자 muscle memory.
- [longbridge/gpui-component](https://github.com/longbridge/gpui-component) — Dock + Resizable Panel 후보 (plan spike 대상).

---

## 14. Exclusions (What NOT to Build in this SPEC)

본 섹션은 §3.2 Non-Goals 의 간결 참조와 §3.2 에 기재되지 않은 SPEC 고유 exclusion 만 기재한다 *(M-9 해소: iter 1 의 10항목 전량 중복을 제거. §3.2 는 product-level non-goal, §14 는 SPEC 구현 경계)*.

- §3.2 Non-Goals (N1 ~ N10) 전체 항목은 본 SPEC 의 exclusion 으로 재확인됨. 본 섹션에서 개별 재나열하지 않음.
- **추가 SPEC 고유 exclusion (§3.2 에 없음)**:
  11. **Command Palette 로부터의 pane/tab 조작** — Phase 5 Command Palette SPEC 에서 정의.
  12. **탭 / pane 액션의 사용자 정의 단축키** — Phase 5 Shortcut Customization SPEC 에서 정의. Linux Super 키 커스터마이징 포함 (R-8).
  13. **SPEC 범위의 gpui-component 의존 확정** — plan 단계에서 spike 후 결정.

---

## 15. 용어 정의

- **Pane**: 하나의 `TerminalSurface` (또는 향후 다른 Surface) 가 점유하는 사각형 영역. `PaneTree::Leaf` 또는 하위 구조를 가진 `PaneTree::Split` 일 수 있다.
- **Tab**: 1 개의 `PaneTree` 와 1:1 매핑되는 컨테이너. 사용자가 탭 바에서 인식하는 단위.
- **Split**: 한 pane 을 두 개의 child pane 으로 나누는 연산. 방향은 `Horizontal` 또는 `Vertical`.
- **Horizontal split**: `SplitDirection::Horizontal`. 두 child 가 **좌/우** 로 배치됨. Divider 는 **수직선**. `first` = 왼쪽, `second` = 오른쪽.
- **Vertical split**: `SplitDirection::Vertical`. 두 child 가 **상/하** 로 배치됨. Divider 는 **수평선**. `first` = 위쪽, `second` = 아래쪽.
- **Leaf**: 더 이상 분할되지 않은 `PaneTree::Leaf`, 1 개의 `TerminalSurface` 엔티티를 보유.
- **Focus**: 키 입력을 받는 단 하나의 pane. Workspace 당 1 개만 존재.
- **PaneId / TabId**: UI 계층이 부여하는 식별자. 문자열 형식 (예: `"pane-0001"`, `"tab-0001"`). Terminal Core 는 이를 알지 못한다.
- **Ratio**: Split 노드에서 `first` (왼쪽 또는 위쪽) child 가 차지하는 비율 (`0.0 < ratio < 1.0`, 경계 제외). 최소 pane 크기 제약으로 clamp.
- **In-order iteration**: PaneTree 순회 순서. Split 노드에서 `first` → `second` 순서로 재귀, focus cycle (prev/next pane 단축키) 에 사용.
- **ResizableDivider**: 두 sibling pane 사이의 drag 가능한 경계선. Trait 추상화로 구현체 선택을 plan 까지 연기.
- **last_focused_pane**: 각 탭이 유지하는 직전 focus pane ID. 탭 전환 시 복원.
- **Persistence file**: `~/.moai/studio/panes-{ws-id}.json`. Workspace 별로 분리되며 `"moai-studio/panes-v1"` schema version 명시.
- **Atomic write**: 임시 파일에 쓴 후 rename 하는 저장 패턴. 비정상 종료 시 부분 파일이 생성되지 않음을 보장.
- **Platform-local modifier**: macOS 에서는 Command (⌘), Linux 에서는 Control (Ctrl). 본 SPEC 의 키 바인딩은 이 modifier 를 기본으로 사용하여 플랫폼 네이티브 앱 관례에 부합한다.

---

## 16. 열린 결정 사항 (Open Decisions — plan 단계로 연기)

본 섹션은 SPEC 단계에서 확정하지 않고 plan 단계 spike / 결정에 위임하는 항목을 명시한다. 이 항목들은 본 SPEC 의 AC 가 추상 trait 기반이므로 결정 결과와 무관하게 AC 는 유효하다.

1. **gpui-component 도입 여부** (§11.1 C-1, Exclusion #13). Plan spike 1~2 로 결정.
2. **iTerm2 Cmd+D 의 공식 horizontal / vertical 명칭** (m-1). Plan spike 3 (30분) 으로 research.md §2 전제를 외부 검증.
3. **Toolbar design token `toolbar.tab.active.background` 의 정확한 색상 값** (REQ-P-044, R-5). Plan 단계에서 `.moai/design/v3/system.md` Toolbar 토큰 추가와 함께 확정.
4. **PaneId / TabId 생성 방식** (A-3). `uuid` crate 채택 vs 자체 `format!("pane-{:04x}", counter)` 패턴. 기존 workspace ID 생성 로직과 정합 확인 후 plan 에서 결정.
5. **MX 태그 적용 지점** (A-1). PaneTree split API, TabContainer 의 탭 전환 API, persistence load API 등 fan_in 증가 예상 지점에 `@MX:ANCHOR` 부착. 구체 위치는 Run 단계 RED/ANALYZE phase 에서 확정.
6. **Milestone 간 CI regression gate yaml 정의** (A-2). Plan 단계에서 `.github/workflows/` 에 추가. 본 SPEC 에서는 §11.3 에 요구만 명시.
7. **Linux Super 키 커스터마이징** (R-8). 본 SPEC 에서는 Ctrl 고정, 커스터마이징은 Exclusion #12 에 따라 Phase 5 SPEC 으로 연기.
8. **Linux Ctrl+D / Ctrl+W / Ctrl+\\ shell 관례와의 충돌 해소 방식** (v1.0.0 Nm-3, R-9, §6.4). Plan **Spike 4** 에서 (a) 현행 유지 + 사용자 설정 override 경로 vs (b) Ctrl+Shift+... 이중 modifier shift 경로 선택. Spike 결과에 따라 RG-P-4 Linux 컬럼 + AC-P-9b 가 plan 단계에서 동기화된다.

---

Version: 1.0.0 · 2026-04-24 · approved (iter 2 CONDITIONAL PASS + annotation cycle)
