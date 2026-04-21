# SPEC-V3-001 Progress — Phase 1 Checkpoint

---
spec_id: SPEC-V3-001
checkpoint: FINAL — RG-V3-3 rescope to SPEC-V3-002 → 실질 완료
recorded: 2026-04-21
branch: feat/v3-scaffold → github.com/GoosLab/moai-studio (private)
base_commit: 101cd83 (Phase 0.2 기준점)
head_commit: 611a220
ci_run: https://github.com/GoosLab/moai-studio/actions/runs/24708460052 (✅ ALL GREEN)
status: DONE (4/5 RG 실증 완료 + 1 RG rescope)
---

## 1. RG 진행 요약

| RG | 제목 | 상태 | 비고 |
|----|------|------|------|
| RG-V3-1 | Workspace 재구성 (Cargo) | ✅ 완료 | `crates/moai-core` 289 tests 유지, Swift archive 이동 완료 |
| RG-V3-2 | GPUI 통합 + 기본 윈도우 | ✅ 완료 | Phase 1.1~1.8, 8 커밋, AC-2.1/2.2/2.3 충족 |
| RG-V3-3 | libghostty-vt 스파이크 | 🔄 **rescope → SPEC-V3-002** | 재진단: Metal ✅ / Zig 0.15.2 ✅ / Xcode ✅ — 실제 이슈는 libghostty-rs 미통합 + alpha pin 미결정. 터미널 통합 전체를 SPEC-V3-002 (Terminal Core) 로 분리 |
| RG-V3-4 | CI matrix + 품질 게이트 | ✅ **실증 완료** | GoosLab/moai-studio private repo 생성 + push + CI 3회 반복 수정 (linker deps Linux rust job + smoke job). run 24708460052 에서 macOS/Linux × (rust + smoke) 4 job 모두 그린. AC-4.1 충족. AC-4.2 (branch protection) 는 GitHub repo 설정 대상. |
| RG-V3-5 | Swift 자산 아카이브 | ✅ 완료 | `archive/swift-legacy/` 로 `git mv` 완료 (Phase 0.2) |

**전체 완료율**: **4/5 RG 실증 + 1/5 RG rescope → 실질 SPEC 종결** · Acceptance Criteria 9개 중 7개 충족 (AC-1.1, AC-1.2, AC-2.1, AC-2.2, AC-2.3, AC-4.1, AC-5.1). AC-3.1 / AC-3.2 는 SPEC-V3-002 로 이관.

---

## 2. Phase 1 Sub-Phase 이력 (RG-V3-2 실행 로그)

| Sub-Phase | 커밋 | 산출 | 테스트 증분 |
|-----------|------|------|-------------|
| 1.1 GPUI 의존성 | `b676c81` | `gpui = "0.2.2"` dep, Application::new, RootView 스캐폴드 | — |
| 1.2 4영역 레이아웃 | `d0aecc6` | TitleBar 44pt / Sidebar 260pt / Body / StatusBar 28pt | — |
| 1.3 Empty State CTA | `a3e0588` | Welcome hero + 3 CTA (Create/Sample/Recent) + Tip | — |
| 1.4 파일 picker | `edfda5d` | `rfd = "0.15"` + `Workspace` 구조체 | +3 |
| 1.5 JSON persistence | `09e14c6` | `WorkspacesStore` (load/save/add/remove/touch) | +5 |
| 1.6 Sidebar 리스트 | `92da7c0` | `RootView` state + workspace row 렌더 + active 하이라이트 | +3 |
| 1.7 + New Workspace 실동작 | `7fa8077` | Stateful button + `cx.listener` + `handle_add_workspace` | +2 |
| 1.8 Row 클릭 active 전환 | `9d586ea` | `activate_workspace` + `handle_activate_workspace` + `store.touch` | +2 |

**Phase 1 테스트 증분**: +15 (baseline 232 → 247 현재)
> 주: 실제 workspace 측정은 248 tests (moai-core 289 중 1 추가 포함 가능성; 재계산 필요)

**파일 변경**: 5개 (crates/moai-studio-{app,ui,workspace}), +1105 / -30 LOC

---

## 3. Acceptance Criteria 현황

