# SPEC-V3-002: Terminal Core — libghostty-vt + PTY + Shell 통합

---
id: SPEC-V3-002
version: 0.2.0-draft
status: draft (EARS 구조 확보, /moai plan SPEC-V3-002 에서 research.md + annotation cycle 로 확정)
created: 2026-04-21
updated: 2026-04-21
author: MoAI (SPEC-V3-001 RG-V3-3 rescope + 본문 보강)
priority: High (Phase 2 Terminal Core)
issue_number: 0
depends_on: SPEC-V3-001
---

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 0.1.0-stub | 2026-04-21 | SPEC-V3-001 RG-V3-3 rescope stub. 이관 근거 기록 |
| 0.2.0-draft | 2026-04-21 | EARS 요구사항 그룹 5개, Acceptance Criteria 8개 정식화. Research 주제 목록화. /moai plan 에서 research.md + annotation cycle 수행 전제 |

---

## 1. 개요

SPEC-V3-001 RG-V3-3 (libghostty-vt 스파이크) 가 재진단 결과 **"Metal blocker"가 아닌 작업 미시작 상태** 임이 확인되어, 터미널 통합 자체를 별도 SPEC 으로 분리.

원 SPEC-V3-001 §9 Exclusions 에 명시된 "Phase 2" Terminal Core 작업과 통합하여 본 SPEC 에서 일괄 다룬다.

**rescope 근거** (SPEC-V3-001/progress.md 참조):
- Metal toolchain 환경 ✅ 작동 확인 (`xcrun -sdk macosx metal`, cryptex MobileAsset v17.5)
- Zig 0.15.2 ✅ 설치 확인
- libghostty-rs 의존성 ❌ 미추가 (TODO 주석 상태)
- pinned commit ❌ 미결정 (alpha 상태)
- 실제 작업 규모: FFI wrapping + portable-pty + shell spawn + GPUI 텍스트 렌더 = 독립 SPEC 가치

---

## 2. 요구사항 그룹 (EARS)

### RG-V3-002-1: libghostty-rs 의존성 통합

**[Ubiquitous]** `moai-studio-terminal` crate 는 libghostty-rs 의 pinned commit 에 대한 Cargo 의존성을 **포함해야 한다**.

**[Event-Driven]** 개발자가 `cargo build -p moai-studio-terminal` 을 실행하면, 시스템은 Zig 0.15.x 를 자동 호출해 libghostty-vt 를 빌드하고 FFI 심볼을 Rust 에 노출**해야 한다**.

**[Unwanted]** Zig 0.15.x 가 PATH 에 없는 환경에서 시스템은 빌드를 **시작해서는 안 되며**, 명확한 에러 메시지 `"Zig 0.15.x required — install via mise/asdf/ziglang.org"` 를 출력하고 exit 1 **해야 한다**.

### RG-V3-002-2: PTY + Shell spawn

**[Ubiquitous]** `moai-studio-terminal::pty` 모듈은 portable-pty 0.9+ 를 래핑한 cross-platform PTY 추상화를 **제공해야 한다**.

**[Event-Driven]** 사용자가 새 terminal surface 를 요청하면, 시스템은 `$SHELL` 환경변수 (fallback `/bin/zsh` on macOS, `/bin/bash` on Linux) 를 spawn 하고 stdin/stdout/stderr 을 PTY master 에 바인딩**해야 한다**.

**[State-Driven]** PTY 가 살아있는 상태에서 (While alive) 시스템은 stdout/stderr 바이트 스트림을 libghostty-vt parser 에 실시간 전달**해야 한다**.

**[Unwanted]** 쉘 프로세스가 crash/exit 하면 시스템은 PTY 핸들을 1초 이내에 정리**해야 한다** (고아 FD 금지).

### RG-V3-002-3: GPUI 터미널 렌더링

**[Ubiquitous]** `moai-studio-ui::terminal::TerminalSurface` GPUI 컴포넌트는 libghostty-vt 의 Grid State 를 읽어 각 셀을 Glyph 로 렌더링**해야 한다**.

**[Event-Driven]** libghostty-vt parser 가 state 변화를 emit 하면 GPUI `cx.notify()` 를 호출해 surface 를 re-render**해야 한다**.

**[State-Driven]** RootView 의 active workspace 가 존재하고 TerminalSurface 가 생성된 상태에서, content_area 는 Empty State CTA 대신 TerminalSurface 를 표시**해야 한다**.

### RG-V3-002-4: 입력 경로 (Keyboard → PTY)

**[Event-Driven]** TerminalSurface 에 포커스가 있는 상태에서 사용자가 키를 누르면, GPUI key event 를 ANSI escape sequence 로 변환해 PTY master stdin 에 write**해야 한다**.

**[Event-Driven]** 사용자가 `Cmd+C` (macOS) / `Ctrl+Shift+C` (Linux) 를 누르면, 시스템은 선택 영역의 텍스트를 시스템 클립보드로 복사**해야 한다** (SIGINT 와 구분).

### RG-V3-002-5: CI matrix 확장

**[Ubiquitous]** `.github/workflows/ci-rust.yml` 의 matrix 는 Zig 0.15.x 설치 스텝 (`mlugg/setup-zig@v1`) 을 macOS + Linux runner 모두에 **포함해야 한다**.

