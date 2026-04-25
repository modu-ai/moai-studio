---
id: RELEASE-V0.1.0
version: 1.0.0
status: draft
created: 2026-04-25
updated: 2026-04-25
author: GOOS행님
priority: High
classification: release-plan
---

# RELEASE v0.1.0 — moai-studio 첫 정식 릴리스 플랜

## HISTORY

- 2026-04-25 (v1.0.0): 최초 작성. SPEC-V3-001/002/003 GREEN, SPEC-V3-004 implementation in progress 시점의 릴리스 플랜 확정. Enhanced GitHub Flow (CLAUDE.local.md §1) 절차 명시. Branch protection rule 활성화 트리거로 v0.1.0 tag 부착 정의.

---

## 0. Vision

moai-studio v0.1.0 은 본 레포의 **첫 정식 릴리스 (first stable tag)** 다. 본 릴리스는 두 가지 운영 의의를 갖는다.

1. **CLAUDE.local.md §8 임시 규칙 종료 트리거** — pre-release 단계 (v0.0.x) 의 임시 운영 규칙을 정식 운영 규칙으로 승격.
2. **Branch protection rule 활성화 트리거** — `main` / `release/*` / `develop` 보호 규칙이 모두 활성화된 상태로 진입.

릴리스 범위는 **V3 아키텍처 (GPUI 기반 multi-pane terminal shell) 의 minimum-viable surface** 로 한정한다. SPEC-V3-005 ~ 011 같은 후속 SPEC 은 **v0.1.0 backlog 에서 제외** 되며 v0.1.x patch 또는 v0.2.0 minor cycle 로 이월한다.

---

## 1. 릴리스 범위 (Release Scope)

### 1.1 v0.1.0 GA-In SPECs

| SPEC ID | 제목 | 상태 (2026-04-25) | 게이트 조건 |
|---------|------|-------------------|------------|
| SPEC-V3-001 | GPUI scaffold + RootView 진입 | GREEN (merged to develop) | 추가 변경 없음. 회귀 테스트만 유지. |
| SPEC-V3-002 | Terminal Core (PTY + alacritty backend) | GREEN (merged to develop) | 추가 변경 없음. 회귀 테스트만 유지. |
| SPEC-V3-003 | Pane + Tab + Persistence (panes-v1 schema) | GREEN (merged to develop, MS-3 완료) | 추가 변경 없음. ci-v3-pane.yml regression gate 유지. |
| SPEC-V3-004 | Render Layer Integration — TabContainer Entity + PaneTree GPUI rendering + divider drag e2e | **In Progress (current branch: `feature/SPEC-V3-004-render`)** | **MS-1 GREEN 필수**. MS-2 / MS-3 carry-over 허용. 자세한 분기 정책 §1.3 참조. |

[HARD] **GA-In 의미**: 본 SPEC 의 minimum acceptance criteria 가 v0.1.0 tag 시점에 GREEN 상태여야 한다. failed regression 또는 incomplete milestone 은 release blocker 로 처리.

### 1.2 v0.1.0 GA-Out SPECs (Backlog → v0.1.x / v0.2.0)

| SPEC ID | 제목 | 이월 사유 | 잠정 target |
|---------|------|----------|-------------|
| SPEC-V3-005 | File Explorer | UI surface 미완. v0.1.0 scope 외. | v0.2.0 |
| SPEC-V3-006 | Markdown / Code Viewer | v0.1.0 minimum surface 외. | v0.2.0 |
| SPEC-V3-008 | (placeholder) | 본 릴리스에서 제외. | v0.2.0+ |
| SPEC-V3-009 | SPEC Management UI | 본 릴리스에서 제외. | v0.2.0+ |
| SPEC-V3-007 / 010 / 011 | (TBD: cargo packaging, distribution, etc.) | 본 릴리스에서 제외. SPEC-V3-011 (packaging) 은 v0.1.0 post-release 에 distributable artifacts 생성 시점에 별도 활성화. | v0.1.x patch 또는 v0.2.0 |

[HARD] **GA-Out 의미**: 본 SPEC 들은 v0.1.0 release notes 의 "Known Limitations" 섹션에 명시되며, 사용자에게 "v0.2.0 backlog" 임을 공지한다.

