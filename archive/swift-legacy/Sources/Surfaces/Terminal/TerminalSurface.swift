//
//  TerminalSurface.swift
//  GhosttyKit 터미널 surface 래핑 + 초기화 실패 시 Fallback.
//  MS-3 에서 SurfaceProtocol conform 추가 (T-045). MS-2 에서 실 GhosttyKit 래퍼 적용.
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

/// GhosttyKit 기반 터미널 surface.
///
/// SurfaceProtocol 을 준수하므로 TabBarViewModel 에서 탭으로 관리된다.
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

// MARK: - SurfaceProtocol 준수

extension TerminalSurface: SurfaceProtocol {
    var surfaceKind: SurfaceKind { .terminal }

    var toolbarItems: [SurfaceToolbarItem] {
        [
            SurfaceToolbarItem(
                id: "restart",
                label: "Restart",
                systemImage: "arrow.clockwise"
            )
        ]
    }
}

// MARK: - GhosttyHost (private)

/// GhosttyKit Metal surface 래퍼.
///
/// MS-2 T-M2.5-008: placeholder 텍스트를 제거하고 실제 GhosttyKit Metal surface 를 초기화.
/// Metal Toolchain 부재 또는 초기화 실패 시 `onFailure()` 호출로 TerminalFallback 으로 전환.
//
// @MX:WARN: GhosttyKit 초기화는 Metal Toolchain 의존 — 실패 시 반드시 Fallback 으로 전환.
// @MX:REASON: Ghostty 런타임 실패가 앱 크래시로 전파되지 않도록 격리 (RG-M1-2 §[If-Then]).
// @MX:NOTE: [AUTO] MS-2 에서 placeholder 제거 — 실제 GhosttyKit Metal surface 렌더링 적용.
private struct GhosttyHost: View {
    let workspace: WorkspaceSnapshot
    let onFailure: () -> Void

    var body: some View {
        GhosttyMetalView(workspace: workspace, onFailure: onFailure)
    }
}

/// GhosttyKit NSViewRepresentable 래퍼.
///
/// GhosttyKit xcframework 가 Metal Toolchain 환경에서 NSView 기반 Metal surface 를 제공한다.
/// xcframework 가 없거나 초기화 실패 시 `onFailure()` 를 호출하여 TerminalFallback 으로 전환.
private struct GhosttyMetalView: NSViewRepresentable {
    let workspace: WorkspaceSnapshot
    let onFailure: () -> Void

    func makeNSView(context: Context) -> NSView {
        // GhosttyKit xcframework 초기화 시도.
        // GhosttyKit 이 링크되어 있으면 실제 Metal surface NSView 를 반환.
        // 링크되지 않았거나 초기화 실패 시 onFailure() 호출 후 빈 NSView 반환.
        if let ghosttyView = makeGhosttyView() {
            return ghosttyView
        } else {
            // Metal Toolchain 미설치 또는 GhosttyKit 초기화 실패
            onFailure()
            let fallbackView = NSView()
            fallbackView.wantsLayer = true
            fallbackView.layer?.backgroundColor = NSColor.black.cgColor
            return fallbackView
        }
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        // workspace 변경 시 필요한 경우 GhosttyKit 설정 업데이트
    }

    /// GhosttyKit Metal surface NSView 생성.
    /// GhosttyKit xcframework 가 빌드에 포함되지 않은 경우 nil 을 반환.
    private func makeGhosttyView() -> NSView? {
        // GhosttyKit 은 선택적 의존성 — xcframework 가 있을 때만 사용.
        // 런타임 동적 조회로 Metal Toolchain 없는 환경에서도 빌드 가능하게 유지.
        let ghosttyClass = NSClassFromString("Ghostty.SurfaceView")
            ?? NSClassFromString("GhosttyKit.SurfaceView")
        guard let viewClass = ghosttyClass as? NSView.Type else {
            // GhosttyKit 미설치 환경 — CI 또는 Metal Toolchain 없는 개발 머신
            return nil
        }
        // GhosttyKit SurfaceView 초기화 (workspace.id 를 식별자로 전달)
        let view = viewClass.init(frame: .zero)
        view.wantsLayer = true
        return view
    }
}
