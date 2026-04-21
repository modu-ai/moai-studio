//
//  PaneTreeModel.swift
//  pane binary tree 의 Swift 측 모델 (SPEC-M2-001 MS-2 T-039).
//
//  @MX:ANCHOR: [AUTO] PaneSplitView, PaneContainer, RootSplitView 세 경로에서 공유하는 pane 상태 소스
//  @MX:REASON: [AUTO] 모든 pane 트리 변이(split/close/ratio)를 이 모델로 집중하여 FFI 호출 지점 단일화 (fan_in>=3)
//
//  binary tree 불변식:
//  - leaf 노드: childIds 가 비어있음, 실제 surface 를 포함
//  - 비-leaf 노드(horizontal/vertical): childIds 에 정확히 2개 자식
//  - split 연산: leaf → 비-leaf (기존 자식 + 신규 자식)
//  - close 연산: leaf 제거 후 형제 노드를 부모 위치로 승격

import Foundation
import Observation

// MARK: - SplitKind

/// pane 분할 방향 — Rust `SplitKind` 와 1:1 매핑.
// @MX:NOTE: [AUTO] "horizontal" = 좌우 분할 (NSSplitView orientation=.vertical),
//            "vertical" = 상하 분할 (orientation=.horizontal). 이름 혼동 주의.
public enum SplitKind: String, Sendable, Codable {
    case horizontal
    case vertical
    case leaf
}

// MARK: - PaneNode

/// pane binary tree 의 단일 노드.
// @MX:NOTE: [AUTO] parentId==0 은 루트(부모 없음)를 의미. Rust 측 parent_id==0 규약과 동일.
public struct PaneNode: Identifiable, Sendable {
    /// Rust DB rowid
    public let id: Int64
    /// 부모 pane id. 루트이면 nil
    public var parentId: Int64?
    /// 분할 방향
    public var split: SplitKind
    /// 분할 비율 (0.0~1.0). leaf 는 의미 없음
    public var ratio: Double
    /// 자식 pane id 목록. leaf 는 빈 배열, 비-leaf 는 정확히 2개
    public var childIds: [Int64]
}

// MARK: - JSON 파싱용 Decodable 헬퍼

/// Rust `list_panes_json` 응답 파싱용.
private struct PaneInfoDTO: Decodable {
    let id: Int64
    let workspace_id: Int64
    let parent_id: Int64
    let split: String
    let ratio: Double
}

// MARK: - PaneTreeModel

/// pane binary tree 의 @Observable 상태 모델.
///
/// `load()` 로 Rust DB 에서 트리를 불러오고, `splitActive` / `closePane` / `updateRatio` 로
/// 트리를 변이한다. 모든 변이는 FFI 를 통해 즉시 DB 에 영속된다.
@Observable
@MainActor
public final class PaneTreeModel {
    // MARK: - 공개 상태

    /// 전체 노드 맵 (id → PaneNode)
    public private(set) var nodes: [Int64: PaneNode] = [:]
    /// 루트 pane id
    public private(set) var rootId: Int64?
    /// 소속 워크스페이스 DB id
    public let workspaceId: Int64

    // MARK: - 내부

    /// PaneSplitView 의 LeafPaneView 에서 TabBarViewModel 초기화에 사용.
    internal let bridge: RustCoreBridging

    // MARK: - 초기화

    public init(workspaceId: Int64, bridge: RustCoreBridging) {
        self.workspaceId = workspaceId
        self.bridge = bridge
    }

    // MARK: - 트리 로드/영속

    /// Rust DB 에서 pane 트리를 로드한다.
    ///
    /// 저장된 pane 이 없으면 루트 leaf pane 을 자동 생성한다.
    public func load() async {
        let json = bridge.listPanesJson(workspaceId: workspaceId)
        let dtos = parseJson(json)

        if dtos.isEmpty {
            // 최초: 루트 leaf pane 자동 생성
            let newId = bridge.createPane(
                workspaceId: workspaceId,
                parentId: 0,
                split: SplitKind.leaf.rawValue,
                ratio: 0.5
            )
            guard newId > 0 else { return }
            let root = PaneNode(id: newId, parentId: nil, split: .leaf, ratio: 0.5, childIds: [])
            nodes = [newId: root]
            rootId = newId
        } else {
            buildTree(from: dtos)
        }
    }

    // MARK: - 트리 조작

