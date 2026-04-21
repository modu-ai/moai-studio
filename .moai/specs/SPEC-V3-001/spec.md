# SPEC-V3-001: GPUI 스캐폴드 + Rust core 통합 (v3 아키텍처 전환)

---
id: SPEC-V3-001
version: 1.0.0
status: draft
created: 2026-04-21
updated: 2026-04-21
author: MoAI (manager-spec)
priority: Critical
issue_number: 0
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.0.0 | 2026-04-21 | 초안 작성. v3 아키텍처 대폭 pivot 반영 (GPUI + libghostty-vt, Tauri/Swift 폐기). Phase 0 + Phase 1 범위. 9 핵심 결정 (master-plan.md §Executive) 에 근거. |
| 1.1.0 | 2026-04-21 | Phase 1 체크포인트 sync. RG-V3-1/2/5 완료 (Phase 0.2~1.8, 8 커밋), 248 tests 통과 regression 0. RG-V3-3 (libghostty) 는 Metal Toolchain 블로킹, RG-V3-4 (CI matrix) 미시작. 상세: `.moai/specs/SPEC-V3-001/progress.md`. |
| 1.2.0 | 2026-04-21 | RG-V3-4 CI matrix 실증 완료 (GoosLab/moai-studio repo 생성 + CI run 24708460052 ALL GREEN, macOS + Linux × rust+smoke 4 job). RG-V3-3 재진단 결과 "Metal blocker" 오해 확인 — Metal toolchain/Zig/Xcode 모두 정상, 실제로는 libghostty-rs 스파이크 작업 미시작 상태. RG-V3-3 을 **SPEC-V3-002 (Terminal Core)** 로 rescope. 본 SPEC 은 4/5 RG 실증 완료로 종결 처리. 상세: progress.md, SPEC-V3-002/spec.md (stub). |

---

## 1. 개요

MoAI Studio v3 아키텍처 전환의 **스캐폴드 스프린트**. 기존 Swift/AppKit 중심 M0~M2.5 산출물을 archive 로 이동하고, **Rust + GPUI + libghostty-vt** 기반의 새로운 워크스페이스를 구축한다.

**성공 기준**:
1. Rust Cargo workspace 재구성 완료 (기존 `core/crates/` → `crates/moai-core`, 289 tests 회귀 0)
2. GPUI 윈도우 + 사이드바 + 메뉴 바 + Empty State CTA 가 **macOS + Linux** 에서 빈 프로젝트 상태로 띄워짐
3. libghostty-vt 스파이크 성공 (간단 쉘 실행 + 텍스트 렌더)
4. GitHub Actions CI matrix (macOS + Linux) 에서 `cargo build --release` 통과
5. 기존 Swift 코드는 `archive/swift-legacy/` 로 보존

**선행 조건**:
- SPEC-M2-002 (M2.5 Polish, Swift) 는 v3 대상 아님 — archive 처리
- SPEC-M2-003, SPEC-M3-001 등 이전 draft 는 v3 재평가 후 일부 재활용

**근거 문서**:
- `.moai/design/master-plan.md` (9 결정 종합)
- `.moai/design/spec.md` (25 기능 Tier)
- `.moai/design/system.md` (디자인 토큰)
- `.moai/design/research.md` (경쟁 분석)

---

## 2. 요구사항 그룹 (EARS)

### RG-V3-1: Workspace 재구성

**[Ubiquitous]** 시스템은 Rust Cargo workspace 구조 (`Cargo.toml` at root, `crates/*`) 를 **유지해야 한다** (shall maintain).

**[Event-Driven]** 개발자가 `cargo test --workspace` 를 실행하면 (When), 시스템은 기존 `moai-core` 289 tests 를 **모두 통과해야 한다** (shall pass all).

**[State-Driven]** 기존 Swift 코드가 존재하는 상태에서 (While), 시스템은 `archive/swift-legacy/` 로 이동 후 Git 히스토리를 **보존해야 한다** (shall preserve).

**[Unwanted]** `crates/moai-core` 로의 이동 과정에서 테스트 수가 **감소해서는 안 된다** (shall not decrease).

### RG-V3-2: GPUI 통합 + 기본 윈도우

**[Ubiquitous]** `moai-studio-app` 바이너리는 GPUI 프레임워크를 사용하여 메인 윈도우를 **렌더해야 한다** (shall render).

**[Event-Driven]** 사용자가 앱을 실행하면 (When), 시스템은 1600 × 1000 기본 크기의 윈도우를 **표시해야 한다** (shall display). 윈도우는 타이틀바 + 사이드바 (260pt) + 컨텐츠 영역 + 상태바 (28pt) 의 4 영역을 **포함해야 한다** (shall include).

**[Event-Driven]** 사용자가 사이드바의 "+ New Workspace" 를 클릭하면 (When), 시스템은 NSOpenPanel 등가의 네이티브 폴더 선택 다이얼로그를 **제공해야 한다** (shall provide).