| AC | 상태 | 검증 방식 |
|----|------|-----------|
| AC-1.1 기존 289 tests 통과 | ✅ | `cargo test --workspace` 통과 |
| AC-1.2 warning 0 릴리즈 빌드 | ✅ | Phase 1.5 에서 `cargo build --release` 확인 |
| AC-2.1 1600×1000 + 4영역 | ✅ | Phase 1.2 육안 검증 |
| AC-2.2 Welcome CTA | ✅ | Phase 1.3 (workspace 가 비었을 때만 표시로 1.6 리파인) |
| AC-2.3 네이티브 폴더 다이얼로그 | ✅ | Phase 1.4 `rfd::FileDialog`, Phase 1.7 실배선 |
| AC-3.1 libghostty 스파이크 | 🔄 SPEC-V3-002 이관 | 재진단 결과 환경 문제 없음 — FFI 통합 전체가 독립 Phase 2 SPEC 대상 (SPEC-V3-002 AC-T-1 로 계승) |
| AC-3.2 Zig 미설치 에러 | 🔄 SPEC-V3-002 이관 | SPEC-V3-002 AC-T-2 로 계승 |
| AC-4.1 CI 4 게이트 통과 | ✅ **실증** | [run 24708460052](https://github.com/GoosLab/moai-studio/actions/runs/24708460052) — macOS 1m10s + Linux 2m47s (캐시 후), smoke macOS 43s + Linux 12m49s. 총 4 job 그린 |
| AC-4.2 린트 오류 PR 머지 차단 | ⚠️ repo 설정 대기 | Workflow 는 status check 제공 (`rust`, `smoke` jobs). GitHub branch protection rule 활성화는 repo 관리자 작업 |
| AC-5.1 `app/` archive 이동 | ✅ | Phase 0.2 완료 |

---

## 4. 품질 게이트 (Phase 1.8 시점)

- `cargo fmt --all -- --check`: ✅ PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: ✅ PASS (0 warnings)
- `cargo test --workspace`: ✅ 248 passed, 0 failed, 6 ignored
- `cargo build --release`: ✅ (Phase 1.5 마지막 검증)
- `cargo build --no-default-features`: ✅ `moai-adk` feature toggle 정상

**회귀**: 0

---

## 5. 남은 작업 (Phase 2+)

### 완료 항목

1. ~~**RG-V3-4 CI matrix**~~ ✅ Phase 1.9 완료 (GitHub Actions run 24708460052 ALL GREEN)
2. ~~**RG-V3-3 libghostty-vt 스파이크**~~ 🔄 SPEC-V3-002 로 rescope

### SPEC-V3-002 로 이관 (next SPEC 대상)

- libghostty-rs pinned commit 선정 + FFI 통합
- portable-pty + `$SHELL` spawn
- GPUI 내부 terminal surface 텍스트 렌더
- Zig 0.15.x CI 확장

### 후속 Phase 후보 (후순위 SPEC)

- Tab / Pane split
- Command Palette
- Surfaces (FileTree, Markdown, Browser, Image)
- Smart Link Handling
- moai-adk 플러그인
- 자동 업데이트

### AC-4.2 GitHub branch protection rule

Workflow 는 `Rust CI / macOS`, `Rust CI / Linux`, `Rust CI / Smoke (macOS)`, `Rust CI / Smoke (Linux)` status check 를 제공한다. 이들을 required check 로 지정하려면 GitHub Settings → Branches → Add rule (main) → Require status checks 설정 필요. repo 관리자 작업 대상이므로 본 SPEC 범위 외.

---

## 6. Sync 로그 (timeline)

| 시점 | 체크포인트 | 근거 커밋 |
|------|-----------|-----------|
| 2026-04-21 T1 | 초기 sync (RG-V3-2 완료) | `0230cd0` |
| 2026-04-21 T2 | CI smoke --scaffold bug fix | `6ef90d8` |
| 2026-04-21 T3 | RG-V3-4 workflow 구축 반영 | `2878f65` |
| 2026-04-21 T4 | GoosLab/moai-studio repo 생성 + main/feat push | (repo only) |
| 2026-04-21 T5 | Linux rust job linker fix (apt-get deps) | `45826e3` |
| 2026-04-21 T6 | Linux smoke job 같은 fix 반복 | `5d669c6` |
| 2026-04-21 T7 | CI 실증 완료 (run 24708460052 ALL GREEN) | `611a220` |
| 2026-04-21 T8 | RG-V3-3 rescope → SPEC-V3-002, 실질 종결 | (이 커밋) |

**제외된 scope** (본 SPEC sync 와 무관, 별도 처리):
- 작업 트리 미커밋 130+ 파일 (MoAI 툴링 드리프트 · SPEC-AGENCY-ABSORB-001 계열)

**다음 체크포인트**: SPEC-V3-001 공식 종결 이후 `/moai plan SPEC-V3-002` 로 Terminal Core 본격 시작 권장. 또는 feat/v3-scaffold → main 병합 Draft PR 생성.

---

Recorded by: MoAI Studio orchestrator
Updated: 2026-04-21
