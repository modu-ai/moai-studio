//
//  SurfaceProtocol.swift
//  10종 Surface 공통 인터페이스 정의 (SPEC-M2-001 MS-3 T-044).
//
//  @MX:ANCHOR: [AUTO] 10종 Surface 의 유일한 공통 계약 (fan_in>=3 예상: MS-4+ 에서 확장)
//  @MX:REASON: [AUTO] TabBarView, SurfaceRouter, 각 Surface 구현체(MS-4+) 에서 참조.
//              모든 Surface 는 이 프로토콜을 준수해야 한다.
//  @MX:NOTE: [AUTO] 10종 Surface 종류 레지스트리:
//            terminal, code, markdown, image, browser,
//            filetree, agent_run, kanban, memory, instructions_graph.
//            Rust moai-store 의 SurfaceKind 와 1:1 매핑.

import SwiftUI

// MARK: - SurfaceKind

/// 10종 Surface 공통 종류 분류 (moai-store `SurfaceKind` 와 1:1 매핑).
public enum SurfaceKind: String, Sendable, CaseIterable, Codable {
    case terminal
    case code
    case markdown
    case image
    case browser
    case filetree
    case agentRun = "agent_run"
    case kanban
    case memory
    case instructionsGraph = "instructions_graph"

    /// Rust/JSON 에서 전달되는 문자열을 안전하게 파싱한다.
    /// 알 수 없는 값은 terminal 로 폴백한다.
    public init(rawString: String) {
        let normalized = rawString.lowercased()
        switch normalized {
        case "terminal": self = .terminal
        case "code": self = .code
        case "markdown": self = .markdown
        case "image": self = .image
        case "browser": self = .browser
        case "filetree": self = .filetree
        case "agent_run": self = .agentRun
        case "kanban": self = .kanban
        case "memory": self = .memory
        case "instructions_graph": self = .instructionsGraph
        default: self = .terminal
        }
    }

    /// 탭 헤더에 표시할 기본 제목.
    public var defaultTitle: String {
        switch self {
        case .terminal: return "Terminal"
        case .code: return "Code"
        case .markdown: return "Markdown"
        case .image: return "Image"
        case .browser: return "Browser"
        case .filetree: return "File Tree"
        case .agentRun: return "Agent Run"
        case .kanban: return "Kanban"
        case .memory: return "Memory"
        case .instructionsGraph: return "Instructions"
        }
    }

    /// SF Symbol 아이콘 이름.
    public var systemImage: String {
        switch self {
        case .terminal: return "terminal"
        case .code: return "chevron.left.forwardslash.chevron.right"
        case .markdown: return "doc.text"
        case .image: return "photo"
        case .browser: return "globe"
        case .filetree: return "folder"
        case .agentRun: return "cpu"
        case .kanban: return "square.grid.3x3"
        case .memory: return "brain"
        case .instructionsGraph: return "flowchart"
        }
    }
}

// MARK: - SurfaceToolbarItem

/// Surface 툴바 아이템 사양 (MS-3 최소 형태, MS-4+ 에서 확장 가능).
public struct SurfaceToolbarItem: Identifiable, Sendable {
    public let id: String
    public let label: String
    public let systemImage: String?

    public init(id: String, label: String, systemImage: String? = nil) {
        self.id = id
        self.label = label
        self.systemImage = systemImage
    }
}

// MARK: - SurfaceProtocol

/// 모든 Surface 가 conform 해야 하는 공통 프로토콜.
///
/// - 기본 제공 타입: TerminalSurface (MS-3 에서 마이그레이션).
///   MS-4+ 에서 FileTree/Markdown/Image/Browser 추가.
///
/// SwiftUI View 프로토콜을 상속하므로 각 Surface 는 body 를 구현해야 한다.
@MainActor
public protocol SurfaceProtocol: View {
    /// Surface 종류 (`SurfaceKind` 와 1:1).
    var surfaceKind: SurfaceKind { get }

    /// Surface 별 툴바 아이템 목록 (없으면 빈 배열).
    var toolbarItems: [SurfaceToolbarItem] { get }
}

// MARK: - SurfaceLifecycleHandler

/// Surface lifecycle 이벤트 (탭 전환 시 호출되는 hook — optional).
///
/// 뷰 외부에서 호출 가능하도록 별도 프로토콜로 분리.
/// SwiftUI View 는 struct value type 이므로 클래스 기반으로 별도 정의.
public protocol SurfaceLifecycleHandler: AnyObject {
    /// 탭이 활성화될 때 호출된다.
    func activate()
    /// 탭이 비활성화(배경)될 때 호출된다.
    func deactivate()
    /// 탭이 완전히 닫힐 때 호출된다.
    func destroy()
}
