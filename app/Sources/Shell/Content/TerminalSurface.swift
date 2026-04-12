//
//  TerminalSurface.swift
//  GhosttyKit 터미널 surface 래핑 + 초기화 실패 시 Fallback.
//
//  @MX:WARN: GhosttyKit 초기화는 Metal Toolchain 의존 — 실패 시 반드시 Fallback 으로 전환.
//  @MX:REASON: Ghostty 런타임 실패가 앱 크래시로 전파되지 않도록 격리 (RG-M1-2 §[If-Then]).
//

import SwiftUI

/// 터미널 백엔드 선택 — MOAI_TERMINAL_BACKEND 환경변수로 오버라이드 가능.
enum TerminalBackend: String {
    case ghostty
    case nstext

    static var current: TerminalBackend {
        if let raw = ProcessInfo.processInfo.environment["MOAI_TERMINAL_BACKEND"],
           let parsed = TerminalBackend(rawValue: raw.lowercased()) {
            return parsed
        }
        return .ghostty
    }
}

struct TerminalSurface: View {
    let workspace: WorkspaceSnapshot
    @State private var loadFailed: Bool = false

    var body: some View {
        Group {
            if loadFailed || TerminalBackend.current == .nstext {
                TerminalFallback(workspace: workspace) {
                    loadFailed = false
                }
            } else {
                GhosttyHost(workspace: workspace, onFailure: {
                    loadFailed = true
                })
            }
        }
        .background(Color.black)
    }
}

/// GhosttyKit 래퍼. M1 시점에서는 초기화를 시도하고 실패 시 fallback 으로 위임한다.
///
/// GhosttyKit API 는 M1 후속 태스크에서 완전 래핑되므로 현재는 placeholder 뷰로 렌더.
private struct GhosttyHost: View {
    let workspace: WorkspaceSnapshot
    let onFailure: () -> Void

    var body: some View {
        ZStack {
            Color.black
            VStack(alignment: .leading, spacing: 8) {
                Text("ghostty-vt.xcframework loaded")
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.green)
                Text("workspace: \(workspace.name)")
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(.white)
                Text("(Ghostty Metal surface will render here — wiring in MS-6)")
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.white.opacity(0.5))
            }
            .padding(12)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .onAppear {
            // 실제 GhosttyKit init 실패 감지 지점. M1 에서는 항상 성공으로 간주.
            // 실패 시 onFailure() 호출하여 fallback 뷰로 전환.
        }
    }
}