**[State-Driven]** 워크스페이스가 0개인 상태 (While initial state), 시스템은 컨텐츠 영역에 Welcome CTA (Create First Workspace / Start Sample / Open Recent) 를 **표시해야 한다** (shall display).

### RG-V3-3: libghostty-vt 스파이크 — **RESCOPED to SPEC-V3-002 (2026-04-21)**

> **현 상태**: 본 RG 는 SPEC-V3-002 (Terminal Core) 로 이관됨. 재진단 결과 "Metal Toolchain blocker" 는 오해였으며 (Metal/Zig/Xcode 모두 정상), 실제로는 libghostty-rs 스파이크 작업이 시작조차 되지 않은 상태 + alpha 단계인 upstream 의 pinned commit 결정 + FFI 통합 전체가 독립 SPEC 가치라는 판단. 원 AC-3.1 / AC-3.2 는 SPEC-V3-002 의 AC-T-1 / AC-T-2 로 계승됨.

~~**[Ubiquitous]** `moai-studio-terminal` crate 는 libghostty-vt (pinned commit) 에 대한 Rust FFI 바인딩 (via libghostty-rs) 을 **포함해야 한다** (shall include).~~

~~**[Event-Driven]** 개발자가 스파이크 예제 바이너리를 실행하면 (When), 시스템은 `$SHELL` 을 spawn 하여 "Hello from libghostty" 에 해당하는 텍스트 출력을 GPUI 윈도우에 **렌더해야 한다** (shall render).~~

~~**[Unwanted]** Zig 0.15.x 미설치 환경에서 시스템은 빌드를 **시작해서는 안 되며**, 명확한 에러 메시지 ("Zig 0.15.x required — install via mise/asdf/ziglang.org") 를 **출력해야 한다** (shall emit).~~

### RG-V3-4: CI Matrix + 품질 게이트

**[Ubiquitous]** GitHub Actions workflow (`build.yml`) 은 macOS 14+ 및 Ubuntu 22.04+ runner 에서 `cargo build --release` 를 **병렬 실행해야 한다** (shall execute in parallel).

**[Event-Driven]** PR 이 생성되면 (When), CI 는 다음 순서로 게이트를 **실행해야 한다** (shall execute):
1. `cargo fmt --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`
4. `cargo build --release` (3 플랫폼)

**[Unwanted]** 위 게이트 중 하나라도 실패하면 PR 머지를 **차단해야 한다** (shall block).

### RG-V3-5: Swift 자산 아카이브

**[Event-Driven]** Phase 0 시작 시점에 (When), 시스템은 `app/` 디렉토리 전체를 `archive/swift-legacy/` 로 **이동해야 한다** (shall move). Git mv 로 히스토리를 보존한다.

**[Ubiquitous]** `archive/swift-legacy/README.md` 는 아카이브 사유와 v3 pivot 근거 링크를 **포함해야 한다** (shall include).

---

## 3. 수용 기준 (Acceptance Criteria)

| AC | Given | When | Then |
|----|-------|------|------|
| AC-1.1 | 기존 moai-studio repo | `git mv core/crates crates/moai-core` 후 `cargo test --workspace` 실행 | 289 tests 모두 통과, 새 실패 0 |
| AC-1.2 | 리팩토링된 workspace | `cargo build --release --all-targets` | 모든 crate 빌드 성공, warning 0 |
| AC-2.1 | GPUI 스캐폴드 완료 | 바이너리 실행 | 1600×1000 윈도우 표시, 4 영역 (타이틀/사이드바/컨텐츠/상태바) 렌더 |
| AC-2.2 | 빈 워크스페이스 목록 | 앱 실행 | Welcome CTA (3 버튼) 컨텐츠 영역 표시 |
| AC-2.3 | "+ New Workspace" 클릭 | 사이드바 버튼 클릭 | 네이티브 폴더 선택 다이얼로그 표시 |
| AC-3.1 | libghostty-vt crate 설치 | `cargo run --example ghostty-spike` | 터미널 윈도우에 `$SHELL` 프롬프트 렌더 |
| AC-3.2 | Zig 미설치 환경 | `cargo build` | 명확한 에러 메시지 + exit code 1 |
| AC-4.1 | PR 생성 | CI workflow 트리거 | 4 게이트 모두 통과, macOS+Linux 빌드 성공 |
| AC-4.2 | 린트 오류 포함 PR | `cargo clippy` | CI 실패, 머지 차단 |
| AC-5.1 | `app/` Swift 코드 존재 | `git mv app/ archive/swift-legacy/` | 히스토리 보존된 채 이동, README 생성 |

---

## 4. 마일스톤

### MS-1: Workspace 재구성 (Phase 0)

- `app/` → `archive/swift-legacy/` git mv
- `core/crates/*` → `crates/moai-core` git mv
- Cargo.toml workspace 재설정
- 기존 289 tests 통과 확인

### MS-2: GPUI + libghostty 스파이크 (Phase 0)

- GPUI 의존성 추가 (Zed 서브모듈 또는 path dependency)
- libghostty-rs 의존성 + Zig 0.15.x CI 설치 스크립트
- "Hello World" GPUI 윈도우 + libghostty 샘플 터미널 렌더

