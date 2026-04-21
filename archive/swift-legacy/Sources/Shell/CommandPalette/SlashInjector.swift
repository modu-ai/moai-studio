//
//  SlashInjector.swift
//  /moai 슬래시 명령어를 Claude subprocess 로 주입 (SPEC-M2-001 MS-6 T-071).
//
//  @MX:NOTE: [AUTO] 슬래시 주입 라우팅:
//            CommandPalette 에서 /moai 명령어 선택 → SlashInjector.inject(text:)
//            → WorkspaceViewModel.selectedWorkspaceId → send_user_message FFI
//            → 활성 워크스페이스의 Claude subprocess 로 전달

import Foundation

// MARK: - SlashInjecting

/// 슬래시 텍스트를 활성 워크스페이스의 Claude subprocess 로 전송하는 프로토콜.
///
/// MockSlashInjector 를 이용한 테스트 분리를 위해 프로토콜로 추상화.
@MainActor
public protocol SlashInjecting: AnyObject {
    func inject(_ text: String)
}

// MARK: - SlashInjector

/// 실제 SlashInjecting 구현 — WorkspaceViewModel 과 RustCoreBridging 을 사용.
@MainActor
public final class SlashInjector: SlashInjecting {
    private let bridge: RustCoreBridging
    private let workspaceVM: WorkspaceViewModel

    public init(bridge: RustCoreBridging, workspaceVM: WorkspaceViewModel) {
        self.bridge = bridge
        self.workspaceVM = workspaceVM
    }

    /// 텍스트를 활성 워크스페이스의 Claude subprocess 로 전송한다.
    ///
    /// - Parameter text: 전송할 슬래시 명령어 텍스트 (예: "/moai plan")
    public func inject(_ text: String) {
        guard let workspaceId = workspaceVM.selectedWorkspaceId else {
            // @MX:NOTE: [AUTO] 선택된 워크스페이스 없음 — 무시 (MS-7 에서 경고 토스트 추가 예정)
            return
        }
        _ = bridge.sendUserMessage(workspaceId: workspaceId, message: text)
    }
}
