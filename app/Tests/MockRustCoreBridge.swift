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
