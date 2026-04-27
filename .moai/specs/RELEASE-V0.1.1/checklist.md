---
id: RELEASE-V0.1.1
version: 1.0.0
status: in-progress
created: 2026-04-27
updated: 2026-04-27
classification: hotfix-checklist
parent: RELEASE-V0.1.0
---

# RELEASE v0.1.1 — UX Audit + Critical Fix Checklist

v0.1.0 publish (2026-04-27 sess 7) 직후 사용자 시연으로 발견된 critical UX 결함의 hotfix. v0.1.0 은 release notes 에 "WITHDRAWN" 표시, v0.1.1 이 정식 사용 가능 첫 release.

## 0. Audit Scope

| 검증 차원 | 대상 |
|----------|------|
| Functional | RefCell panic, dead button, TabContainer auto-init |
| Visual | Traffic light 중복, active state 표시, alignment |
| UX flow | Workspace 생성 → 사용 가능 상태까지의 1st-run UX |
| Design tokens | `.moai/design/tokens.json` v2.0.0 사용 일관성 |
| Accessibility | hover cursor, contrast, keyboard navigation |

---

## 1. Critical Bugs (release blocker tier)

| ID | Issue | 발견 | Fix scope |
|----|-------|------|-----------|
| C-1 | RefCell panic on `+ New Workspace` click — rfd::FileDialog blocking modal vs GPUI tick 재진입 | sess 7 | ✅ v0.1.1 (cx.prompt_for_paths + cx.spawn) |
| C-2 | `+ Create First Workspace` button dead (no on_mouse_down handler) | sess 7 | ✅ v0.1.1 (cx.listener wire) |
| C-3 | **TabContainer Entity production 코드 부재** — workspace 활성화 후 영구 "tab container initializing" stuck | sess 7 | 🟡 v0.1.1 즉시 fix |
| C-4 | **Custom traffic_lights() + macOS native traffic light 동시** → 좌상단 6개 dot 중복 | sess 7 | 🟡 v0.1.1 즉시 fix |

## 2. High Priority Bugs (UX 정상 사용 차단)

| ID | Issue | 발견 | Fix scope |
|----|-------|------|-----------|
| H-1 | 사이드바 workspace row 의 active dot (●) 이 모든 row 에 표시 — `traffic::RED` 토큰 오용 | sess 7 image | 🟡 v0.1.1 (token 분리 + is_active 검증) |
| H-2 | "Start Sample" button — handler 없음, 동작 미정의 | sess 7 | ⏸️ v0.1.2 또는 비활성 표시 (v0.1.1 disable + tooltip) |
| H-3 | "Open Recent" button — handler 없음, 마지막 active workspace open 의도 추정 | sess 7 | ⏸️ v0.1.2 또는 비활성 표시 |
| H-4 | "Tip: ⌘K opens Command Palette anytime" — V3-012 palette 가 구현되어 있는지 wire 검증 필요 | sess 7 | 🟡 v0.1.1 (false promise 면 텍스트 변경) |

## 3. Medium Priority (기능 기대치)

| ID | Issue | Fix scope |
|----|-------|-----------|
| M-1 | Status bar `no git` — workspace 가 git repo 일 때도 갱신 안 됨 | ⏸️ v0.1.2 SPEC plan |
| M-2 | Status bar `⌘K to search` — palette wire 검증 (H-4 와 동일) | 🟡 v0.1.1 |
| M-3 | Sidebar `GIT WORKTREES`, `SPECS` 섹션 `—` placeholder — 실제 데이터 binding 부재 | ⏸️ v0.1.2+ SPEC (V3-009 통합) |
| M-4 | Workspace 이름 long string 시 TitleBar/sidebar truncate 안 됨 | ⏸️ v0.1.2 polish |

## 4. Functional Gap (별 SPEC 후보)

