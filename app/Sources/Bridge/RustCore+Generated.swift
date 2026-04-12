//
//  RustCore+Generated.swift
//  swift-bridge 로 생성된 `RustCore` 에 Swift 친화 API 를 얹는 확장 레이어.
//
//  @MX:ANCHOR: Swift → Rust FFI 경계의 유일한 호출 지점
//  @MX:REASON: 생성 코드가 반환하는 RustString/RustVec 를 Swift 네이티브 타입으로 변환하여,
//              상위 ViewModel 이 FFI 세부를 인지하지 않도록 보호 (fan_in>=4).
//

import Foundation

/// 경량 스냅샷 — Rust 의 `WorkspaceInfo` 를 Swift 네이티브 타입으로 미러링.
public struct WorkspaceSnapshot: Identifiable, Hashable, Sendable {
    public let id: String
    public let name: String
    public let status: WorkspaceStatus

    public init(id: String, name: String, status: WorkspaceStatus) {
        self.id = id
        self.name = name
        self.status = status
    }
}

/// 6-state workspace lifecycle (moai-store `state.rs` 와 1:1 매핑).
public enum WorkspaceStatus: String, Sendable, CaseIterable {
    case created = "Created"
    case starting = "Starting"
    case running = "Running"
    case paused = "Paused"
    case error = "Error"
    case deleted = "Deleted"

    public init(rawString: String) {
        // Rust 측에서 대소문자 변형으로 올 가능성 대비.
        let normalized = rawString.lowercased()
        switch normalized {
        case "created": self = .created
        case "starting": self = .starting
        case "running": self = .running
        case "paused": self = .paused
        case "error": self = .error
        case "deleted": self = .deleted
        default: self = .created
        }
    }
}

/// FFI 호출 결과를 추상화 — 테스트용 mock 과 실제 RustCore 양쪽을 동일 프로토콜로 다룬다.
@MainActor
public protocol RustCoreBridging: AnyObject {
    func version() -> String
    func listWorkspaces() -> [WorkspaceSnapshot]
    func createWorkspace(name: String, projectPath: String) -> String
    func deleteWorkspace(id: String) -> Bool
    func sendUserMessage(workspaceId: String, message: String) -> Bool
    func subscribeEvents(workspaceId: String) -> Bool
    func pollEvent(workspaceId: String) -> String?
}

// @MX:WARN: swift-bridge 0.1 struct_repr 의 Vectorizable 미생성 우회 — stub 구현.
// @MX:REASON: 생성 코드의 `func list_workspaces() -> RustVec<WorkspaceInfo>` 컴파일 위해 필요.
//              RustCoreBridge.listWorkspaces() 가 이 벡터를 실제로 소비하지 않으므로 fatalError trap 안전.
//              MS-5/6 에서 FFI 를 JSON 반환으로 전환 시 제거.
extension WorkspaceInfo: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer { fatalError("WorkspaceInfo vec not yet bridged (swift-bridge 0.1 limitation)") }
    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {}
    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: WorkspaceInfo) {}
    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<WorkspaceInfo> { nil }
    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WorkspaceInfo> { nil }
    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WorkspaceInfo> { nil }
    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<WorkspaceInfo> {
        UnsafePointer<WorkspaceInfo>(bitPattern: 0x1)!
    }
    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt { 0 }
}

#if canImport(MoaiBridge) || true
// 실제 swift-bridge 생성 코드는 `RustCore` 클래스를 제공한다.
// Package.swift / xcodegen 이 생성 파일을 동일 모듈에 포함시키므로
// `RustCore` 심볼이 사용 가능.

/// 실제 Rust 코어로 요청을 위임하는 브리지.
///
/// - Note: swift-bridge 는 클래스 래퍼를 생성하므로 이 구조체는 얇은 어댑터에 그친다.
/// - Warning: `RustCore` 는 tokio 런타임을 소유하므로 프로세스 당 1개만 생성할 것.
@MainActor
public final class RustCoreBridge: RustCoreBridging {
    private let core: RustCore

    public init(core: RustCore) {
        self.core = core
    }

    /// 기본 이니셜라이저. MainActor 격리 위반 없이 호출 가능.
    public convenience init() {
        self.init(core: RustCore())
    }

    public func version() -> String {
        core.version().toString()
    }

    // @MX:TODO: swift-bridge 0.1 은 struct_repr 벡터의 Vectorizable 미생성 — MS-5/6 에서
    //           list_workspaces 를 JSON String 반환으로 FFI 변경하여 해소.
    //           현재는 Swift 측 캐시 없이 빈 배열 반환 (ViewModel 은 createWorkspace 경로로 동작).
    public func listWorkspaces() -> [WorkspaceSnapshot] {
        []
    }

    public func createWorkspace(name: String, projectPath: String) -> String {
        core.create_workspace(name, projectPath).toString()
    }

    public func deleteWorkspace(id: String) -> Bool {
        core.delete_workspace(id)
    }

    public func sendUserMessage(workspaceId: String, message: String) -> Bool {
        core.send_user_message(workspaceId, message)
    }

    public func subscribeEvents(workspaceId: String) -> Bool {
        core.subscribe_events(workspaceId)
    }

    public func pollEvent(workspaceId: String) -> String? {
        core.poll_event(workspaceId)?.toString()
    }
}
#endif