    /// 지정 pane 을 분할한다. 반환값은 새로 생성된 형제 pane id.
    ///
    /// - leaf 를 지정 direction 으로 분할: 기존 leaf 가 비-leaf 로 전환되고
    ///   두 leaf 자식(기존 + 신규)이 생성된다.
    /// - 실패 시 nil 반환 (FFI 오류 등).
    @discardableResult
    public func splitActive(_ paneId: Int64, direction: SplitKind) -> Int64? {
        guard direction != .leaf else { return nil }
        guard var target = nodes[paneId], target.split == .leaf else { return nil }

        // 기존 leaf 를 복제해 첫 번째 자식으로 등록
        let childA = bridge.createPane(
            workspaceId: workspaceId,
            parentId: paneId,
            split: SplitKind.leaf.rawValue,
            ratio: 0.5
        )
        guard childA > 0 else { return nil }

        // 신규 leaf 를 두 번째 자식으로 등록
        let childB = bridge.createPane(
            workspaceId: workspaceId,
            parentId: paneId,
            split: SplitKind.leaf.rawValue,
            ratio: 0.5
        )
        guard childB > 0 else {
            _ = bridge.deletePane(paneId: childA)
            return nil
        }

        // 부모 노드 업데이트 (leaf → 분할 노드)
        _ = bridge.updatePaneRatio(paneId: paneId, ratio: 0.5)
        target.split = direction
        target.ratio = 0.5
        target.childIds = [childA, childB]
        nodes[paneId] = target

        // 자식 노드 추가
        nodes[childA] = PaneNode(id: childA, parentId: paneId, split: .leaf, ratio: 0.5, childIds: [])
        nodes[childB] = PaneNode(id: childB, parentId: paneId, split: .leaf, ratio: 0.5, childIds: [])

        return childB
    }

    /// 지정 leaf pane 을 닫는다.
    ///
    /// - 마지막 pane 이면 false 반환 (닫기 금지).
    /// - 형제 노드가 부모 위치로 승격된다.
    @discardableResult
    public func closePane(_ paneId: Int64) -> Bool {
        // 마지막 pane 보호
        guard nodes.count > 1 else { return false }
        guard let target = nodes[paneId], target.split == .leaf else { return false }

        guard let parentId = target.parentId, let parent = nodes[parentId] else {
            // 루트 leaf — 닫기 불가 (마지막 pane)
            return false
        }

        // 형제 노드 탐색
        let siblingId = parent.childIds.first(where: { $0 != paneId })
        guard let siblingId, var sibling = nodes[siblingId] else { return false }

        // Rust DB 에서 닫는 pane 삭제
        _ = bridge.deletePane(paneId: paneId)

        // 조부모가 있으면 부모 대신 형제를 조부모의 자식으로 연결
        if let grandParentId = parent.parentId, var grandParent = nodes[grandParentId] {
            if let idx = grandParent.childIds.firstIndex(of: parentId) {
                grandParent.childIds[idx] = siblingId
                nodes[grandParentId] = grandParent
            }
            sibling.parentId = grandParentId
        } else {
            // 부모가 루트였으면 형제가 새 루트
            sibling.parentId = nil
            rootId = siblingId
        }

        // 부모 노드와 닫힌 pane 삭제
        _ = bridge.deletePane(paneId: parentId)
        nodes.removeValue(forKey: paneId)
        nodes.removeValue(forKey: parentId)
        nodes[siblingId] = sibling

        return true
    }

    /// pane 의 분할 비율을 업데이트하고 DB 에 영속한다.
    public func updateRatio(_ paneId: Int64, ratio: Double) {
        guard var node = nodes[paneId] else { return }
        // @MX:NOTE: [AUTO] 최소 pane 크기 200pt 는 NSSplitView 레이어에서 보장 — 여기서 ratio 클램프 불필요
        node.ratio = ratio
        nodes[paneId] = node
        _ = bridge.updatePaneRatio(paneId: paneId, ratio: ratio)
    }

    // MARK: - 조회 헬퍼

    /// 루트 PaneNode 를 반환한다.
    public func tree() -> PaneNode? {
        guard let rootId else { return nil }
        return nodes[rootId]
    }

    /// 지정 pane 의 자식 노드 목록을 반환한다.
    public func children(of paneId: Int64) -> [PaneNode] {
        guard let node = nodes[paneId] else { return [] }
        return node.childIds.compactMap { nodes[$0] }
    }

    // MARK: - 내부 파싱

    private func parseJson(_ json: String) -> [PaneInfoDTO] {
        guard let data = json.data(using: .utf8) else { return [] }
        return (try? JSONDecoder().decode([PaneInfoDTO].self, from: data)) ?? []
    }

    private func buildTree(from dtos: [PaneInfoDTO]) {
        var newNodes: [Int64: PaneNode] = [:]

        for dto in dtos {
            let split = SplitKind(rawValue: dto.split) ?? .leaf
            let node = PaneNode(
                id: dto.id,
                parentId: dto.parent_id == 0 ? nil : dto.parent_id,
                split: split,
                ratio: dto.ratio,
                childIds: []
            )
            newNodes[dto.id] = node
        }

        // 자식 목록 구성
        for dto in dtos where dto.parent_id != 0 {
            guard var parent = newNodes[dto.parent_id] else { continue }
            parent.childIds.append(dto.id)
            newNodes[dto.parent_id] = parent
        }

        nodes = newNodes
        // 루트 = parent_id 가 없는 노드
        rootId = dtos.first(where: { $0.parent_id == 0 })?.id
    }
}
