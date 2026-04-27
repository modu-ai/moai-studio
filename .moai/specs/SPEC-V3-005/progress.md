# SPEC-V3-005 Progress

**Started**: 2026-04-27

## USER-DECISION 게이트 결과

### USER-DECISION-A: moai-fs API shape
- **선택**: (a) WorkspaceWatcher helper 추가 (권장)
- **결과**: WorkspaceWatcher helper를 `crates/moai-fs/src/workspace_watcher.rs`에 추가
- **영향**: SPEC-V3-008과 공유 가능

### USER-DECISION-B: gpui test-support
- **선택**: (a) gpui test-support 추가 (권장)
- **Spike 0 결과**: 빌드 성공 (9.12s)
- **결과**: GPUI 환경 e2e 검증 가능
- **기록**: `gpui = { version = "0.2", features = ["test-support"] }` 이미 활성화됨

### USER-DECISION-C: delete trash policy
- **선택**: (a) trash crate - OS 휴지통 (권장)
- **Spike 1 결과**: 빌드 성공 (0.97s)
- **결과**: 모든 delete가 OS 휴지통 송부
- **기록**: `trash = "5"` 이미 추가됨

## Milestone 진행 상태

- [x] Spike 0 + Spike 1 완료
- [ ] MS-1: FsNode + ChildState + lazy load + path normalize + GPUI render
- [ ] MS-2: WorkspaceWatcher + debounce + diff apply + rename matching
- [ ] MS-3: git status + context menu + DnD + search + e2e

## 다음 단계

- Phase 1.5: Task Decomposition (T1~T13)
- Phase 1.6: Acceptance Criteria Initialization
- Phase 1.7: File Structure Scaffolding
- Phase 1.8: Pre-Implementation MX Context Scan
- Phase 2: TDD Implementation
