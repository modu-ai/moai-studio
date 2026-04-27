# MoAI Studio

> **moai-adk 공식 크로스플랫폼 Agent IDE.**
> SPEC-first 개발 방법론, MoAI-ADK 통합, 27개 Hook 이벤트, 26개 전문 에이전트, TRUST 5 품질 게이트, @MX 코드 어노테이션을 갖춘 고성능 네이티브 터미널 멀티플렉서.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org/)
[![Pane CI](https://github.com/modu-ai/moai-studio/actions/workflows/ci-v3-pane.yml/badge.svg?branch=develop)](https://github.com/modu-ai/moai-studio/actions/workflows/ci-v3-pane.yml)
[![Rust CI](https://github.com/modu-ai/moai-studio/actions/workflows/ci-rust.yml/badge.svg?branch=develop)](https://github.com/modu-ai/moai-studio/actions/workflows/ci-rust.yml)
![Status](https://img.shields.io/badge/status-v0.1.1-green.svg)

**저장소**: `github.com/modu-ai/moai-studio` (2026-04-26 GoosLab에서 modu-ai org로 이전)  
**언어**: Pure Rust  
**Edition**: Rust 2024  
**MSRV**: Rust 1.93+  
**라이선스**: Apache License 2.0

---

## 이것이 MoAI Studio입니다

MoAI Studio는 **SPEC-first 개발 방법론을 구현한 크로스플랫폼 Agent IDE**입니다. Rust 기반의 고성능 엔진과 GPUI 프레임워크를 통해 GPU 가속 네이티브 UI를 제공하며, 터미널 엔진으로 libghostty-vt를 통합합니다.

**핵심 특징:**

- **SPEC-driven 워크플로우**: EARS 형식 요구사항, TRUST 5 품질 게이트, @MX 코드 어노테이션으로 엄격한 개발 규율 유지
- **MoAI-ADK 통합**: Go CLI 오케스트레이션 프레임워크와 시스템 수준 협력 (Hook, Agent dispatch)
- **고성능 Terminal**: libghostty-vt 기반 터미널 멀티플렉서로 최대 성능 + 호환성
- **크로스플랫폼 배포**: macOS (arm64 + x86_64), Linux (x86_64), Windows (x86_64) — 부호화 및 자동 업데이트는 MS-2 예정

---

## 현재 상태 & 로드맵

| 마일스톤 | 상태 | 구현 내용 |
|---------|------|---------|
| **V3 Scaffold** | ✅ 완료 | 23개 Rust crate 컴파일, 기본 UI/Terminal 구조 |
| **SPEC-V3-001~013** | 🔄 진행 중 | 핵심 Agent IDE 기능, Pane system, Terminal multiplexing |
| **SPEC-V3-011 MS-1** | ✅ 완료 | macOS (.app), Linux (.deb + .AppImage), Windows (.msi) 패키징 infra |
| **v0.1.0 (unsigned)** | ✅ 첫 정식 릴리스 | GitHub Releases unsigned 배포 (USER-DECISION-PK-B (b) 결정) |
| **SPEC-V3-DIST-001** | 📅 ready (구현 예정) | Homebrew Cask + Scoop bucket + AUR + AppImage 무료 배포 채널 등록 |
| **MS-2 (signed)** | 📅 차기 | macOS codesign + notarize, Windows EV signtool 인증 (인증서 보유 시) |
| **MS-3 (auto-update)** | 📅 차기 | Ed25519 서명, GitHub Releases JSON manifest 기반 자동 업데이트 |

---

## 아키텍처 (V3)

```
┌─────────────────────────────────────────────────────────────┐
│              MoAI Studio (Rust + GPUI)                      │
│                                                             │
│   GPU-accelerated UI (gpui crate)                          │
│       │                                                     │
│       ▼                                                     │
│   moai-studio-app (binary: moai-studio)                    │
│       │                                                     │
│       ├─ moai-studio-terminal ──────► libghostty-vt        │
│       ├─ moai-studio-ui              (terminal subsystem)  │
│       ├─ moai-studio-workspace                             │
│       ├─ moai-studio-spec   ──────► .moai/specs/ parsing   │
│       ├─ moai-studio-agent  ──────► agent dispatch         │
│       └─ moai-studio-plugin-api ──► plugin runtime         │
│                                                             │
│   Foundation crates (moai-core, moai-fs, moai-git, ...)    │
└─────────────────────────────────────────────────────────────┘
                       │
                       ▼ subprocess / IPC
              moai-adk-go (Go CLI orchestration)
```

**핵심 설계:**

- **Pure Rust**: 메모리 안전성, 성능, 빌드 일관성 (Swift 레거시 제거 완료)
- **GPUI 기반**: Zed Industries의 GPU 가속 UI 프레임워크 (의존성 계획)
- **libghostty-vt 통합**: ghostty-org의 재사용 가능한 VT 서브시스템
- **23개 Rust crate 워크스페이스**: 모듈식 아키텍처, 각 도메인별 책임 분리

---

## 저장소 구조

```
moai-studio/
├── README.md                  ← 이 파일
├── CHANGELOG.md
├── LICENSE                    ← Apache 2.0
├── Cargo.toml                 ← Rust workspace
├── Cargo.lock
├── crates/                    ← 23개 Rust crate
│   ├── moai-claude-host/
│   ├── moai-core/
│   ├── moai-fs/
│   ├── moai-git/
│   ├── moai-studio-app/       ← 메인 binary (moai-studio)
│   ├── moai-studio-ui/        ← UI layer
│   ├── moai-studio-terminal/  ← Terminal multiplexer
│   ├── moai-studio-workspace/
│   ├── moai-studio-spec/      ← SPEC parsing
│   └── ... (16 more crates)
├── .moai/
│   ├── specs/                 ← SPEC-V3-001~015 드래프트 및 구현
│   ├── config/
│   └── project/
├── .github/
│   ├── workflows/             ← CI/CD (Pane CI, Rust CI, Release Drafter)
│   └── labels.yml
├── .claude/                   ← Claude Code 설정 & rules
│   ├── rules/moai/
│   ├── skills/
│   └── agents/
├── scripts/                   ← 빌드 & 배포 유틸
│   ├── build-macos.sh
│   ├── build-linux.sh
│   └── build-windows.sh
├── wix/                       ← Windows MSI 패키징
└── archive/                   ← 이전 버전 (Swift M2 era)
    └── swift-legacy/          ← 레거시 Xcode 프로젝트
```

**주목:**

- `.moai/specs/`: EARS 형식 SPEC 문서, TRUST 5 품질 검증
- `.github/workflows/`: macOS + Linux 매트릭스 CI, bench-smoke, Release Drafter
- `crates/`: 23개 독립 crate — 각 crate가 명확한 책임 소유

---

## 빠른 시작

**필수 조건**: Rust 1.93+, git, 크로스플랫폼 빌드 환경 (Cargo)

### 소스에서 빌드

```bash
git clone https://github.com/modu-ai/moai-studio.git
cd moai-studio
cargo build --release -p moai-studio-app
./target/release/moai-studio
```

## 설치 방법

MoAI Studio는 다양한 채널을 통해 설치할 수 있습니다. 권장되는 방법부터 순서대로 안내합니다.

### 🍺 패키지 매니저 설치 (권장)

#### macOS (Homebrew)
```bash
# Homebrew tap 추가
brew tap modu-ai/tap

# 설치
brew install --cask moai-studio
```

#### Windows (Scoop)
```bash
# Scoop bucket 추가
scoop bucket add moai https://github.com/modu-ai/scoop-bucket

# 설치
scoop install moai-studio
```

#### Arch Linux (AUR)
```bash
# yay를 사용한 설치
yay -S moai-studio-bin

# 또는 다른 AUR 헬러 사용
paru -S moai-studio-bin
```

### 📦 직접 다운로드

#### v0.1.1+ 릴리스 바이너리
[GitHub Releases](https://github.com/modu-ai/moai-studio/releases)에서 최신 버전 다운로드:

- **macOS**: `moai-studio-mac-*.dmg` (universal binary: arm64 + x86_64)
- **Linux**: `moai-studio-linux-amd64.AppImage`
- **Windows**: `moai-studio-windows-x64.msi`

#### Linux AppImage
```bash
# 다운로드
wget https://github.com/modu-ai/moai-studio/releases/latest/download/moai-studio-linux-amd64.AppImage

# 실행 권한 부여
chmod +x moai-studio-linux-amd64.AppImage

# 실행
./moai-studio-linux-amd64.AppImage
```

### 🚀 소스에서 빌드

**필수 조건**: Rust 1.93+, git

```bash
# 저장소 클론
git clone https://github.com/modu-ai/moai-studio.git
cd moai-studio

# 빌드
cargo build --release -p moai-studio-app

# 실행
./target/release/moai-studio
```

---

## Known Limitations (v0.1.0)

v0.1.0은 OS 레벨 인증서 미보유 상태로 release됩니다 (USER-DECISION-PK-B (b) 결정, 2026-04-27). 사용자는 다음 한 번의 우회 작업이 필요하며, 이후 일반 앱과 동일하게 동작합니다.

### macOS — Gatekeeper quarantine 제거

다운로드한 `.dmg`를 마운트하고 `moai-studio.app`을 `/Applications/`로 드래그한 후, 터미널에서 다음 명령을 한 번 실행:

```bash
xattr -dr com.apple.quarantine /Applications/moai-studio.app
open /Applications/moai-studio.app
```

또는 `Finder → moai-studio.app 우클릭 → "열기" → "열기"` 1회. 이후 Spotlight/Launchpad에서 일반 앱처럼 실행 가능.

### Windows — SmartScreen 우회

`.msi` 더블클릭 시 "Windows protected your PC" 경고가 표시됩니다.

```
1. 경고 화면에서 "More info" 클릭
2. "Run anyway" 클릭
```

이후 시작 메뉴에서 일반 앱처럼 실행 가능.

### Linux — 무서명 표준

`.deb` / `.AppImage` 모두 별 인증 작업 없이 실행됩니다.

```bash
# Debian/Ubuntu
sudo dpkg -i moai-studio-v0.1.0.deb

# AppImage (모든 distribution)
chmod +x moai-studio-v0.1.0.AppImage
./moai-studio-v0.1.0.AppImage
```

### 알려진 제약사항

**v0.1.0**: unsigned 배포로 인한 OS별 우회 작업이 필요합니다. 아래 내용 참조하시거나, **v0.1.1+ 로 업그레이드하여 패키지 매니저로 자동 처리**할 수 있습니다.

#### macOS — v0.1.0: Gatekeeper quarantine 제거

다운로드한 `.dmg`를 마운트하고 `moai-studio.app`을 `/Applications/`로 드래그한 후, 터미널에서 다음 명령을 한 번 실행:

```bash
xattr -dr com.apple.quarantine /Applications/moai-studio.app
open /Applications/moai-studio.app
```

또는 `Finder → moai-studio.app 우클릭 → "열기" → "열기"` 1회. 이후 Spotlight/Launchpad에서 일반 앱처럼 실행 가능.

#### Windows — v0.1.0: SmartScreen 우회

`.msi` 더블클릭 시 "Windows protected your PC" 경고가 표시됩니다.

```
1. 경고 화면에서 "More info" 클릭
2. "Run anyway" 클릭
```

이후 시작 메뉴에서 일반 앱처럼 실행 가능.

#### Linux — v0.1.0: 무서명 표준

`.deb` / `.AppImage` 모두 별 인증 작업 없이 실행됩니다.

```bash
# Debian/Ubuntu
sudo dpkg -i moai-studio-v0.1.0.deb

# AppImage (모든 distribution)
chmod +x moai-studio-v0.1.0.AppImage
./moai-studio-v0.1.0.AppImage
```

> **팁**: 패키지 매니저를 사용하면 위의 우회 작업이 필요 없습니다. **v0.1.1+ 부터는 Homebrew, Scoop, AUR에서 자동으로 설치됩니다.**

---

**v0.1.1+ 패키지 매니저 설치**: 패키지 매니저를 통해 설치 시 아래와 같은 이점이 있습니다:
- 자동 서명 및 인증서 처리
- 업데이트 알림 및 자동 업그레이드
- 의존성 자동 관리
- 표준 설치 경로 사용

### 알려진 carry-over 제약

v0.1.0은 V3 아키텍처의 minimum-viable surface로 한정됩니다. 다음 SPEC들은 v0.1.x patch 또는 v0.2.0+ backlog로 이월됩니다:

- SPEC-V3-005 (File Explorer)
- SPEC-V3-006 (Markdown / Code Viewer)
- SPEC-V3-007 ~ 010 (Agent Dashboard, SPEC Management UI 등)
- SPEC-V3-DIST-001 (배포 채널 등록 — 위 안내 참조)

자세한 분류는 [.moai/specs/RELEASE-V0.1.0/plan.md](./.moai/specs/RELEASE-V0.1.0/plan.md) §1.2 참고.

---

## SPEC-driven 개발

MoAI Studio는 **MoAI-ADK 프레임워크**를 통해 SPEC-first 개발을 실현합니다.

**핵심 개념:**

- **EARS 형식**: Ubiquitous, Event-driven, State-driven, Unwanted, Optional 요구사항 표기
- **TRUST 5 게이트**: Tested (85%+ coverage), Readable, Unified, Secured, Trackable 품질 검증
- **@MX 태그**: 코드 수준 어노테이션 (@MX:NOTE, @MX:WARN, @MX:ANCHOR, @MX:TODO)
- **26개 전문 에이전트**: manager-spec, expert-backend, expert-frontend, manager-ddd, manager-tdd 등

현재 구현 중인 SPEC: [.moai/specs/](https://github.com/modu-ai/moai-studio/tree/develop/.moai/specs/)

**학습 자료:**

- [MoAI-ADK 문서](https://github.com/modu-ai/moai-adk) (Go CLI 프레임워크)
- [SPEC-V3-001~015](https://github.com/modu-ai/moai-studio/tree/develop/.moai/specs/)

---

## CI/품질 게이트

모든 코드 변경은 다음 자동 검증을 거칩니다:

| 검사 | 대상 | 상태 |
|------|------|------|
| **Fmt + Clippy** | macOS + Linux | ✅ |
| **Cargo test** | macOS + Linux | ✅ (7 contexts required, §2.1) |
| **Bench-smoke** | macOS + Linux | ✅ |
| **Tmux-test** | macOS + Linux | ⏳ (--ignored 버킷, SPEC-V3-FS-WATCHER-001 pending) |
| **Release Drafter** | PR 라벨 기반 CHANGELOG | ✅ |

**Branch protection** ([CLAUDE.local.md §2](./CLAUDE.local.md#2-branch-protection-rules-hard--github-settings)):

- `main`: 1개 approval + 7 required contexts (Squash merge: feature, Merge commit: release/hotfix)
- `develop`: 7 required contexts, 0 approvals (Squash merge: feature)
- `release/*`, `hotfix/*`: 활성화 시 main과 동일 규칙

---

## 기여 방법

**현재**: 단일 개발자 운영 (Goos Kim, namgoos@gmail.com). 외부 기여는 허용되지만 일괄 처리될 수 있습니다.

**저장소 정책:**

- **branch model**: `main` (releases only) < `release/*` < `develop` < `feature/SPEC-XXX-*` + `hotfix/*`
- **Conventional Commits**: `feat(scope): description [AC-XXX]` + `🗿 MoAI <email@mo.ai.kr>`
- **PR base branch**: `develop` (단, hotfix는 `main`에서 분기)
- **라벨링**: type/ + priority/ + area/ (3축 필수, .github/labels.yml)
- **Auto-merge**: PR approval + CI GREEN 시 자동 머지 (Squash merge for feature)

자세한 내용: [CLAUDE.local.md §6](./CLAUDE.local.md#6-일상-워크플로-체크리스트)

---

## 브랜딩 제약 (Anthropic 공식)

출처: [Claude Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview)

- ✅ **허용**: "MoAI Studio", "MoAI Agent IDE", "moai + Claude", "Powered by Claude"
- ❌ **금지**: "Claude Code" 명칭 사용, "Claude Code Agent" 명명
- ❌ **금지**: claude.ai OAuth 로그인 구현
- ✅ **인증**: `ANTHROPIC_API_KEY`, Bedrock, Vertex, Foundry

---

## 관련 저장소

- [**modu-ai/moai-adk**](https://github.com/modu-ai/moai-adk) — Go CLI 오케스트레이션 프레임워크 (이 GUI의 본체)
- [**ghostty-org/ghostty**](https://github.com/ghostty-org/ghostty) — 터미널 VT 엔진 (libghostty-vt 통합)
- [**zed-industries/zed**](https://github.com/zed-industries/zed) — GPUI UI 프레임워크
- [**anthropics/claude-code**](https://github.com/anthropics/claude-code) — Claude Code CLI

---

## 레거시 설계 문서

다음 문서들은 진화 기록 및 아키텍처 결정으로 보존됩니다:

- [DESIGN.md](./DESIGN.md) (v2) — 초기 아키텍처
- [DESIGN.v3.md](./DESIGN.v3.md) (v3) — SDK 임베드 가정
- [DESIGN.v4.md](./DESIGN.v4.md) (v4) — 공식 문서 기준 Rust + UI
- [NEXT-STEPS.md](./NEXT-STEPS.md) — 다음 마일스톤 계획

Swift M2 era의 레거시 Xcode 프로젝트는 `archive/swift-legacy/`에 보존됩니다 (참고 목적).

---

## 라이선스

[Apache License 2.0](./LICENSE) — 자유로운 상용/개인 사용, 수정, 배포 허용.

Copyright © 2026 MoAI Studio (modu-ai organization)

---

## 연락처

- **메인테이너**: Goos Kim (namgoos@gmail.com)
- **이슈 추적**: [GitHub Issues](https://github.com/modu-ai/moai-studio/issues)
- **토론**: [GitHub Discussions](https://github.com/modu-ai/moai-studio/discussions)
