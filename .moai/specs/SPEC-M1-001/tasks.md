# Task Decomposition

SPEC: SPEC-M1-001
Methodology: TDD (RED-GREEN-REFACTOR), --team parallel
Generated: 2026-04-12

## Task Table

| Task ID | Description | Requirement | Dependencies | Planned Files | Owner | Status |
|---------|-------------|-------------|--------------|---------------|-------|--------|
| T-001 | swift-bridge 크레이트 추가 + build.rs 설정 (Rust→Swift 코드 생성 파이프라인 구축) | RG-M1-3 | - | core/crates/moai-ffi/Cargo.toml, core/crates/moai-ffi/build.rs | backend-dev | pending |
| T-002 | swift-bridge 매크로로 RustCore opaque type + new/version 마이그레이션 (C ABI 교체) | RG-M1-3 | T-001 | core/crates/moai-ffi/src/lib.rs, core/crates/moai-ffi/tests/bridge_basic.rs | backend-dev | pending |
| T-003 | WorkspaceInfo swift_repr struct + list_workspaces/create_workspace/delete_workspace FFI 확장 | RG-M1-3, RG-M1-4 | T-002 | core/crates/moai-ffi/src/lib.rs, core/crates/moai-ffi/src/workspace.rs | backend-dev | pending |
| T-004 | send_user_message + subscribe_events async/callback FFI (tokio runtime bridge, <1ms overhead) | RG-M1-3, RG-M1-5 | T-003 | core/crates/moai-ffi/src/lib.rs, core/crates/moai-ffi/src/events.rs, core/crates/moai-ffi/tests/events_stream.rs | backend-dev | pending |
| T-005 | moai-store `workspaces` 테이블 마이그레이션 v2 (name, project_path, worktree_path, status, spec_id, claude_session_id 확장) | RG-M1-4 | - | core/crates/moai-store/migrations/V2__workspaces_expand.sql, core/crates/moai-store/src/workspace.rs, core/crates/moai-store/tests/workspace_crud.rs | backend-dev | pending |
| T-006 | moai-store 6-state machine (Created/Starting/Running/Paused/Error/Deleted) 상태 전환 검증 | RG-M1-4 | T-005 | core/crates/moai-store/src/state.rs, core/crates/moai-store/tests/state_machine.rs | tester | pending |
| T-007 | moai-git workspace별 git worktree create/remove (git2 crate) | RG-M1-4 | - | core/crates/moai-git/src/worktree.rs, core/crates/moai-git/tests/worktree_lifecycle.rs | backend-dev | pending |
| T-008 | moai-fs workspace 경로별 감시 시작/중단 (notify 7.x) + EventBus 발행 | RG-M1-4 | - | core/crates/moai-fs/src/watcher.rs, core/crates/moai-fs/tests/watch_publish.rs | backend-dev | pending |
| T-009 | RootSupervisor에 multi-workspace orchestration (WorkspaceSupervisor spawn/terminate) | RG-M1-4 | T-005, T-006, T-007, T-008 | core/crates/moai-supervisor/src/root.rs, core/crates/moai-supervisor/src/workspace.rs, core/crates/moai-supervisor/tests/multi_workspace.rs | backend-dev | pending |
| T-010 | Workspace 생성 5단계 오케스트레이션 (store→git→fs→claude-host→상태 전환) | RG-M1-4 | T-009 | core/crates/moai-supervisor/src/lifecycle.rs, core/crates/moai-supervisor/tests/lifecycle_orchestration.rs | backend-dev | pending |
| T-011 | 앱 재시작 시 store에서 workspace 복원 (lazy Claude spawn) | RG-M1-4 | T-010 | core/crates/moai-supervisor/src/restore.rs, core/crates/moai-supervisor/tests/restore_from_store.rs | backend-dev | pending |
| T-012 | moai-claude-host workspace별 독립 subprocess spawn (claude --bare 인자 세트) | RG-M1-5 | T-009 | core/crates/moai-claude-host/src/spawn.rs, core/crates/moai-claude-host/tests/spawn_args.rs | backend-dev | pending |
| T-013 | moai-claude-host stdin 사용자 메시지 전송 (SDKUserMessage JSON) + 비정상 종료→Error 상태 전이 | RG-M1-5, RG-M1-4 | T-012 | core/crates/moai-claude-host/src/stdin.rs, core/crates/moai-claude-host/src/monitor.rs, core/crates/moai-claude-host/tests/crash_to_error.rs | backend-dev | pending |
| T-014 | moai-stream-json stdout→SDKMessage 13종 실시간 디코딩 + EventBus 발행 | RG-M1-5 | T-012 | core/crates/moai-stream-json/src/decoder.rs, core/crates/moai-stream-json/tests/decode_13_types.rs | tester | pending |
| T-015 | moai-ide-server workspace별 MCP 서버 인스턴스 (포트 분리) + full round-trip 테스트 | RG-M1-5 | T-009 | core/crates/moai-ide-server/src/instance.rs, core/crates/moai-ide-server/tests/mcp_roundtrip.rs | backend-dev | pending |
| T-016 | moai-hook-http workspace별 endpoint + HTTP POST→EventBus 발행 (<10ms P95) | RG-M1-5 | T-009 | core/crates/moai-hook-http/src/endpoint.rs, core/crates/moai-hook-http/tests/hook_publish.rs | backend-dev | pending |
| T-017 | moai-plugin-installer bundle→~/.claude/plugins/moai-studio@local 복사 + 버전 비교 | RG-M1-7 | - | core/crates/moai-plugin-installer/src/installer.rs, core/crates/moai-plugin-installer/tests/install_copy.rs | backend-dev | pending |
| T-018 | moai-plugin-installer 무결성 검증 (plugin.json, hooks/hooks.json E5 wrapper, mcp-config.json) + 쓰기 권한 오류 처리 | RG-M1-7 | T-017 | core/crates/moai-plugin-installer/src/verify.rs, core/crates/moai-plugin-installer/tests/verify_and_permissions.rs | backend-dev | pending |
| T-019 | Metal Toolchain 환경 검증 스크립트 + GhosttyKit.xcframework 빌드 + Xcode 연결 (실패 시 NSTextView fallback 모드) | RG-M1-2 | - | scripts/build-ghostty-xcframework.sh, scripts/check-metal-toolchain.sh, app/Frameworks/.gitkeep | frontend-dev | pending |
| T-020 | Xcode 프로젝트 생성 (xcodegen spec) — macOS 14+ 타겟, swift-bridge 생성 코드 통합 | RG-M1-1 | T-002 | app/project.yml, app/Sources/App/MoAIStudioApp.swift, app/Sources/Bridge/RustCore+Generated.swift | frontend-dev | pending |
| T-021 | NavigationSplitView 기반 메인 윈도우 + 사이드바 너비 200~400px 범위 드래그 조정 | RG-M1-1, RG-M1-6 | T-020 | app/Sources/Shell/MainWindow.swift, app/Sources/Shell/RootSplitView.swift | frontend-dev | pending |
| T-022 | 윈도우 크기/사이드바 너비 UserDefaults 저장·복원 | RG-M1-1 | T-021 | app/Sources/Shell/WindowStateStore.swift, app/Tests/WindowStateTests.swift | tester | pending |
| T-023 | Sidebar 워크스페이스 목록 뷰 + 상태 아이콘 (Starting 스피너, Running 녹색, Error 빨강, Paused 회색) | RG-M1-6, RG-M1-4 | T-021, T-003 | app/Sources/Shell/Sidebar/WorkspaceListView.swift, app/Sources/Shell/Sidebar/StatusIcon.swift | frontend-dev | pending |
| T-024 | Sidebar "+" 버튼 → 새 워크스페이스 생성 플로우 (이름 입력 시트) + 우클릭 컨텍스트 메뉴 (Rename/Restart/Delete) | RG-M1-6 | T-023 | app/Sources/Shell/Sidebar/NewWorkspaceSheet.swift, app/Sources/Shell/Sidebar/ContextMenu.swift | frontend-dev | pending |
| T-025 | 콘텐츠 영역 workspace 전환 + 빈 상태 환영 메시지 ("Create Workspace" CTA) | RG-M1-6 | T-021 | app/Sources/Shell/Content/ContentArea.swift, app/Sources/Shell/Content/EmptyState.swift | frontend-dev | pending |
| T-026 | GhosttyKit 터미널 surface 래핑 + 초기화 실패 시 "Terminal unavailable" + 재시도 버튼 | RG-M1-2, RG-M1-6 | T-019, T-025 | app/Sources/Shell/Content/TerminalSurface.swift, app/Sources/Shell/Content/TerminalFallback.swift | frontend-dev | pending |
| T-027 | @Observable WorkspaceViewModel — swift-bridge subscribe_events 바인딩 + 사이드바/콘텐츠 상태 동기화 | RG-M1-3, RG-M1-6 | T-004, T-023 | app/Sources/ViewModels/WorkspaceViewModel.swift, app/Tests/WorkspaceViewModelTests.swift | frontend-dev | pending |
| T-028 | E2E 통합 테스트 시나리오 (앱 실행→생성→메시지→응답→전환→삭제→재시작 복원) | RG-M1-5 전체, RG-M1-4 전체 | T-011, T-014, T-016, T-027 | core/crates/moai-integration-tests/tests/e2e_working_shell.rs, app/UITests/E2EWorkingShellTests.swift | tester | pending |
| T-029 | 4개 동시 워크스페이스 stress + 메모리 <400MB + 비기능 목표 검증 (cold start <1s, FFI <1ms, hook <10ms P95, MCP <50ms) | NFR §5 | T-028 | core/crates/moai-integration-tests/tests/nfr_stress.rs, .moai/specs/SPEC-M1-001/nfr-report.md | quality | pending |
| T-030 | TRUST 5 + MX 태그 감사 + cargo check 0 errors/0 warnings + Xcode 빌드 0 errors + M1 Go/No-Go 보고서 | Quality Gates | T-029 | .moai/specs/SPEC-M1-001/m1-completion-report.md, .moai/reports/m1-trust5-audit.md | quality | pending |

## File Ownership Boundaries

- **backend-dev**: `core/crates/**` (Rust core), `scripts/*.sh` (build scripts except Metal)
- **frontend-dev**: `app/**` (Xcode + SwiftUI), Metal/GhosttyKit 빌드 스크립트
- **tester**: `**/tests/**`, `app/Tests/**`, `app/UITests/**`, stream-json decoder tests, state machine tests
- **quality**: `.moai/specs/SPEC-M1-001/*.md` reports, `.moai/reports/**`, NFR/TRUST5 감사
