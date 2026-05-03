---
id: SPEC-V0-2-0-MULTI-SHELL-001
version: 1.0.0
status: ready
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (sess 11 main session, simplified plan)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-012]
parallel_with: [SPEC-V0-2-0-PLUGIN-MGR-001, SPEC-V0-2-0-MISSION-CTRL-001]
milestones: [MS-1]
language: ko
labels: [v0.2.0, terminal, ui, multi-shell, audit-A-4]
---

# SPEC-V0-2-0-MULTI-SHELL-001: Multi-shell picker — Shell registry + Command Palette switch

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-ready | 2026-05-04 | 초안 + ready 단일 단계. v0.2.0 Sprint 6 (audit A-4) — 멀티 shell 지원 (zsh/bash/fish/nu/pwsh/cmd/sh/dash) registry + Command Palette `shell.switch` entry. UnixPty::spawn_with_shell helper. annotation cycle 생략 (단순 SPEC). |

---

## 1. 개요

### 1.1 목적

design v3 spec.md C-2 멀티 쉘 제약 (`zsh / bash / fish / nu / sh / pwsh / cmd / dash` 지원) 의 v1 lock-in. 사용자가 Command Palette 에서 shell 선택 → pane 의 새 shell session spawn. v0.1.x 부터 carry 된 가장 오래된 critical gap (audit Top 8 #8).

### 1.2 차별화 위치

design v3 §C-2 표:
- macOS: zsh (default), bash, fish, nu, sh, pwsh
- Linux: bash (default), zsh, fish, nu, sh, pwsh, dash
- Windows: pwsh (default), cmd, bash (WSL), nu

v1 lock-in 은 macOS / Linux 의 `zsh / bash / fish / nu / sh` + macOS `pwsh` (있으면). Windows shell 은 별 SPEC.

### 1.3 근거 문서

- `.moai/design/v3/spec.md` v3.1.0 §C-2 멀티 쉘 지원 + §3.1 OS 별 default
- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier A A-4 (PARTIAL → 본 SPEC 으로 DONE)
- `crates/moai-studio-terminal/src/pty/unix.rs:25-40` — UnixPty::spawn_shell (default $SHELL) + spawn(cmd) 기존 API
- `crates/moai-studio-ui/src/palette/registry.rs` — Command Palette entry 추가 위치

---

## 2. 목표 / 비목표

### 2.1 목표 (Goals)

- G1. `crates/moai-studio-terminal/src/shell.rs` 신규 — `Shell` enum (Zsh / Bash / Fish / Nu / Sh / Pwsh / Cmd / Dash) + `executable()` + `display_name()` + `default_args()`
- G2. `Shell::detect_available()` — 시스템에서 `which` (Unix) 또는 `where` (Windows) 명령으로 사용 가능한 shell 검출
- G3. `UnixPty::spawn_with_shell(shell: Shell) -> io::Result<Self>` 신규 — 명시 shell 로 spawn (기존 spawn_shell 은 $SHELL 사용)
- G4. `crates/moai-studio-ui/src/shell_picker.rs` 신규 — `ShellPicker` logic (selected / available / select() / current()) + Command Palette entry handler
- G5. `palette/registry.rs` 에 신규 entry `shell.switch` ("Switch Shell...", category "Terminal", keybinding `None`) — 선택 시 ShellPicker activate
- G6. RootView 에 `shell_picker: Option<ShellPicker>` 필드 (R3 새 필드만) + `handle_switch_shell(cx)` method

### 2.2 비목표 (N1~N5)

- N1. **Windows shell 지원** — pwsh / cmd / WSL bash. v1 은 Unix (macOS/Linux) only. Windows ConPty 통합은 별 SPEC.
- N2. **Shell profile 자동 source** — `.zshrc / .bashrc / fish.config` 자동 source 는 spawn 시 default shell behavior 그대로. 추가 로직 없음.
- N3. **Active pane 의 in-place shell 교체** — v1 은 새 pane 만. active pane 교체는 별 PR.
- N4. **Shell environment 전달 (`MOAI_WORKSPACE_ID`, `MOAI_PANE_ID` 등)** — v1 미구현. design v3 §C-2 carry.
- N5. **GUI shell picker modal** — Command Palette 만. dropdown/dialog 별 PR.

---

## 3. 사용자 스토리

- US-MS1: 개발자가 Cmd+K → "Switch Shell..." 선택 → fish, bash, nu, zsh, sh 중 선택 → 새 pane 이 fish 로 spawn.
- US-MS2: 개발자가 nu 가 시스템에 없을 때 picker 에서 nu 항목이 grayed-out 또는 absent.
- US-MS3: 개발자가 Cmd+K → "List Available Shells" → 사용 가능한 shell 목록 + 현재 default ($SHELL) 표시.

---

## 4. EARS 요구사항

| REQ ID | 패턴 | 요구사항 |
|--------|------|---------|
| REQ-MS-001 | Ubiquitous | 시스템은 `Shell` enum 8 variant (Zsh / Bash / Fish / Nu / Sh / Pwsh / Cmd / Dash) + `executable() -> &str` + `display_name() -> &str` + `default_args() -> Vec<String>` 를 노출한다. |
| REQ-MS-002 | Ubiquitous | `Shell::all_unix() -> Vec<Shell>` 가 Unix 에서 검토 가능한 5+ shell list 반환. `Shell::all_windows()` 별도 (v1 미구현, stub). |
| REQ-MS-003 | Event-Driven | `Shell::detect_available()` 호출 시, 시스템은 `which {executable}` (Unix) 명령을 각 shell 에 대해 실행하여 exit 0 여부로 사용 가능 여부 판단. 결과는 `Vec<Shell>` 으로 반환. |
| REQ-MS-004 | Ubiquitous | `UnixPty::spawn_with_shell(shell: Shell) -> io::Result<UnixPty>` 가 `shell.executable()` 을 명령어로 spawn. 기존 `spawn_shell()` 은 변경 없음 ($SHELL fallback). |
| REQ-MS-005 | Ubiquitous | `ShellPicker` struct (selected / available / current_default) + `new(available, current_default) -> Self` + `select(shell) -> Option<Shell>` + `current() -> Option<Shell>`. |
| REQ-MS-006 | Ubiquitous | `palette/registry.rs` 에 `CommandEntry::new("shell.switch", "Switch Shell...", "Terminal", None)` 추가. id / category 안정. |
| REQ-MS-007 | Event-Driven | RootView 에 `shell_picker: Option<ShellPicker>` 추가 (R3 새 필드만). `handle_switch_shell(cx)` 가 detect_available() + ShellPicker::new + Some 로 활성화. |

---

## 5. R 제약

- R1. `crates/moai-studio-terminal/src/pty/unix.rs` 의 `spawn_shell` / `spawn` 시그니처 변경 금지 (READ-ONLY 호출). `spawn_with_shell` 신규 함수 추가는 OK.
- R2. `crates/moai-studio-terminal/src/pty/mod.rs` Pty trait 시그니처 변경 금지.
- R3. RootView 에 `shell_picker: Option<ShellPicker>` 필드만 추가. 기존 필드 rename / delete 금지.
- R4. `palette/registry.rs` 의 기존 entry 무변경, `shell.switch` 신규만 추가.
- R5. 기존 SPEC-V3-002 (Terminal Core) 의 모든 코드 영역 무변경 (spawn flow 만 새 helper 추가).

---

## 6. Acceptance Criteria

| AC ID | 검증 방법 | DoD |
|-------|-----------|-----|
| AC-MS-1 | 단위: `Shell` enum 8 variant 확인 + executable/display_name/default_args 매핑 확인 | `cargo test -p moai-studio-terminal --lib shell::tests` PASS |
| AC-MS-2 | 단위: `Shell::all_unix()` 가 Zsh/Bash/Fish/Nu/Sh/Pwsh 중 5+ 반환 | 단위 테스트 PASS |
| AC-MS-3 | 단위: `Shell::detect_available()` 가 적어도 sh 또는 bash 1 개 이상 발견 (CI runner 환경 보장) | 단위 테스트 PASS |
| AC-MS-4 | 단위: `UnixPty::spawn_with_shell(Shell::Sh)` 가 sh 프로세스 spawn 성공 + alive 검증 | 단위 테스트 PASS |
| AC-MS-5 | 단위: `ShellPicker::new(available, current)` + `select(Shell::Bash)` → `current() = Some(Bash)` | 단위 테스트 PASS |
| AC-MS-6 | 단위: `palette/registry.rs` 에 `shell.switch` entry 존재 + label / category 정확 | 단위 테스트 PASS |
| AC-MS-7 | 단위: RootView `handle_switch_shell()` 호출 시 `shell_picker.is_some()` true | 단위 테스트 (logic-level) PASS |

---

## 7. Milestone

### MS-1 (single milestone, 본 SPEC 전체)

- 신규: `crates/moai-studio-terminal/src/shell.rs` (~120 LOC), `crates/moai-studio-ui/src/shell_picker.rs` (~80 LOC)
- 수정: `crates/moai-studio-terminal/src/{lib.rs, pty/unix.rs}`, `crates/moai-studio-ui/src/{lib.rs, palette/registry.rs}` (수정 ~50 LOC)
- 추정 LOC: prod ~250, test ~80, 합계 ~330
- 검증 AC: AC-MS-1 ~ AC-MS-7

---

## 8. 위험

- 위험 1: `which` subprocess spawn cost — detect_available() 가 Shell::all_unix() 의 6+ shell 에 대해 which 실행. `Command::new("which").arg(name).status()` 동기 호출 ~5ms × 6 = ~30ms. 본 SPEC v1 에서는 일회성 (Command Palette 활성화 시점) 이므로 무시 가능. 빈번 호출 시 결과 캐시는 v0.2.1 carry.
- 위험 2: pwsh 가 Unix 에 설치되어 있어도 standard PATH 외에 있을 수 있음 — `which` 가 fail. v1 은 standard PATH 만 검색.

---

## 9. v0.2.1+ Carry

- Windows shell (pwsh / cmd / WSL bash) — 별 SPEC
- Active pane in-place shell 교체
- Shell environment 전달 (MOAI_WORKSPACE_ID 등)
- GUI shell picker modal (dropdown / dialog)
- detect_available() 결과 캐시
