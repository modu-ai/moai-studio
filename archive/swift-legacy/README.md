# Swift Legacy Archive — MoAI Studio v1/v2

이 디렉토리는 **MoAI Studio v1.0 ~ v2.5 (Swift + AppKit + SwiftUI + Ghostty xcframework) 의 소스 트리** 를 보존합니다.

## 아카이브 사유 (2026-04-21)

MoAI Studio v3 에서 아키텍처 전면 재설계를 결정했습니다:

- **이전 (v1/v2)**: Swift + AppKit + SwiftUI + Ghostty xcframework (macOS 전용)
- **현재 (v3)**: Rust + GPUI + libghostty-vt (macOS/Linux/Windows 네이티브 크로스플랫폼)

자세한 결정 근거:
- `.moai/design/master-plan.md` (9 핵심 결정 종합)
- `.moai/design/research.md` (경쟁 분석 — cmux, Claude Code Desktop, Warp, Wave, Zed, Raycast 등)
- `.moai/design/spec.md` (v3 25 기능 Tier 구조)
- `.moai/design/archive/tb-vs-tc-report.md` (터미널 라이브러리 비교)

## 이 디렉토리의 역할

**참조용 보존 (Reference only)**:
- v1/v2 구현 패턴 (ActivePaneProvider, GhosttyHost, CommandRegistry 등) 을 v3 Rust 구현 시 설계 참조
- 디자인 결정 감사 추적 (git history)
- 필요 시 macOS 특화 기능을 v3 에 포트할 때 base 로 활용

**비-가동 (NOT BUILT)**:
- 이 디렉토리의 Xcode 프로젝트는 v3 빌드 파이프라인에 포함되지 않음
- CI 에서 빌드/테스트 대상 아님
- Swift 코드 수정 금지 (archive 만)

## 산출물 요약 (아카이브 시점)

| 지표 | 값 |
|------|-----|
| Swift 테스트 | 130 PASS |
| Swift LOC | ~2,500 줄 |
| 구현 SPEC | M0-001, M1-001, M2-001, M2-002 (M2.5 Polish) |
| 핵심 모듈 | App, Shell (RootSplitView, MainWindow, Sidebar, Splits, Tabs, Command, Environments), Surfaces (Terminal, Markdown, Image, Browser, FileTree), ViewModels, Bridge (swift-bridge FFI) |
| 외부 의존 | Ghostty xcframework, swift-bridge FFI, SwiftUI |

## 복원 절차 (if needed)

만약 v3 전환 실패 시 또는 macOS 전용 재개 시:

```bash
# archive 에서 복원 (참조만, 직접 복원 비권장)
git log --all --oneline -- archive/swift-legacy/
# v2 완결 commit: ec19d9b docs(sync): SPEC-M2-002 M2.5 Full GO
```

## 관련 Git Tags/Commits

- `fa274a2` — M2 최종 GO
- `544330d` — SPEC-M2-001 C-1 해소
- `899c635` — SPEC-M2-002 M2.5 Polish (마지막 Swift 구현)
- `ec19d9b` — SPEC-M2-002 문서 동기화
- `fadb67b` — v3 재설계 확정 (디자인 단계, 아직 Swift)
- (이 커밋) — archive 이동 (v3 Phase 0)

---

**Never mix Swift legacy with v3 Rust code. Reference only.**
