# M2 수동 로컬 검증 체크리스트

**대상**: SPEC-M2-001 (M2 Viewers, v1.2.0, completed 2026-04-15) 산출물
**목적**: Apple Developer 가입 전, 로컬 환경에서 M2 기능이 실제로 동작하는지 육안 + 키보드로 검증
**소요**: 약 20~30 분
**전제**: `xcodebuild` 실행 가능, Xcode 15.0+, macOS 14.0+ (Sonoma 이상 권장)

---

## 0. 준비 (빌드 + 실행)

### 0-1. Rust / Swift 유닛 테스트 재확인 (선택)

```bash
# Rust 워크스페이스 (233 tests 예상)
cargo test --workspace

# Swift 유닛 (106 tests 예상, UITest 제외)
xcodebuild test \
  -project app/MoAIStudio.xcodeproj \
  -scheme MoAIStudio \
  -destination 'platform=macOS' \
  -skip-testing:MoAIStudioUITests
```

### 0-2. Debug 빌드 + 실행

```bash
# 1) 빌드
xcodebuild \
  -project app/MoAIStudio.xcodeproj \
  -scheme MoAIStudio \
  -configuration Debug \
  build

# 2) 앱 경로 확인
DERIVED="$(xcodebuild -project app/MoAIStudio.xcodeproj -scheme MoAIStudio -showBuildSettings | awk -F= '/ BUILT_PRODUCTS_DIR / {gsub(/^[ \t]+|[ \t]+$/, "", $2); print $2; exit}')"
echo "빌드 산출물: $DERIVED"

# 3) 실행
open "$DERIVED/MoAI Studio.app"
```

앱이 실행되고 사이드바 + 메인 콘텐츠 영역이 표시되면 0 단계 성공.

---

## 1. 핵심 시나리오 체크리스트 (16개)

각 항목: `[ ] PASS` / `[X] FAIL` (실패 시 하단 이슈 기록란에 증상 기재)

### Section A. Shell 기본 (M1 Carry-over)

| # | 시나리오 | 조작 | 확인 | 검증 대상 |
|---|---------|------|------|-----------|
| 1 | 앱 기본 레이아웃 | 앱 실행 | 사이드바(좌) + 콘텐츠 영역(우) 분리 표시 | RG-M1-1, RG-M1-6 |
| 2 | 워크스페이스 생성 | 사이드바 "+" 버튼 클릭 → 이름 입력 | 터미널 placeholder(또는 GhosttyHost) 가 콘텐츠에 표시 | RG-M1-4 |

### Section B. Pane Splitting (RG-M2-1)

