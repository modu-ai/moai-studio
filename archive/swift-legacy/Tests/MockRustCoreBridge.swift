//
//  MockRustCoreBridge.swift
//  WorkspaceViewModel / 기타 ViewModel 테스트용 in-memory 브리지.
//

import Foundation
@testable import MoAIStudio

@MainActor
final class MockRustCoreBridge: RustCoreBridging {
    var workspaces: [WorkspaceSnapshot] = []
    var eventQueues: [String: [String]] = [:]
    var subscribedIds: Set<String> = []
    var messagesSent: [(String, String)] = []
    private var idCounter: Int = 0

    func version() -> String { "1.0.0-mock" }

    func listWorkspaces() -> [WorkspaceSnapshot] { workspaces }

    func createWorkspace(name: String, projectPath: String) -> String {
        idCounter += 1
        let id = "mock-\(idCounter)"
        workspaces.append(WorkspaceSnapshot(id: id, name: name, status: .starting))
        return id
    }

    func deleteWorkspace(id: String) -> Bool {
        guard let idx = workspaces.firstIndex(where: { $0.id == id }) else { return false }
        workspaces.remove(at: idx)
        eventQueues.removeValue(forKey: id)
        subscribedIds.remove(id)
        return true
    }

    func sendUserMessage(workspaceId: String, message: String) -> Bool {
        messagesSent.append((workspaceId, message))
        return true
    }

    func subscribeEvents(workspaceId: String) -> Bool {
        subscribedIds.insert(workspaceId)
        return true
    }

    func pollEvent(workspaceId: String) -> String? {
        guard var queue = eventQueues[workspaceId], !queue.isEmpty else { return nil }
        let next = queue.removeFirst()
        eventQueues[workspaceId] = queue
        return next
    }

    // ── Pane FFI (MS-2) ─────────────────────────────────────────────────────
    var panes: [Int64: MockPaneRecord] = [:]
    var paneIdCounter: Int64 = 0

    func listPanesJson(workspaceId: Int64) -> String {
        let rows = panes.values.filter { $0.workspaceId == workspaceId }
        let arr = rows.map { r in
            """
            {"id":\(r.id),"workspace_id":\(r.workspaceId),"parent_id":\(r.parentId),"split":"\(r.split)","ratio":\(r.ratio)}
            """
        }
        return "[\(arr.joined(separator: ","))]"
    }

    func createPane(workspaceId: Int64, parentId: Int64, split: String, ratio: Double) -> Int64 {
        paneIdCounter += 1
        panes[paneIdCounter] = MockPaneRecord(
            id: paneIdCounter, workspaceId: workspaceId,
            parentId: parentId, split: split, ratio: ratio
        )
        return paneIdCounter
    }

    func updatePaneRatio(paneId: Int64, ratio: Double) -> Bool {
        guard var p = panes[paneId] else { return false }
        p.ratio = ratio
        panes[paneId] = p
        return true
    }

    func deletePane(paneId: Int64) -> Bool {
        guard panes[paneId] != nil else { return false }
        panes.removeValue(forKey: paneId)
        return true
    }

    // ── Surface FFI (MS-2/3) ─────────────────────────────────────────────────
    var surfaces: [Int64: MockSurfaceRecord] = [:]
    var surfaceIdCounter: Int64 = 0

    func listSurfacesJson(paneId: Int64) -> String {
        let rows = surfaces.values.filter { $0.paneId == paneId }
            .sorted { $0.tabOrder < $1.tabOrder }
        let arr = rows.map { r in
            """
            {"id":\(r.id),"pane_id":\(r.paneId),"kind":"\(r.kind)","state_json":"\(r.stateJson)","tab_order":\(r.tabOrder)}
            """
        }
        return "[\(arr.joined(separator: ","))]"
    }

    func createSurface(paneId: Int64, kind: String, stateJson: String, tabOrder: Int64) -> Int64 {
        surfaceIdCounter += 1
        surfaces[surfaceIdCounter] = MockSurfaceRecord(
            id: surfaceIdCounter, paneId: paneId,
            kind: kind, stateJson: stateJson, tabOrder: tabOrder
        )
        return surfaceIdCounter
    }

    func updateSurfaceTabOrder(surfaceId: Int64, tabOrder: Int64) -> Bool {
        guard var s = surfaces[surfaceId] else { return false }
        surfaces[surfaceId] = MockSurfaceRecord(
            id: s.id,
            paneId: s.paneId,
            kind: s.kind,
            stateJson: s.stateJson,
            tabOrder: tabOrder
        )
        return true
    }

    func deleteSurface(surfaceId: Int64) -> Bool {
        guard surfaces[surfaceId] != nil else { return false }
        surfaces.removeValue(forKey: surfaceId)
        return true
    }

    // ── Workspace → DB id 변환 ───────────────────────────────────────────────
    var workspaceDbIds: [String: Int64] = [:]

    func getWorkspaceDbId(workspaceUuid: String) -> Int64 {
        workspaceDbIds[workspaceUuid] ?? 0
    }

    // ── FileTree FFI (MS-4) ─────────────────────────────────────────────────
    var stubbedDirectoryJson: String = "[]"
    var stubbedStatusJson: String = "{}"

    func listDirectoryJson(workspacePath: String, subpath: String) -> String {
        stubbedDirectoryJson
    }

    func gitStatusMapJson(workspacePath: String) -> String {
        stubbedStatusJson
    }

    // Test helpers
    func enqueue(workspaceId: String, event: String) {
        eventQueues[workspaceId, default: []].append(event)
    }

    func setStatus(id: String, status: WorkspaceStatus) {
        if let idx = workspaces.firstIndex(where: { $0.id == id }) {
            let w = workspaces[idx]
            workspaces[idx] = WorkspaceSnapshot(id: w.id, name: w.name, status: status)
        }
    }
}

// MARK: - 테스트용 데이터 레코드

struct MockPaneRecord {
    let id: Int64
    let workspaceId: Int64
    let parentId: Int64
    var split: String
    var ratio: Double
}

struct MockSurfaceRecord {
    let id: Int64
    let paneId: Int64
    let kind: String
    let stateJson: String
    var tabOrder: Int64
}