### 1.3 SPEC-V3-004 분기 정책 (Critical Gate)

SPEC-V3-004 는 v0.1.0 의 **최후 GA-In SPEC** 이며 현재 implementation in progress 상태다. 다음 분기 정책을 적용한다.

#### Path A — MS-1 + MS-2 + MS-3 모두 GREEN (권장 시나리오)

- 결정 시점: SPEC-V3-004 작업 종료 시점 (사용자 판단)
- 조치: SPEC-V3-004 전체를 v0.1.0 에 포함. release notes 에 "render layer integration complete" 명기.

#### Path B — MS-1 만 GREEN, MS-2 / MS-3 carry-over (fallback 시나리오)

- 트리거 조건: MS-1 (TabContainer Entity 골격) 은 GREEN 이지만, MS-2 (PaneTree render + key dispatch) 또는 MS-3 (divider drag e2e) 가 timeline 내 미완성.
- 조치:
  - MS-1 에 한해 v0.1.0 에 포함 (TabContainer Entity skeleton + RootView wiring).
  - MS-2 / MS-3 는 v0.1.1 patch 로 이월. SPEC-V3-004 spec.md 에 carry-over 표시 (HISTORY 섹션 업데이트).
  - 새 SPEC ID 를 만들지 않는다 — 기존 SPEC-V3-004 의 milestone 만 분할.
- release notes 명기: "Render layer integration partial (TabContainer Entity 도입, divider drag e2e 는 v0.1.1 carry-over)"

#### Path C — MS-1 미완성 (release blocker)

- 트리거 조건: MS-1 GREEN 미달성.
- 조치: **v0.1.0 release 보류**. SPEC-V3-004 MS-1 GREEN 이전까지 `release/v0.1.0` 분기 금지.
- 사용자 결정 게이트: blocker 사유 진단 후 (a) MS-1 완료 대기, (b) SPEC-V3-004 전체를 v0.1.0 에서 제외하고 SPEC-V3-003 까지의 surface 만으로 release, (c) v0.1.0 자체를 v0.0.4 등 추가 pre-release 로 격하 — 셋 중 택일.

[HARD] **default**: Path A 시도 → 불가 시 Path B 적용. Path C 는 사용자 명시적 승인 필요.

---

## 2. Pre-release Checklist (Release Gate 진입 조건)

`release/v0.1.0` 분기를 만들기 **이전에** 모든 항목이 GREEN 이어야 한다.

### 2.1 외부 의존성 (현 차단 항목)

- [ ] **GitHub Actions billing 해소** — 현재 본 레포는 Actions billing 차단 상태로 워크플로 실행 불가. v0.1.0 진입 전 결제 수단 정상화 필수.
  - 검증: GitHub `Settings → Billing` 에서 Actions usage 정상 표시.
  - blocker: billing 미해소 시 §2.2 검증 자체 불가능 → release 일정 전체 보류.

### 2.2 CI GREEN 검증 (billing 해소 후 즉시)

