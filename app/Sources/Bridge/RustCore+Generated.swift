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

    // ── Pane FFI (MS-2) ─────────────────────────────────────────────────────
    // @MX:ANCHOR: [AUTO] Swift 측 pane CRUD FFI 프로토콜 (fan_in>=3: PaneTreeModel, MockRustCoreBridge, RustCoreBridge)
    // @MX:REASON: [AUTO] MS-2 T-039 PaneTreeModel 이 이 메서드들을 통해 Rust pane DB 를 조작함
    func listPanesJson(workspaceId: Int64) -> String
    func createPane(workspaceId: Int64, parentId: Int64, split: String, ratio: Double) -> Int64
    func updatePaneRatio(paneId: Int64, ratio: Double) -> Bool
    func deletePane(paneId: Int64) -> Bool

    // ── Surface FFI (MS-2/3) ─────────────────────────────────────────────────
    func listSurfacesJson(paneId: Int64) -> String
    func createSurface(paneId: Int64, kind: String, stateJson: String, tabOrder: Int64) -> Int64
    func updateSurfaceTabOrder(surfaceId: Int64, tabOrder: Int64) -> Bool
    func deleteSurface(surfaceId: Int64) -> Bool

    // ── Workspace → DB id 변환 ───────────────────────────────────────────────
    func getWorkspaceDbId(workspaceUuid: String) -> Int64

    // ── FileTree FFI (MS-4) ─────────────────────────────────────────────────
    // @MX:ANCHOR: [AUTO] FileTree 디렉토리 리스팅 + git status FFI 프로토콜 (fan_in>=3)
    // @MX:REASON: [AUTO] FileTreeViewModel, MockRustCoreBridge, RustCoreBridge 세 경로에서 참조
    func listDirectoryJson(workspacePath: String, subpath: String) -> String
    func gitStatusMapJson(workspacePath: String) -> String
}

// @MX:WARN: [AUTO] swift-bridge 0.1 struct_repr 의 Vectorizable 미생성 우회 — stub 구현.
// @MX:REASON: [AUTO] 생성 코드의 `func list_workspaces() -> RustVec<WorkspaceInfo>` 컴파일 위해 필요.
//              RustCoreBridge.listWorkspaces() 가 이 벡터를 실제로 소비하지 않으므로 fatalError trap 안전.
//              C-5 (swift-bridge Vectorizable 지원 버전 출시) 시 제거. M2 기술부채 참조.
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

// @MX:WARN: [AUTO] PaneInfo Vectorizable stub — swift-bridge 0.1 한계 우회 (C-5 기술부채).
// @MX:REASON: [AUTO] list_panes() -> Vec<PaneInfo> FFI 컴파일 위해 필요. 실제 벡터 소비는
//              list_panes_json() JSON 경로로 대체. C-5 해소 시 제거.
extension PaneInfo: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer { fatalError("PaneInfo vec not bridged (C-5 tech debt)") }
    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {}
    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: PaneInfo) {}
    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<PaneInfo> { nil }
    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PaneInfo> { nil }
    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PaneInfo> { nil }
    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<PaneInfo> {
        UnsafePointer<PaneInfo>(bitPattern: 0x1)!
    }
    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt { 0 }
}

// @MX:WARN: [AUTO] SurfaceInfo Vectorizable stub — swift-bridge 0.1 한계 우회 (C-5 기술부채).
// @MX:REASON: [AUTO] list_surfaces() -> Vec<SurfaceInfo> FFI 컴파일 위해 필요. 실제 벡터 소비는
//              list_surfaces_json() JSON 경로로 대체. C-5 해소 시 제거.
extension SurfaceInfo: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer { fatalError("SurfaceInfo vec not bridged (C-5 tech debt)") }
    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {}
    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: SurfaceInfo) {}
    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<SurfaceInfo> { nil }
    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<SurfaceInfo> { nil }
    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<SurfaceInfo> { nil }
    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<SurfaceInfo> {
        UnsafePointer<SurfaceInfo>(bitPattern: 0x1)!
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

    // ── Pane FFI (MS-2) ─────────────────────────────────────────────────────

    public func listPanesJson(workspaceId: Int64) -> String {
        core.list_panes_json(workspaceId).toString()
    }

    public func createPane(workspaceId: Int64, parentId: Int64, split: String, ratio: Double) -> Int64 {
        core.create_pane(workspaceId, parentId, split, ratio)
    }

    public func updatePaneRatio(paneId: Int64, ratio: Double) -> Bool {
        core.update_pane_ratio(paneId, ratio)
    }

    public func deletePane(paneId: Int64) -> Bool {
        core.delete_pane(paneId)
    }

    // ── Surface FFI (MS-2/3) ─────────────────────────────────────────────────

    public func listSurfacesJson(paneId: Int64) -> String {
        core.list_surfaces_json(paneId).toString()
    }

    public func createSurface(paneId: Int64, kind: String, stateJson: String, tabOrder: Int64) -> Int64 {
        core.create_surface(paneId, kind, stateJson, tabOrder)
    }

    public func updateSurfaceTabOrder(surfaceId: Int64, tabOrder: Int64) -> Bool {
        core.update_surface_tab_order(surfaceId, tabOrder)
    }

    public func deleteSurface(surfaceId: Int64) -> Bool {
        core.delete_surface(surfaceId)
    }

    // ── Workspace → DB id 변환 ───────────────────────────────────────────────

    public func getWorkspaceDbId(workspaceUuid: String) -> Int64 {
        core.get_workspace_db_id(workspaceUuid)
    }

    // ── FileTree FFI (MS-4) ─────────────────────────────────────────────────

    public func listDirectoryJson(workspacePath: String, subpath: String) -> String {
        core.list_directory_json(workspacePath, subpath).toString()
    }

    public func gitStatusMapJson(workspacePath: String) -> String {
        core.git_status_map_json(workspacePath).toString()
    }
}
#endif
