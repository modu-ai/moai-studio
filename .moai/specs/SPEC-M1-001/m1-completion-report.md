# M1 Working Shell — 완료 보고서 (SPEC-M1-001)

- SPEC: SPEC-M1-001 v1.0.0
- 완료 판정일: 2026-04-13
- 판정자: MoAI MS-6 Sprint (quality+tester teammate)
- 참조: [nfr-report.md](nfr-report.md), [.moai/reports/m1-trust5-audit.md](../../reports/m1-trust5-audit.md)

---

## 1. Executive Summary

**판정: 조건부 GO (Conditional GO)**

M1 "Working Shell" 마일스톤의 **Rust core 전체 (7 RG 중 5 RG)** 와 **SwiftUI Shell 코드 구현 (2 RG)** 이 완료되었다. 측정 가능한 모든 NFR 및 AC 는 목표를 달성하였으며, 186개 테스트 0 regression, clippy/fmt clean. 단 **3개 항목은 Xcode UITest 서명 환경과 실제 Claude CLI 바이너리를 필요로 해 본 스프린트 환경(headless)에서 검증 불가** — M2 Kickoff 전 수동 검증 체크리스트로 이월한다.

---

## 2. Sprint 구성 및 진행

| 스프린트 | 태스크 | 담당 | 상태 |
|----------|--------|------|------|
| MS-1 | T-001~T-004 (swift-bridge FFI) + T-019 (Ghostty) | backend-dev + frontend-dev | ✅ |
| MS-2 | T-005~T-011 (Workspace Lifecycle) | backend-dev | ✅ |
| MS-3 | T-012~T-016 (Claude Subprocess Full Stack) | backend-dev | ✅ |
| MS-4 | T-020~T-027 (SwiftUI Shell + Sidebar) | frontend-dev | ✅ |
| MS-5 | T-017~T-018 (Plugin Installer) | backend-dev | ✅ |
| **MS-6** | **T-028~T-030 (E2E + NFR + Go/No-Go)** | **quality+tester** | **✅ (본 보고서)** |

---

## 3. 결과 하이라이트

### 3.1 테스트
- **전체**: `cargo test --workspace` → **186 passed / 0 failed / 5 ignored**
- **신규 M1 테스트 (T-028 + T-029)**:
  - `tests/e2e_working_shell.rs`: 3 tests (전체 파이프라인 + 4-ws 격리 + list sanity)
  - `tests/nfr_stress.rs`: 7 tests (cold start + create + switch + FFI + store CRUD + 4-ws stress + summary)
  - `app/UITests/E2EWorkingShellTests.swift`: 3 UI smoke tests (XCTSkipIf 로 CI 스킵 지원)

### 3.2 NFR 대비표 (§5)

| 항목 | 목표 | 측정 | 합격 |
|------|------|------|------|
| 콜드 스타트 | <1.0s | 14ms (init) / 121ms (init+첫 ws) | ✅ |
| FFI call | <1ms | 0.33–0.39µs | ✅ |
| Workspace 생성 | <3s | 80–134ms | ✅ |
| Workspace 전환 | <100ms | <0.001ms | ✅ |
| Hook HTTP P95 | <10ms | ≈2.5ms/call | ✅ |
| MCP round-trip | <50ms | ≈3ms/call | ✅ |
| Store CRUD | <5ms | insert 0.02ms / list 0.16ms | ✅ |
| 4-ws 동시 | stable | 3 cycle no deadlock | ✅ |
| cargo check | 0/0 | 0 errors 0 warnings | ✅ |
| 터미널 60fps@4K | 60fps | DEFERRED (앱 런타임) | M2 |
| 메모리 <400MB | <400MB | DEFERRED (앱 런타임) | M2 |
| Xcode 빌드 | 0 errors | DEFERRED (서명 환경) | M2 |

### 3.3 AC 매트릭스
- 34 AC 중 **27 PASS (79%)**, **7 DEFERRED (21%)**, **0 FAIL**
- 전체 상세 매트릭스: [.moai/reports/m1-trust5-audit.md §3](../../reports/m1-trust5-audit.md)

