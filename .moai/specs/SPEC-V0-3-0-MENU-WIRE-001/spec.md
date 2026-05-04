# SPEC-V0-3-0-MENU-WIRE-001 — Menu Stub Functional Wire (3 actions)

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-MENU-WIRE-001 |
| **Title** | View menu 잔존 stub functional wiring (ToggleSidebar / ToggleBanner / ReloadWorkspace) |
| **Status** | merged (PR #100, 2026-05-04 main 859e6fe) — MS-1 closure (3 stub functional). 5 stub carry to Surface SPEC + Pane SPEC. |
| **Priority** | Medium |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V0-1-2-MENUS-001 (carry §3.1) |
| **Cycle** | v0.3.0 (audit Top 16 #14 / F-2 amendment 대체) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안 작성. v0.3.0 cycle Sprint 1 #2 진입. SPEC-V0-1-2-MENUS-001 §3.1 의 carry 명시 ("Remaining stubs ... stay deferred to follow-up SPECs") 에 따라 신규 SPEC 분기. AC 한도 (≤8) 및 milestones (≤2) 모두 lightweight 충족.

## 1. Purpose

V0-1-2-MENUS-001 MS-2 (#68) 가 4 stub (`OpenCommandPalette` / `OpenSpecPanel` / `SplitRight` / `SplitDown`) 만 functional 처리하고 잔존 8 stub 을 follow-up SPECs 로 carry 했다. 본 SPEC 은 그 중 **자가완결적 3 stub** (`ToggleSidebar` / `ToggleBanner` / `ReloadWorkspace`) 만 functional 로 변환한다. 나머지 5 stub (Surface×3, ClosePane, FocusNext/Prev) 은 `dispatch_command` 의 `surface.*` / `pane.*` namespace 가 stub 인 의존성이라 별도 SPEC 으로 carry.

## 2. Goals

- View menu 의 ToggleSidebar 액션이 사이드바 visibility 를 실제 토글
- View menu 의 ToggleBanner 액션이 banner strip visibility 를 실제 토글
- View menu 의 ReloadWorkspace 액션이 storage 로부터 workspace 목록을 다시 로드
- 잔존 5 stub (NewTerminalSurface / NewMarkdownSurface / NewCodeViewerSurface / ClosePane / FocusNext/PrevPane) 은 차후 SPEC carry 명시 보존
- TRUST 5 gates (clippy / fmt / unit test) ALL PASS

## 3. Non-Goals / Exclusions

- 5 잔존 stub (Surface×3, Pane×3) functional 변환 (별 SPEC carry)
- `dispatch_command` 의 `surface.*` / `pane.*` namespace 활성화 (Surface SPEC / Pane SPEC 의존)
- 사이드바 width 변경 또는 collapsible 애니메이션 (단순 display gate 만)
- BannerStack entity 자체 mutation (RootView 단 visibility flag 만)
- 신규 키바인딩 (Cmd+B 는 이미 V0-1-2-MENUS-001 에서 wire 됨)

## 4. Requirements

- REQ-MW-001: RootView 는 `sidebar_visible: bool` 필드를 가진다 (default `true`).
- REQ-MW-002: RootView 는 `banner_visible: bool` 필드를 가진다 (default `true`).
- REQ-MW-003: ToggleSidebar action handler 는 `sidebar_visible` 을 토글하고 `cx.notify()` 를 호출한다.
- REQ-MW-004: ToggleBanner action handler 는 `banner_visible` 을 토글하고 `cx.notify()` 를 호출한다.
- REQ-MW-005: ReloadWorkspace action handler 는 `WorkspacesStore::load(self.storage_path)` 를 호출하여 결과의 `entries()` 로 `self.workspaces` 를 갱신한다.
- REQ-MW-006: ReloadWorkspace 후 `self.active_id` 가 reload 된 workspaces 에 여전히 존재하면 보존, 부재하면 가장 최근 last_active 워크스페이스로 재설정.
- REQ-MW-007: `main_body` 는 `sidebar_visible` 인자를 받아 `false` 일 때 sidebar element 를 렌더하지 않는다.
- REQ-MW-008: `Render::render` 에서 banner strip child 는 `banner_visible` 가 true 일 때만 추가된다.

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-MW-1 | RootView 신규 인스턴스 | 생성 직후 | `sidebar_visible == true && banner_visible == true` | unit test (`root_view_default_sidebar_and_banner_visible`) |
| AC-MW-2 | RootView 의 sidebar_visible=true | `toggle_sidebar_visible()` 호출 | `sidebar_visible == false` 로 변경됨 | unit test (`toggle_sidebar_visible_flips_state`) |
| AC-MW-3 | RootView 의 banner_visible=true | `toggle_banner_visible()` 호출 | `banner_visible == false` 로 변경됨 | unit test (`toggle_banner_visible_flips_state`) |
| AC-MW-4 | RootView 의 storage_path 가 유효한 workspaces.json 가리킴 | `reload_workspaces_from_storage()` 호출 | `workspaces` 가 storage 의 최신 내용으로 갱신 | unit test (`reload_workspaces_from_storage_replaces_in_memory_list`) |
| AC-MW-5 | reload 후 기존 active_id 가 새 목록에 존재 | `reload_workspaces_from_storage()` | `active_id` 가 그대로 보존 | unit test (`reload_preserves_active_id_when_present_after_reload`) |
| AC-MW-6 | reload 후 기존 active_id 가 새 목록에 부재 | `reload_workspaces_from_storage()` | `active_id` 가 가장 최근 last_active 워크스페이스로 재설정, 또는 빈 목록이면 None | unit test (`reload_resets_active_id_when_absent`) |
| AC-MW-7 | main_body 시그니처 sidebar_visible=false | render 호출 (logic-level) | sidebar element 미생성 (helper boolean 으로 검증) | unit test (`main_body_skips_sidebar_when_invisible`) |
| AC-MW-8 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 회귀 0 | CI |

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | RootView 필드 2개 추가, 3 stub handler functional, main_body 시그니처 + Render banner 분기, 신규 helper 메서드 + tests |
| `.moai/specs/SPEC-V0-3-0-MENU-WIRE-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-MENU-WIRE-001/progress.md` | created | 구현 후 갱신 |

FROZEN (touch 금지):
- `crates/moai-studio-terminal/**` — terminal crate 무관
- `crates/moai-studio-workspace/src/lib.rs` 의 `WorkspacesStore::load` 시그니처 — 호출만, 구현 무수정
- SPEC-V3-004 / V3-005 / V3-014 의 진행 중 SPEC

## 7. Test Strategy

ui crate `lib.rs::tests` 모듈에 7 신규 unit test 추가 (T-MW 블록).

- AC-MW-1~3: RootView state 토글 logic
- AC-MW-4~6: WorkspacesStore reload 시나리오 (tempfile 사용)
- AC-MW-7: main_body 분기는 helper boolean (`should_render_sidebar`) 또는 직접 분기 검증

GPUI 미가동 (cx 의존 X) — 모든 helper 는 cx 무관 메서드로 분리.

회귀 검증: 기존 ui crate 1312 tests 무영향 (additive only).

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 1
Carry-from: SPEC-V0-1-2-MENUS-001 §3.1
Carry-to: 차후 Surface SPEC + Pane SPEC (잔존 5 stub)