**[Event-Driven]** PR 이 생성되면 CI 는 SPEC-V3-001 의 4 gate 에 더해 `cargo run --example ghostty-spike -- --headless` 스모크 테스트를 **추가 실행해야 한다**.

---

## 3. Acceptance Criteria

| AC | Given | When | Then |
|----|-------|------|------|
| AC-T-1 | libghostty-rs pinned, Zig 0.15.x 설치 | `cargo build -p moai-studio-terminal` | 빌드 성공, warning 0 |
| AC-T-2 | Zig 미설치 환경 | `cargo build -p moai-studio-terminal` | exit 1 + `"Zig 0.15.x required"` 메시지 |
| AC-T-3 | PTY spawn 가능한 환경 | `cargo run --example ghostty-spike` | 새 GPUI 윈도우에 `$SHELL` prompt 표시 |
| AC-T-4 | 스파이크 윈도우 실행 중 | 키 입력 (예: `echo hello\n`) | 셸이 명령 실행, 결과가 Grid 에 렌더 |
| AC-T-5 | 스파이크 윈도우 실행 중 | 셸에서 `exit` | 1초 이내 윈도우 종료, FD 누수 없음 (verified via lsof count) |
| AC-T-6 | RootView + active workspace | 사용자가 TerminalSurface 생성 | content_area 가 Empty State 대신 TerminalSurface 렌더 |
| AC-T-7 | CI PR 트리거 | GitHub Actions 실행 | macOS + Linux 에서 Zig 설치 + spike 빌드 + 스모크 통과 |
| AC-T-8 | RootView 테스트 | `cargo test -p moai-studio-terminal` | 신규 테스트 ≥ 10, PTY mock 기반 |

---

## 3. 전제 의존성

- **SPEC-V3-001 완료**: GPUI 윈도우 + RootView (✅ Phase 1.8)
- **SPEC-V3-001 CI**: GitHub Actions matrix (✅ Phase 1.9, run 24708460052)
- **libghostty-rs upstream**: https://github.com/Uzaaft/libghostty-rs — pinned commit 조사 필요
- **portable-pty**: crates.io 최신 (2024-10 기준 0.9.x)
- **Zig**: 0.15.x (CI: `mlugg/setup-zig@v1` action)

---

## 4. Research 주제 (next session `/moai plan SPEC-V3-002` 에서 research.md 작성 대상)

1. **libghostty-rs pinned commit 선정**
   - 최신 released commit 스캔 (main 브랜치 HEAD 아님 — stability 필요)
   - crush (charmbracelet) 나 다른 consumer 의 채택 pin 비교
   - alpha 단계 API 변동성 리스크 측정

2. **portable-pty API 설계 선택**
   - 0.9.x 최신 vs 메이저 버전 선택
   - async integration: Tokio 기반 vs blocking thread pool
   - Windows ConPTY 대응 (SPEC 에서는 Phase 7 대상이지만 API 추상화에 영향)

3. **libghostty-vt 공식 예제 / Zed 통합 코드 정독**
   - gpui-ghostty 튜토리얼 (xuanwo.io)
   - Zed 자체 terminal panel 구현 (`crates/terminal` in zed repo)
   - Grid state → GPUI element 매핑 패턴

4. **Zig 0.15.x CI 설치 비용**
   - `mlugg/setup-zig@v1` 캐시 효율
   - Windows runner 의 Zig 지원 (Phase 7 대비)

5. **선택 영역 / 클립보드 처리**
   - GPUI mouse selection → screen coordinate 변환
   - OSC 52 지원 여부 (원격 클립보드)

## 5. 다음 단계

1. **`/moai plan SPEC-V3-002` 실행** — manager-spec 에이전트가:
   - research.md 작성 (위 5 주제 deep-dive)
   - 이 SPEC 에 acceptance criteria 세부화 (AC-T-3~8 구체 metric)
   - Annotation cycle 1-6 iteration 으로 사용자 승인
2. **`/clear`** — plan phase 완료 후 context reset
3. **`/moai run SPEC-V3-002`** — TDD 또는 DDD (quality.yaml 설정 따라) 로 실제 구현
4. **`/moai sync SPEC-V3-002`** — 문서화 + PR

## 6. 제외 (SPEC 밖)

- Shell configuration (`.zshrc`, `.bashrc` loader) — OS 에 위임
- Terminal scrollback UI (마우스 휠 히스토리) — Phase 2.5
- Tab / Pane split — SPEC-V3-003 예정
- SSH / remote terminal — Phase 3+
- Windows 빌드 — Phase 7 (GPUI Windows GA 대기)

---

## 5. 참조

- SPEC-V3-001 § RG-V3-3: 원 요구사항
- SPEC-V3-001 progress.md § 5: rescope 근거
- `.moai/design/master-plan.md` § Phase 2: Terminal Core 설계 방향
- `crates/moai-studio-terminal/Cargo.toml` 내 TODO 주석: 시작점
- [Uzaaft/libghostty-rs](https://github.com/Uzaaft/libghostty-rs)
- [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty)

---

Version: 0.1.0-stub
상태: Stub placeholder — full SPEC 작성 보류 (사용자 승인 + /moai plan 필요)
