# MoAI Studio

> **moai-adk 공식 크로스플랫폼 Agent IDE.**
> SPEC-first 개발 방법론, MoAI-ADK 통합, 27개 Hook 이벤트, 26개 전문 에이전트, TRUST 5 품질 게이트, @MX 코드 어노테이션을 갖춘 고성능 네이티브 터미널 멀티플렉서.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange.svg)](https://www.rust-lang.org/)
[![Pane CI](https://github.com/modu-ai/moai-studio/actions/workflows/ci-v3-pane.yml/badge.svg?branch=develop)](https://github.com/modu-ai/moai-studio/actions/workflows/ci-v3-pane.yml)
[![Rust CI](https://github.com/modu-ai/moai-studio/actions/workflows/ci-rust.yml/badge.svg?branch=develop)](https://github.com/modu-ai/moai-studio/actions/workflows/ci-rust.yml)
![Status](https://img.shields.io/badge/status-pre--v0.1.0-yellow.svg)

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
| **v0.1.0 (unsigned)** | 📅 예정 | GitHub Releases 자동 배포, GHA billing 복구 후 |
| **MS-2 (signed)** | 📅 차기 | macOS codesign + notarize, Windows EV signtool 인증 |
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

### 사전 컴파일된 바이너리

v0.1.0 릴리스 후 [GitHub Releases](https://github.com/modu-ai/moai-studio/releases)에서 다운로드:

- **macOS**: `moai-studio-v0.1.0.dmg` (universal binary: arm64 + x86_64)
- **Linux**: `moai-studio-v0.1.0.deb`, `moai-studio-v0.1.0.AppImage`
- **Windows**: `moai-studio-v0.1.0.msi`

**주의**: 현재는 unsigned 배포입니다 (MS-2에서 서명 예정).

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