### MS-3: 스캐폴드 바이너리 + 기본 레이아웃 (Phase 1)

- `moai-studio-app` 엔트리 + 4 영역 레이아웃
- `moai-studio-ui` 초기 컴포넌트 (Sidebar, TitleBar, StatusBar 뼈대)
- `moai-studio-workspace` 초기 CRUD + SQLite persistence
- Empty State CTA
- 네이티브 폴더 picker

### MS-4: CI matrix (Phase 1)

- GitHub Actions workflow (macOS + Linux)
- 4 품질 게이트
- 자동 릴리즈 아티팩트 (`.app`, `.AppImage`) 스모크

---

## 5. 비기능 요구사항

- **성능**: 콜드 스타트 ≤ 500ms (빈 상태), 윈도우 첫 프레임 ≤ 100ms
- **메모리**: 빈 상태 RSS ≤ 120 MB
- **빌드 시간**: Clean build macOS+Linux 각 ≤ 5 분 (CI 환경)
- **안정성**: 기존 289 tests regression 0
- **접근성**: VoiceOver (macOS) / Orca (Linux) 기본 지원

---

## 6. 의존성 및 제약

### 외부 의존성

- **GPUI**: Zed 저장소에서 서브모듈 또는 `git = "..."` path dependency (공식 crates.io 배포 대기)
- **libghostty-vt** via **libghostty-rs**: `Uzaaft/libghostty-rs` (pinned commit, 현재 alpha)
- **portable-pty**: crates.io (cross-platform PTY)
- **Zig 0.15.x**: CI + 개발자 로컬 환경 필수

### 내부 제약

- Rust MSRV 1.82+ (GPUI 요구)
- macOS 14+, Ubuntu 22.04+, Windows 11 (Windows 는 Phase 7)
- Swift 코드 수정 금지 (archive 만)
- moai-core 테스트 유지

### Git 제약

- `app/` → `archive/swift-legacy/` 이동 시 `git mv` 사용 (히스토리 보존)
- main 브랜치 직접 커밋 금지, `feat/v3-scaffold` 브랜치 사용

---

## 7. 테스트 전략

### 유닛 테스트

- `moai-core` 기존 289 tests 유지
- `moai-studio-ui` 신규 컴포넌트별 스모크 (≥ 5)
- `moai-studio-workspace` CRUD (≥ 10)
- `moai-studio-terminal` libghostty FFI wrap (≥ 5)

### 통합 테스트

- GPUI 헤드리스 렌더 (if supported) 또는 스냅샷 기반
- libghostty 스파이크 실제 쉘 spawn 테스트

### 수동 검증

- macOS: 빌드 → 실행 → 4 영역 표시 → CTA 클릭 → 폴더 picker
- Linux: 동일 시나리오
- Zig 미설치 환경: 에러 메시지 확인

### 성능 벤치

- criterion 기반 cold-start 벤치 (시작 → 첫 프레임)
- RSS 측정 (ps -o rss)

---

## 8. 참조 문서

- `.moai/design/master-plan.md` (§Executive + Phase 0+1)
- `.moai/design/spec.md` (Tier A Terminal Core 일부 선반영)
- `.moai/design/research.md` (GPUI + libghostty 결정 근거)
- `.moai/design/archive/tb-vs-tc-report.md` (TB vs TC 비교)
- [Zed GPUI repo](https://github.com/zed-industries/zed)
- [libghostty-rs](https://github.com/Uzaaft/libghostty-rs)
- [awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty)
- [gpui-ghostty tutorial](https://xuanwo.io/2026/01-gpui-ghostty/)

---

## 9. Exclusions (What NOT to Build in this SPEC)

- Windows 빌드 (Phase 7, GPUI Windows GA 대기)
- moai-adk 플러그인 (Phase 6)
- Smart Link Handling (Phase 3)
- 모든 Surfaces (Phase 4, 본 SPEC 은 기본 layout skeleton 만)
- Tab UI / Pane split (Phase 2)
- Command Palette (Phase 5)
- LSP 통합 (Phase 4+)
- 자동 업데이트 (Phase 8)
- 배포 산출물 (Phase 8)
- Mermaid 렌더러 (Phase 4)
- tmux 호환성 검증 (Phase 2)

---

## 10. 용어 정의

- **GPUI**: Zed Editor 가 사용하는 Rust-native GPU UI 프레임워크. Metal/Vulkan/D3D 기반.
- **libghostty-vt**: Ghostty 에서 추출된 zero-dependency C/Zig 터미널 VT state/parser 라이브러리.
- **libghostty-rs**: libghostty-vt 의 Rust FFI 바인딩 (Uzaaft 유지).
- **Phase 0**: v3 아키텍처 전환을 위한 준비 + 스파이크 단계.
- **Phase 1**: 스캐폴드 + 기본 레이아웃 구축 단계.
- **MSRV**: Minimum Supported Rust Version.

---

버전: 1.0.0 · 2026-04-21
