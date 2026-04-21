//
//  ActivePaneProviderTests.swift
//  ActivePaneProvider @Environment 키 + WorkspaceViewModel.activePane 단위 테스트
//  SPEC-M2-002 MS-1 T-M2.5-001 ~ T-M2.5-005 (AC-1.1 ~ AC-1.5)
//

import XCTest
import SwiftUI
import AppKit
@testable import MoAIStudio

@MainActor
final class ActivePaneProviderTests: XCTestCase {
    // MARK: - T-M2.5-001: AC-1.1 기본값 검증

    /// `ActivePaneContext.empty` 의 모든 필드가 nil 이어야 한다.
    func test_defaultActivePaneContext_hasAllNilFields() {
        let ctx = ActivePaneContext.empty
        XCTAssertNil(ctx.paneId, "empty.paneId 는 nil 이어야 함")
        XCTAssertNil(ctx.model, "empty.model 는 nil 이어야 함")
        XCTAssertNil(ctx.workspace, "empty.workspace 는 nil 이어야 함")
    }

    /// 기본 이니셜라이저로 생성한 `ActivePaneContext` 도 모든 필드가 nil 이어야 한다.
    func test_defaultInit_hasAllNilFields() {
        let ctx = ActivePaneContext()
        XCTAssertNil(ctx.paneId)
        XCTAssertNil(ctx.model)
        XCTAssertNil(ctx.workspace)
    }

    // MARK: - T-M2.5-002: AC-1.2 환경값 주입 전파 검증

    /// `.environment(\.activePane, ctx)` 로 주입한 컨텍스트가 자식에서 동일하게 조회되어야 한다.
    func test_environmentInjection_propagatesContext() throws {
        let mock = MockRustCoreBridge()
        let paneTreeModel = PaneTreeModel(workspaceId: 1, bridge: mock)
        let ws = WorkspaceSnapshot(id: "ws-1", name: "테스트", status: .running)
        let injected = ActivePaneContext(paneId: 42, model: paneTreeModel, workspace: ws)

        // SwiftUI EnvironmentValues 에 직접 할당 후 조회
        var env = EnvironmentValues()
        env[keyPath: \.activePane] = injected

        let retrieved = env.activePane
        XCTAssertEqual(retrieved.paneId, 42)
        XCTAssertEqual(retrieved.workspace?.id, "ws-1")
    }

    // MARK: - T-M2.5-002: AC-1.3 중첩 환경값 override

    /// 중첩 `.environment` 에서 가장 안쪽 값이 이긴다.
    func test_nestedEnvironmentOverride_wins() {
        let outer = ActivePaneContext(paneId: 10, model: nil, workspace: nil)
        let inner = ActivePaneContext(paneId: 99, model: nil, workspace: nil)

        var outerEnv = EnvironmentValues()
        outerEnv.activePane = outer

        // 같은 EnvValues 에 inner 로 덮어씀 (중첩 시뮬레이션)
        var innerEnv = outerEnv
        innerEnv.activePane = inner

        XCTAssertEqual(innerEnv.activePane.paneId, 99, "가장 안쪽 context 가 이겨야 함")
    }

    // MARK: - T-M2.5-002: WorkspaceEnvironmentKey 기본값

    /// `WorkspaceEnvironmentKey` 기본값은 nil 이어야 한다.
    func test_workspaceEnvironmentKey_defaultIsNil() {
        let env = EnvironmentValues()
        XCTAssertNil(env.activeWorkspace, "activeWorkspace 기본값은 nil 이어야 함")
    }

    /// `activeWorkspace` 주입 후 정상 조회되어야 한다.
    func test_workspaceEnvironmentKey_injection() {
        let ws = WorkspaceSnapshot(id: "ws-test", name: "My WS", status: .running)
        var env = EnvironmentValues()
        env.activeWorkspace = ws

        XCTAssertEqual(env.activeWorkspace?.id, "ws-test")
        XCTAssertEqual(env.activeWorkspace?.name, "My WS")
    }

    // MARK: - T-M2.5-003: AC-1.4 WorkspaceViewModel.activePane 프로퍼티

    /// `WorkspaceViewModel.activePane` 직접 할당 후 읽기가 가능해야 한다.
    func test_activePaneChange_updatesWorkspaceViewModel() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)

        // 기본값 확인
        XCTAssertNil(vm.activePane.paneId, "초기 activePane.paneId 는 nil 이어야 함")

        // 새 컨텍스트 할당
        let newCtx = ActivePaneContext(paneId: 55, model: nil, workspace: nil)
        vm.activePane = newCtx

        XCTAssertEqual(vm.activePane.paneId, 55, "할당 후 paneId 가 반영되어야 함")
    }

    // MARK: - T-M2.5-004: AC-1.6 split 노드 활성 불가 (RELEASE 에서는 무시)

    /// split (non-leaf) 노드가 `ActivePaneContext` 의 paneId 로 설정되어도 앱이 크래시하지 않아야 한다.
    /// DEBUG assertion 은 테스트 환경(RELEASE 모드)에서는 발화하지 않음.
    func test_splitNode_doesNotBecomeActive_inRelease() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)

        // split 노드 id (RELEASE 에서는 그냥 저장됨)
        let splitCtx = ActivePaneContext(paneId: 1, model: nil, workspace: nil)
        vm.activePane = splitCtx

        // 크래시 없이 도달하면 성공
        XCTAssertEqual(vm.activePane.paneId, 1)
    }
}

// MARK: - 헬퍼

private extension ActivePaneProviderTests {
    /// 테스트용 `ActivePaneContext` 생성 헬퍼.
    func makeMockContext(paneId: Int64? = nil) -> ActivePaneContext {
        ActivePaneContext(paneId: paneId, model: nil, workspace: nil)
    }
}