| # | 시나리오 | 조작 | 확인 | 검증 대상 |
|---|---------|------|------|-----------|
| 3 | 수평 분할 | 콘텐츠 영역 포커스 → `Cmd+\` | 좌/우 pane 으로 분할, 경계선 표시 | AC-1.1 |
| 4 | 수직 분할 | 좌측 pane 포커스 → `Cmd+Shift+\` | 상/하 분할 | AC-1.2 |
| 5 | 리사이즈 제약 | pane 경계 드래그 | 한쪽이 200pt 이하로 줄어들지 않음 | AC-1.3 |
| 6 | pane 닫기 + 마지막 보호 | 각 pane 포커스 → `Cmd+Shift+W` 반복 | 마지막 pane 은 닫히지 않음 | AC-1.4, AC-1.5 |
| 7 | 레이아웃 영속 | pane 여러 개 연 뒤 앱 종료 → 재실행 | 동일한 pane 트리 복원 | AC-1.6 |

### Section C. Tab UI (RG-M2-2)

| # | 시나리오 | 조작 | 확인 | 검증 대상 |
|---|---------|------|------|-----------|
| 8 | 탭 생성 + reorder | `Cmd+T` 로 탭 2~3 개 → 드래그 | 새 탭 생성 및 순서 변경 | RG-M2-2 |
| 9 | 탭 닫기 | 포커스된 탭에서 `Cmd+W` | 해당 탭만 닫힘 | AC-2.2 |

### Section D. Command Palette (RG-M2-3) — **현재 제한**

| # | 시나리오 | 조작 | 확인 | 검증 대상 |
|---|---------|------|------|-----------|
| 10 | 팔레트 열기 + fuzzy search | `Cmd+K` → `moai` 입력 | `/moai plan`, `/moai run` 등 명령 목록이 fuzzy match 로 표시 | RG-M2-3 |
| 11 | /moai 명령 전달 | `/moai plan` 선택 → Enter | 명령이 Claude subprocess 로 전달됨 (또는 터미널 출력) | AC-3.3 |

> ⚠️ **알려진 제한**: "Open FileTree", "Open Markdown", "Open Browser", "Split Pane Horizontally/Vertically" 명령은 현재 **no-op** 상태 (SPEC-M2-002 에서 해소 예정). 팔레트 UI 에 노출은 되지만 클릭해도 surface 생성/분할이 일어나지 않음.

### Section E. Surfaces (RG-M2-4 / 5 / 6 / 7) — **현재 제한**

| # | 시나리오 | 조작 | 확인 | 검증 대상 |
|---|---------|------|------|-----------|
| 12 | FileTree surface | 사이드바 "+" → FileTree surface 선택 (존재 시) | 디렉토리 트리 + git status 색상 표시 | RG-M2-4 |
| 13 | Markdown surface | FileTree 에서 `.md` 파일 더블클릭 | 새 탭에 Markdown 렌더 | AC-4.4 |
| 14 | SPEC/KaTeX/Mermaid 렌더 | SPEC-M2-001/spec.md 열기 (온라인 환경) | EARS 포매팅 + 수식/다이어그램 렌더 | RG-M2-5 |
| 15 | Image surface | 이미지 파일(.png/.jpg) 열기 | zoom/pan 가능 | RG-M2-6 |
| 16 | Browser surface | Browser surface 열기 (localhost:3000 로컬 서버 가동 중이라면) | 자동 감지 + 렌더 | RG-M2-7 |

> ⚠️ **알려진 제한**:
> - **12 (FileTree 경로)**: `resolveWorkspacePath()` 가 홈 디렉토리 폴백 → FileTree 가 엉뚱한 경로를 표시할 수 있음 (SPEC-M2-003 P-8 에서 해소).
> - **12 (FileTree depth)**: 루트 1 레벨만 표시됨. 하위 디렉토리 expand 불가 (SPEC-M2-003 P-6 에서 해소).
> - **13/15 (재시작 복원)**: 탭의 파일 경로가 재시작 시 소실됨 (SPEC-M2-003 P-5 에서 해소).
> - **14 (오프라인)**: KaTeX/Mermaid 는 CDN 사용 중 → 인터넷 연결 필요 (SPEC-M2-003 후속 또는 별도 SPEC).
> - **16 (URL 영속)**: BrowserSurface URL 은 재시작 시 기본값 리셋 (SPEC-M2-003 P-7 에서 해소).

---

## 2. 이슈 기록란 (복사해서 사용)

```
[ISSUE-#]
시나리오 번호: #N
증상:
  -
재현 절차:
  1.
  2.
기대 동작:
예상 원인 / 관련 파일:
참조 SPEC / P-번호:
```

---

## 3. 종합 판정

- [ ] **GO (M3 진입 가능)**: Section A~C 100% PASS + Section D/E 는 "알려진 제한" 범위 내 동작 확인
- [ ] **HOLD (M2.5 먼저 필요)**: Section A~C 에서 새로운 regression 발견
- [ ] **BLOCK (SPEC 수정 필요)**: 기존 수용 기준 위반 (예: pane split 이 작동 안 함)

---

## 4. 다음 단계 가이드

| 판정 | 다음 행동 |
|------|----------|
| GO | SPEC-M2-002 (M2.5 Polish) `/moai run` 착수 → SPEC-M2-003 (Persistence) 병렬 가능 |
| HOLD | 이슈 기록 → SPEC-M2-002 로 이월 또는 M2-001 hotfix SPEC 신규 작성 |
| BLOCK | 즉시 MoAI 에게 보고 → 필요 시 SPEC-M2-001 재오픈 |

---

## 5. 관련 SPEC / 참조

- [SPEC-M2-001 (M2 Viewers, completed)](.moai/specs/SPEC-M2-001/spec.md)
- [SPEC-M2-001 완료 보고서](.moai/specs/SPEC-M2-001/m2-completion-report.md)
- [SPEC-M2-002 (M2.5 Polish, draft)](.moai/specs/SPEC-M2-002/spec.md) ← 12 개 placeholder 중 P-1~P-4 해소
- [SPEC-M2-003 (Surface State Persistence, draft)](.moai/specs/SPEC-M2-003/spec.md) ← P-5~P-8 해소
- [SPEC-M3-001 (Code Viewer, draft)](.moai/specs/SPEC-M3-001/spec.md) ← P-1 완료 후 착수

---

**작성**: 2026-04-16 · MoAI 오케스트레이터
**갱신 규칙**: 이 파일은 M2/M2.5 기간 동안 활성. M3 착수 시 `docs/QA/manual-verification-M3.md` 로 승계.
