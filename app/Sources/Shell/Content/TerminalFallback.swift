//
//  TerminalFallback.swift
//  GhosttyKit 실패 / MOAI_TERMINAL_BACKEND=nstext 시 NSTextView 기반 fallback.
//

import SwiftUI

struct TerminalFallback: View {
    let workspace: WorkspaceSnapshot
    let onRetry: () -> Void

    var body: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.largeTitle)
                .foregroundStyle(.orange)
            Text("Terminal unavailable")
                .font(.title2)
                .bold()
            Text("workspace: \(workspace.name)")
                .foregroundStyle(.secondary)
            Text("The Metal-based terminal could not initialize. Using text fallback.")
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Button("Retry") {
                onRetry()
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("terminalFallback.retry")

            // 최소한의 텍스트 출력 영역 — 스트리밍 이벤트 덤프.
            ScrollView {
                Text("(fallback console — stream events will render here)")
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(.white.opacity(0.7))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
            }
            .background(Color.black)
            .frame(minHeight: 160)
            Spacer()
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