| ID | Issue | 처리 |
|----|-------|-----|
| F-1 | TabContainer 자동 생성 + default leaf payload (placeholder String 또는 TerminalSurface) | C-3 fix 의 일부 |
| F-2 | PTY worker spawn per-pane — V3-002 Terminal Core 와 통합 | ⏸️ V3-005 또는 별 SPEC |
| F-3 | Explorer click → viewer mount — V3-005/006 e2e | 이미 구현 (#16) — 동작 검증만 |
| F-4 | Settings modal (Cmd+,) — V3-013 wire 검증 | 🟡 v0.1.1 verify |
| F-5 | Command palette (Cmd+K / Cmd+Shift+P) — V3-012 wire 검증 | 🟡 v0.1.1 verify |
| F-6 | Spec panel (Cmd+L 등) — V3-015 wire 검증 | 🟡 v0.1.1 verify |
| F-7 | Banner stack — crash/update/lsp 알림 — V3-014 동작 시나리오 검증 | ⏸️ v0.1.2 |

## 5. Design Tokens Audit (`.moai/design/tokens.json` v2.0.0)

| ID | 검토 항목 | 상태 |
|----|----------|------|
| T-1 | `traffic::RED/YELLOW/GREEN` 의 의도 — macOS traffic light only, sidebar active dot 등 다른 의미 사용 금지 | 🟡 H-1 fix 의 일부 |
| T-2 | `tok::ACCENT` (PRIMARY_DARK 청록 #22938a) — primary CTA + active state 일관 사용 | ✅ 일관됨 |
| T-3 | `tok::FG_MUTED` (#9a9aa0) vs `BG_APP` 대비 — WCAG AA 4.5:1 검증 | ⏸️ v0.1.2 |
| T-4 | `BG_SURFACE` / `BG_ELEVATED` / `BG_PANEL` 의 상대적 명도 차 — sidebar/content/modal 깊이감 | ⏸️ v0.1.2 |
| T-5 | Hover state token 부재 — 현재 모든 button 이 static. hover 변화 없음 | ⏸️ v0.1.2 token 추가 |
| T-6 | Border radius / spacing 의 일관성 (px_3, px_4, px_6 의 사용 패턴) | ⏸️ v0.1.2 audit |

## 6. Accessibility / Keyboard

| ID | 검토 항목 | 상태 |
|----|----------|------|
| A-1 | Tab navigation — 모든 interactive element 가 keyboard 로 도달 가능한가 | ⏸️ v0.1.2 |
| A-2 | hover cursor `pointer` 가 모든 click 가능 element 에 부여 | 🟡 v0.1.1 (empty_state_primary_cta 만 추가됨, 다른 element 추가 필요) |
| A-3 | Focus ring visible — 현재 GPUI 0.2.2 의 focus indicator 부재 | ⏸️ v0.1.2 token |
| A-4 | ARIA-equivalent semantic role (GPUI 의 role 시스템) | ⏸️ v0.2.0 |

## 7. v0.1.1 즉시 Fix 우선순위

본 세션 fix:
1. **C-3** TabContainer 자동 생성 (apply_added_workspace + handle_activate_workspace)
2. **C-4** Traffic light 중복 제거 (custom traffic_lights() 제거, native 만 사용)
3. **H-1** Active dot token 분리 (`traffic::RED` → 별 `ACTIVE_DOT` 또는 `ACCENT` 사용)
4. **H-4 / M-2 / F-4 / F-5 / F-6** — palette / settings / spec panel 키바인딩 wire 검증 (간단 manual test)

다음 세션 (v0.1.2 또는 v0.2.0):
- H-2 / H-3 (Start Sample / Open Recent)
- M-1 / M-3 (status bar git, 사이드바 SPECS binding)
- T-3 ~ T-6 (디자인 토큰 audit)
- A-1 / A-3 (a11y)
- F-2 / F-7 (feature 추가)

## 8. v0.1.1 Verification

각 fix 적용 후 검증 절차:

| 검증 단계 | 방법 |
|----------|------|
| cargo fmt --check | 자동 |
| cargo clippy --workspace --all-targets -- -D warnings | 자동 |
| cargo test --workspace --all-targets | 자동 |
| .app bundle 재빌드 + dylib bundling + codesign | 메인 세션 자동 |
| 수동 시연 (사용자) | "+ Create First Workspace" 클릭 → 폴더 선택 → workspace 생성 → TabContainer 가시 → tab 새 생성 (Cmd+T) → 분할 (Cmd+\) → divider drag |
| 사이드바 active state | 두 workspace 중 active 만 highlight |
| Traffic light 단일 row | macOS native 만 visible |

---

Version: 1.0.0
Branch: hotfix/v0.1.1-rfd-modal-borrow (기존 hotfix 에 UX fix 추가 → branch 이름 그대로 유지하되 PR 본문에서 scope 확장 명시)