- [ ] **`ci-rust.yml` (Rust CI v3) 첫 GREEN run** — `develop` 브랜치 head 기준 매트릭스 (macOS / Linux / Windows × stable / nightly) 전체 PASS.
- [ ] **`ci-v3-pane.yml` (Pane CI — SPEC-V3-003 regression gate) 첫 GREEN run** — `develop` 브랜치 head 기준 PASS.
- [ ] **release-drafter.yml dry-run 검증** — PR 라벨 (type/* + area/* + priority/*) 부착 상태로 PR 머지 시 draft release 가 정상 누적되는지 확인.

### 2.3 로컬 검증 (모든 git push 이전)

- [ ] **`cargo test --workspace --all-targets`** — 0 failures (현 SPEC-V3-001/002/003 회귀 + SPEC-V3-004 진행분 포함).
- [ ] **`cargo clippy --workspace --all-targets -- -D warnings`** — 0 warnings (deny-warnings 모드).
- [ ] **`cargo fmt --all -- --check`** — formatting drift 0.
- [ ] **`cargo build --release --workspace`** — release profile 빌드 성공.

### 2.4 수동 smoke test (release 후보 commit 기준)

- [ ] **macOS smoke test** — `cargo run --release -p moai-studio-app` 실행 후 다음 시나리오:
  - 앱 실행 (panic 없이 RootView 가시).
  - 새 탭 생성 (Cmd+T).
  - Pane split (Cmd+\\, Cmd+Shift+\\) — divider 가시.
  - 탭 간 이동 (Cmd+1 ~ Cmd+9).
  - PTY 입출력 정상 (echo, ls 실행).
  - 앱 종료 후 재실행 → persistence 복원 (panes-{ws-id}.json).
- [ ] **Linux smoke test** — `cargo run --release -p moai-studio-app` 실행 (Cmd → Ctrl 매핑) — 위 시나리오 동일 검증.
- [ ] **Windows smoke test** — best-effort. CI 매트릭스에 포함되지만 manual smoke 는 optional. 미실행 시 release notes 에 "Windows: CI verified, no manual smoke test for v0.1.0" 명기.

### 2.5 Cross-platform binary 빌드 검증

- [ ] **macOS binary** — `cargo build --release --workspace` on macOS-latest runner. artifact 확보.
- [ ] **Linux binary** — `cargo build --release --workspace` on ubuntu-latest runner. artifact 확보.
- [ ] **Windows binary** — `cargo build --release --workspace` on windows-latest runner. CI GREEN 으로 갈음 가능. distributable 은 SPEC-V3-011 packaging 도입 후 별도 처리 (§4.4 참조).

[HARD] §2.1 ~ §2.5 의 모든 체크박스가 체크되기 전 §3 (Release Procedure) 진입 금지.

---

## 3. Release Procedure (Enhanced GitHub Flow §1 / §6.2 준수)

CLAUDE.local.md §1 (Branch Model) 및 §6.2 (Release 준비 체크리스트) 를 그대로 따른다. 본 절은 v0.1.0 에 특화한 구체 절차를 명시한다.

### 3.1 Step 1 — `develop` ahead of `main` 검증

- [ ] `git checkout develop && git pull origin develop`
- [ ] `git log --oneline main..develop` — `develop` 에 머지된 PR 목록 확인 (SPEC-V3-001/002/003/004 관련 commit 모두 포함되어 있어야 함).
- [ ] `develop` 의 head commit hash 를 release tag 후보로 기록.

### 3.2 Step 2 — `release/v0.1.0` 분기 (from `develop`)

- [ ] `git checkout develop && git pull origin develop`
- [ ] `git checkout -b release/v0.1.0`
- [ ] `git push -u origin release/v0.1.0`
- [ ] **시점 이후 `develop` 에 머지되는 새 feature 는 v0.1.0 에 포함되지 않음** — 모든 작업자에게 공지.

### 3.3 Step 3 — QA + final bug fix only

`release/v0.1.0` 분기 후 다음만 허용한다.

- [ ] **bug fix only** — feature 추가 금지. SPEC-V3-005/006/008/009 등 backlog SPEC 머지 금지.
- [ ] **bug fix 절차**:
  - `git checkout -b fix/<short-slug> release/v0.1.0` (release 분기 기준)
  - 수정 + reproduction test 작성 (CLAUDE.md §7 Rule 4)
  - PR `fix/<slug>` → `release/v0.1.0` (Squash merge)
  - PR 라벨: `type/bug` + `area/*` + `priority/*` + (해당 시) `release/patch`
- [ ] **§2.3 로컬 검증을 release 후보 commit 마다 재실행** — fix 머지 시마다 `cargo test --workspace --all-targets` GREEN 유지.

### 3.4 Step 4 — Release Drafter draft 검토

- [ ] GitHub `Releases → Drafts` 에서 `v0.1.0` draft 진입.
- [ ] 카테고리별 (Added / Fixed / Security / Performance / Refactored / Documentation / Internal) 항목 검토.
- [ ] 누락된 PR 또는 잘못 분류된 항목이 있으면 PR 라벨 수정 후 draft 재생성 (release-drafter 자동 갱신).
- [ ] **CHANGELOG 초안 final review** — 사용자 승인 후 §3.5 진입.

### 3.5 Step 5 — PR `release/v0.1.0` → `main` (Merge commit + tag)

- [ ] PR 생성: base `main`, compare `release/v0.1.0`.
- [ ] PR 제목: `merge(release): release/v0.1.0 → main [v0.1.0]`
- [ ] PR 본문: Release Drafter draft 본문 그대로 사용.
- [ ] PR 라벨: `release/minor` (v0.0.x → v0.1.0 은 minor bump 로 처리. semantic versioning 상 0.1.0 은 첫 정식 minor 이므로 minor 분류).
- [ ] CI GREEN + 1 review 확보.
- [ ] **Merge commit (`--no-ff`)** 으로 머지. Squash 금지.
- [ ] 머지 직후:
  - [ ] `git checkout main && git pull origin main`
  - [ ] `git tag -a v0.1.0 -m "moai-studio v0.1.0"`
  - [ ] `git push origin v0.1.0`
- [ ] Release Drafter draft → **publish**. CHANGELOG 정식 공개.

[HARD] **tag commit 은 Merge commit 자체** — squash 결과물이 아닌 `--no-ff` merge commit 에 tag 부착.

### 3.6 Step 6 — Back-merge `main` → `develop`

- [ ] PR 생성: base `develop`, compare `main`.
- [ ] PR 제목: `merge(release): main → develop (v0.1.0 back-merge)`
- [ ] PR 본문: "v0.1.0 release 후 back-merge. release/v0.1.0 에 직접 커밋된 bug fix 를 develop 으로 동기화."
- [ ] PR 라벨: `type/chore` + `area/ci` + `skip-changelog`
- [ ] **Merge commit (`--no-ff`)** 으로 머지.

### 3.7 Step 7 — `release/v0.1.0` 브랜치 삭제

- [ ] `git push origin --delete release/v0.1.0`
- [ ] 로컬 `git branch -d release/v0.1.0` (선택).

---

## 4. Post-release Tasks

v0.1.0 tag 부착 + back-merge 완료 직후 실행한다.

### 4.1 CLAUDE.local.md §8 임시 규칙 삭제

- [ ] `develop` 에서 `feature/cleanup-v0.1.0-postrelease` 분기.
- [ ] CLAUDE.local.md 편집:
  - §8 (Version 1차 확정 전 임시 규칙) 섹션 전체 삭제.
  - §1 ~ §7 의 "v0.1.0 까지" 식 임시 표현 제거 (검색: "v0.1.0", "pre-release").
- [ ] commit: `chore(docs): CLAUDE.local.md §8 임시 규칙 삭제 + v0.1.0 정식 운영 전환`
- [ ] PR `feature/cleanup-v0.1.0-postrelease` → `develop` (Squash merge).
- [ ] 라벨: `type/docs` + `area/docs` + `priority/p1-high` + `skip-changelog`.

### 4.2 Branch Protection Rule 활성화 체크리스트

CLAUDE.local.md §2 의 모든 체크박스를 GitHub UI 에서 활성화한다.

- [ ] **§2.1 `main` 브랜치 보호 규칙** (Settings → Branches → Branch protection rules → Add rule):
  - Branch name pattern: `main`
  - [ ] Require a pull request before merging (Approvals: 1, Dismiss stale: on)
  - [ ] Require status checks: `Rust CI (v3)`, `Pane CI (SPEC-V3-003 regression gate)` (필수). Up-to-date: on.
  - [ ] Require linear history: **off**
  - [ ] Include administrators: **on**
  - [ ] Restrict who can push: 관리자 외 차단
  - [ ] Allow force pushes: **off**
  - [ ] Allow deletions: **off**
- [ ] **§2.2 `release/*` 브랜치 보호 규칙**:
  - Branch name pattern: `release/*`
  - [ ] Require a pull request (Approvals: 1)
  - [ ] Require status checks: 위와 동일
  - [ ] Restrict who can push: 릴리스 담당자만
  - [ ] Allow force pushes: **off**
  - [ ] Allow deletions: **on**
- [ ] **§2.3 `develop` 브랜치 보호 규칙**:
  - Branch name pattern: `develop`
  - [ ] Require a pull request (권장)
  - [ ] Require status checks: 위와 동일
  - [ ] Allow force pushes: **off**
  - [ ] Include administrators: **off**
- [ ] CLAUDE.local.md §2 의 "설정 완료 후 체크박스 체크 및 날짜 기록" 항목 업데이트:
  - [ ] main: `2026-MM-DD` 활성화
  - [ ] release/*: `2026-MM-DD` 활성화
  - [ ] develop: `2026-MM-DD` 활성화

### 4.3 Repo Public 전환

memory `repo_visibility` 노트 (project_repo_visibility.md) 의 트리거 조건 충족.

- [ ] GitHub `Settings → General → Danger Zone → Change visibility → Make public` 실행.
- [ ] README.md / LICENSE 최종 검토 (public 노출 안전성).
- [ ] memory `project_repo_visibility.md` 업데이트: PRIVATE → PUBLIC (`2026-MM-DD` 전환 일자 기록).

### 4.4 Distributable Artifacts (SPEC-V3-011 packaging 도입 후)

SPEC-V3-011 (packaging) 은 v0.1.0 GA-In 이 아니므로, v0.1.0 시점에는 source-only release 로 처리한다. 단:

- [ ] SPEC-V3-011 활성화 시점에 v0.1.0 tag 기준 distributable (macOS .dmg, Linux .deb / AppImage, Windows .exe) 생성.
- [ ] GitHub Releases 의 v0.1.0 entry 에 binary asset 첨부 (post-publish edit).
- [ ] release notes 에 "Binaries added on 2026-MM-DD via SPEC-V3-011" 노트 추가.

---

## 5. Rollback Plan

### 5.1 Hotfix 절차 (critical bug 발견 시)

CLAUDE.local.md §6.3 (Hotfix 워크플로) 그대로 적용.

- [ ] `git checkout main && git pull`
- [ ] `git checkout -b hotfix/v0.1.1-{slug}` — `{slug}` 는 2~5 단어 영문 kebab-case (예: `pane-focus-crash`, `pty-fd-leak`).
- [ ] **reproduction test 우선 작성** (CLAUDE.md §7 Rule 4) — 실패 확인 후 수정 진행.
- [ ] 최소 변경 원칙 준수 (CLAUDE.md §7 Rule 5 "Maintain Scope Discipline").
- [ ] PR `hotfix/v0.1.1-{slug}` → `main` (Merge commit + tag `v0.1.1`).
- [ ] Back-merge PR `main` → `develop` (Merge commit, `skip-changelog` 라벨).
- [ ] 활성 `release/*` 분기 있으면 그쪽에도 back-merge.
- [ ] `hotfix/*` 브랜치 삭제.

### 5.2 v0.1.0 tag 직접 패치 금지 [HARD]

- [HARD] **v0.1.0 tag 자체를 force-update 하거나 삭제하지 않는다**. tag 는 immutable 으로 취급.
- [HARD] **rollback 은 항상 hotfix 신 tag (v0.1.1) 로 표현**. tag move 또는 tag delete 금지.
- 예외: tag publish 직후 (≤ 24h) GitHub Actions billing / CI 인프라 결함으로 인해 tag 자체에 결함이 있는 경우만 사용자 명시적 승인 후 tag 재생성 가능. 단 이 경우에도 release notes 에 사실 명기.

### 5.3 catastrophic failure response (release publish 후 즉시 critical bug)

- 트리거 조건: v0.1.0 publish 후 <2h 이내 production-blocking bug 발견 (e.g. 앱 실행 시 즉시 panic, 데이터 손실).
- 조치:
  1. GitHub Releases 의 v0.1.0 entry 를 **draft 로 되돌리고** (unpublish) "Withdrawn due to critical bug" 명기.
  2. tag `v0.1.0` 은 유지 (§5.2 원칙).
  3. hotfix 절차 (§5.1) 즉시 실행, v0.1.1 으로 정식 release 재시도.
  4. 사용자 공지: README / CHANGELOG / Releases 에 v0.1.0 withdrawn 사실 명시.

---

## 6. Risks and Open Questions

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R1 | GitHub Actions billing 미해소 | §2.2 CI 검증 불가 → release 전체 보류 | billing 해소를 release timeline 의 critical path 로 관리. 미해소 시 §3.2 진입 금지. |
| R2 | SPEC-V3-004 MS-1 미완성 | Path C 발동 → release 보류 또는 scope 축소 | Path B fallback 확보 (MS-2/MS-3 carry-over). MS-1 GREEN 을 hard prerequisite 으로 명시. |
| R3 | Windows manual smoke test 미수행 | Windows 사용자 첫 인상 결함 가능 | release notes 에 "Windows: CI verified only" 명기. v0.1.1 에서 manual smoke test 수행 권장. |
| R4 | Release Drafter 라벨 누락 PR | CHANGELOG 초안 부정확 | §3.4 에서 final review 강제. 라벨 미부착 PR 은 review 단계에서 reject (CLAUDE.local.md §9 트러블슈팅 규칙). |
| R5 | back-merge 누락 → develop regression | hotfix 시 동일 bug 재출현 | §3.6 back-merge 단계 체크리스트화. hotfix 시 §5.1 마지막 단계 (back-merge) 누락 금지. |

### Open Questions

- **OQ1**: SPEC-V3-004 MS-2 / MS-3 가 Path B 로 carry-over 되는 경우, v0.1.1 patch release 의 timeline 은? — release 시점에 사용자 결정 필요.
- **OQ2**: SPEC-V3-011 packaging 도입 시점 — v0.1.x 중 어디? 또는 v0.2.0? — packaging SPEC 작성 시점에 결정.

---

## 7. Exclusions (이 plan 이 다루지 않는 것)

- **개별 SPEC 의 내부 구현 디테일** — 본 plan 은 release 절차에 한정. SPEC-V3-001 ~ 004 의 acceptance criteria 는 각 SPEC 의 spec.md 참조.
- **v0.2.0 이후 roadmap** — backlog 명시는 §1.2 까지. v0.2.0 milestone 정의는 별도 문서.
- **CI workflow 자체 수정** — `ci-rust.yml` / `ci-v3-pane.yml` / `release-drafter.yml` 의 신규 변경은 본 release 범위 외. 기존 workflow GREEN 검증만 다룸.
- **distribution channel 선택** (Homebrew / apt / Microsoft Store 등) — SPEC-V3-011 packaging 범위.
- **마케팅 / 공지 / blog post** — 본 plan 은 기술 release 절차만 다룸.

---

## 8. References

- CLAUDE.local.md §1 (Branch Model) — 브랜치 수명 및 머지 전략.
- CLAUDE.local.md §2 (Branch Protection Rules) — `main` / `release/*` / `develop` 보호 규칙.
- CLAUDE.local.md §3 (Label 체계) — 3축 라벨 (type/priority/area).
- CLAUDE.local.md §4 (Merge Strategy) — feature → develop (squash), release → main (merge --no-ff + tag).
- CLAUDE.local.md §5 (Release Drafter) — CHANGELOG 자동화 카테고리 매핑.
- CLAUDE.local.md §6.2 (Release 준비) — 표준 release 체크리스트.
- CLAUDE.local.md §6.3 (Hotfix) — hotfix 절차.
- CLAUDE.local.md §8 (Version 1차 확정 전 임시 규칙) — v0.1.0 release 시점에 삭제 대상.
- CLAUDE.md §7 Rule 4 (Reproduction-First Bug Fixing) — hotfix 시 reproduction test 우선.
- `.moai/specs/SPEC-V3-001/spec.md` — GPUI scaffold acceptance.
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core acceptance.
- `.moai/specs/SPEC-V3-003/spec.md` — Pane + Tab + Persistence acceptance.
- `.moai/specs/SPEC-V3-004/spec.md` — Render Layer Integration (in progress).
- `.github/workflows/ci-rust.yml` — Rust CI (v3).
- `.github/workflows/ci-v3-pane.yml` — Pane CI (SPEC-V3-003 regression gate).
- `.github/workflows/release-drafter.yml` — Release Drafter automation.

---

Version: 1.0.0
Last Updated: 2026-04-25
Scope: moai-studio v0.1.0 first stable release