### 3.4 TRUST 5
| 필러 | 판정 |
|------|------|
| Tested | PASS |
| Readable | PASS (fmt clean, clippy 0) |
| Unified | PASS (7 RG ↔ crate 1:1 매핑) |
| Secured | PASS (minor: auth token rotation M2 이월) |
| Trackable | PASS (conventional commits, T-xxx 참조) |

### 3.5 @MX Tag Census
- 총 **45 tags / 28 files**
- ANCHOR 24, NOTE 14, WARN 7, TODO **0**
- 모든 WARN 은 @MX:REASON 동반 / anchor_per_file ≤ 3 한도 준수 / TODO 0건 (GREEN 완수)

---

## 4. 조건부 GO — 정식 GO 전 체크리스트

| 조건 | 검증 방법 | 책임 |
|------|-----------|------|
| C-1. Xcode UITest 서명 + `E2EWorkingShellTests` 실행 | macOS 개발자 계정 + Xcode Cloud 또는 로컬 실행 | 배포 담당 |
| C-2. 실제 Claude CLI 로 AC-4.1 응답 수신 | `claude` CLI 설치 + `ANTHROPIC_API_KEY` 설정 후 수동 E2E | 배포 담당 |
| C-3. 10분 4-ws stress + RSS <400MB (AC-7.2) | Instruments 또는 `ps -o rss` 10분 샘플링 | QA |

이 3개 조건 완료 시 M1 **정식 GO** 선언.

---

## 5. 알려진 제한 사항 (M2+ 이월)

| ID | 항목 | 분류 |
|----|------|------|
| C-4 | GhosttyKit Metal 60fps 측정 | 성능 검증 |
| C-5 | swift-bridge Vectorizable workaround 제거 | 기술 부채 |
| C-6 | Auth token rotation (hook-http) | 보안 강화 |
| C-7 | Swift 측 FFI <1ms XCTest benchmark | 검증 보강 |
| C-8 | state machine `force_paused` 정식 API | 기술 부채 |

---

## 6. 산출물 목록 (M1 전체)

### Rust core (12 crates)
```
core/crates/
├── moai-core/         # 공개 facade
├── moai-ffi/          # swift-bridge FFI (MS-1)
├── moai-store/        # SQLite + state machine (MS-2)
├── moai-git/          # git worktree (MS-2)
├── moai-fs/           # FS 감시 (MS-2)
├── moai-supervisor/   # RootSupervisor + lifecycle + restore (MS-2)
├── moai-claude-host/  # subprocess spawn + stdin + monitor (MS-3)
├── moai-stream-json/  # SDKMessage 13종 디코더 (MS-3)
├── moai-ide-server/   # MCP 서버 (MS-3)
├── moai-hook-http/    # Plugin hook HTTP receiver (MS-3)
├── moai-plugin-installer/ # Plugin 자동 설치 (MS-5)
└── moai-integration-tests/ # 크로스-크레이트 통합 (MS-6 확장)
```

### SwiftUI App (MS-4 + MS-6)
```
app/
├── Sources/
│   ├── App/MoAIStudioApp.swift
│   ├── Bridge/RustCore+Generated.swift
│   ├── Shell/{MainWindow, RootSplitView, WindowStateStore, Sidebar/**, Content/**}
│   └── ViewModels/WorkspaceViewModel.swift
├── UITests/
│   ├── E2EShellSmokeTests.swift        (MS-4)
│   └── E2EWorkingShellTests.swift      (MS-6 신규)
└── Frameworks/GhosttyKit.xcframework   (T-019)
```

### 보고서 (MS-6)
```
.moai/specs/SPEC-M1-001/
├── nfr-report.md                       # T-029
└── m1-completion-report.md             # T-030 (본 문서)
.moai/reports/
└── m1-trust5-audit.md                  # T-030
```

---

## 7. 최종 게이트

```
cargo check --workspace               : 0 errors, 0 warnings
cargo clippy --workspace -- -D warnings: 0 errors, 0 warnings
cargo fmt --all -- --check            : clean
cargo test --workspace                : 186 passed, 0 failed, 5 ignored
```

---

## 8. 서명

**M1 Working Shell — 조건부 GO 선언**

- 완료 담당: MS-6 quality+tester teammate
- 완료 일시: 2026-04-13
- 다음 게이트: M2 Kickoff (C-1 ~ C-3 완료 후 정식 GO 재선언)
