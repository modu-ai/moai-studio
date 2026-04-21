//
//  StatusIcon.swift
//  워크스페이스 상태 아이콘 (Starting 스피너 / Running 녹색 / Error 빨강 / Paused 회색).
//

import SwiftUI

struct StatusIcon: View {
    let status: WorkspaceStatus

    var body: some View {
        switch status {
        case .starting:
            ProgressView()
                .controlSize(.small)
                .scaleEffect(0.7)
                .frame(width: 12, height: 12)
                .accessibilityLabel("Starting")
        case .running:
            Circle()
                .fill(Color.green)
                .frame(width: 10, height: 10)
                .accessibilityLabel("Running")
        case .error:
            Circle()
                .fill(Color.red)
                .frame(width: 10, height: 10)
                .accessibilityLabel("Error")
        case .paused:
            Circle()
                .fill(Color.gray)
                .frame(width: 10, height: 10)
                .accessibilityLabel("Paused")
        case .created:
            Circle()
                .fill(Color.blue.opacity(0.6))
                .frame(width: 10, height: 10)
                .accessibilityLabel("Created")
        case .deleted:
            Circle()
                .fill(Color.black.opacity(0.3))
                .frame(width: 10, height: 10)
                .accessibilityLabel("Deleted")
        }
    }
}
