//
//  CommandRegistry.swift
//  Command Palette 명령어 레지스트리 (SPEC-M2-001 MS-6 T-068).
//
//  @MX:ANCHOR: [AUTO] PaletteCommand 전체 목록의 유일한 소스 (fan_in>=3)
//  @MX:REASON: [AUTO] CommandPaletteController, RootSplitView, 테스트 세 경로에서 참조.
//              모든 명령어 등록/해제는 이 레지스트리를 통해야 한다.
//
//  @MX:NOTE: [AUTO] CommandRegistry 는 생성 시 콜백을 주입받는 방식으로 ViewModel 과 결합을 분리.
//            콜백: onMoaiSlash (슬래시 주입), onSurfaceOpen (탭 오픈), onWorkspaceCreate,
//            onPaneSplit (방향 enum). RootSplitView 가 연결 담당.

import Foundation

// MARK: - PaletteCommand

/// Command Palette 에 표시되는 개별 명령어.
///
/// handler 는 @MainActor 에서 실행된다.
public struct PaletteCommand: Identifiable, Hashable, Sendable {
    /// 안정적인 식별자 (예: "moai.plan", "surface.filetree")
    public let id: String
    /// 표시 이름
    public let title: String
    /// 부가 설명 (선택)
    public let subtitle: String?
    /// 카테고리
    public let category: Category
    /// Fuzzy 매칭 부스트용 키워드
    public let keywords: [String]
    /// 실행 핸들러
    public let handler: @MainActor @Sendable () -> Void

    public init(
        id: String,
        title: String,
        subtitle: String? = nil,
        category: Category,
        keywords: [String] = [],
        handler: @escaping @MainActor @Sendable () -> Void
    ) {
        self.id = id
        self.title = title
        self.subtitle = subtitle
        self.category = category
        self.keywords = keywords
        self.handler = handler
    }

    // Hashable: id 만 사용
    public static func == (lhs: PaletteCommand, rhs: PaletteCommand) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    // MARK: - Category

    /// Command 카테고리 — 검색 필터 및 그룹 표시에 사용.
    public enum Category: String, Sendable, CaseIterable, Equatable {
        case moai = "/moai"
        case surface = "Surface"
        case workspace = "Workspace"
        case pane = "Pane"
    }
}

// MARK: - PaneSplitDirection

/// Pane 분할 방향 (Pane Command 핸들러에 전달).
public enum PaneSplitDirection: String, Sendable {
    case horizontal
    case vertical
}

// MARK: - CommandRegistry

/// PaletteCommand 전체 목록을 관리한다.
///
/// - init 시 4개 카테고리의 내장 명령어를 자동 등록한다.
/// - 외부에서 커스텀 명령어를 register/unregister 할 수 있다.
@MainActor
public final class CommandRegistry {

    // MARK: 공개 상태

    /// 등록된 전체 명령어 목록.
    public private(set) var commands: [PaletteCommand] = []

    // MARK: 콜백 (RootSplitView 에서 주입)

    private let onMoaiSlash: @MainActor @Sendable (String) -> Void
    private let onSurfaceOpen: @MainActor @Sendable (SurfaceKind) -> Void
    private let onWorkspaceCreate: @MainActor @Sendable () -> Void
    private let onPaneSplit: @MainActor @Sendable (PaneSplitDirection) -> Void

    // MARK: 초기화

    public init(
        onMoaiSlash: @escaping @MainActor @Sendable (String) -> Void,
        onSurfaceOpen: @escaping @MainActor @Sendable (SurfaceKind) -> Void,
        onWorkspaceCreate: @escaping @MainActor @Sendable () -> Void,
        onPaneSplit: @escaping @MainActor @Sendable (PaneSplitDirection) -> Void
    ) {
        self.onMoaiSlash = onMoaiSlash
        self.onSurfaceOpen = onSurfaceOpen
        self.onWorkspaceCreate = onWorkspaceCreate
        self.onPaneSplit = onPaneSplit

        registerMoaiCommands()
        registerSurfaceCommands()
        registerWorkspaceCommands()
        registerPaneCommands()
    }

    // MARK: 공개 메서드

    /// 커스텀 명령어를 등록한다. 동일 id 가 이미 있으면 교체한다.
    public func register(_ command: PaletteCommand) {
        commands.removeAll { $0.id == command.id }
        commands.append(command)
    }

    /// id 로 명령어를 제거한다.
    public func unregister(id: String) {
        commands.removeAll { $0.id == id }
    }

    // MARK: - 내장 명령어 등록

    // @MX:NOTE: [AUTO] /moai 슬래시 명령어 14종 — 각 handler 가 SlashInjector 를 통해 FFI 로 전달됨.
    private func registerMoaiCommands() {
        let slashCommands: [(id: String, title: String, subtitle: String?, slash: String, keywords: [String])] = [
            ("moai.plan",     "/moai plan",     "SPEC 문서 생성",      "/moai plan",     ["spec", "planning", "새 기능"]),
            ("moai.run",      "/moai run",      "SPEC 구현 실행",      "/moai run",      ["implement", "ddd", "tdd"]),
            ("moai.sync",     "/moai sync",     "문서 동기화",         "/moai sync",     ["docs", "documentation"]),
            ("moai.fix",      "/moai fix",      "오류 자동 수정",      "/moai fix",      ["error", "bug", "fix"]),
            ("moai.loop",     "/moai loop",     "반복 실행",           "/moai loop",     ["iterate", "loop"]),
            ("moai.project",  "/moai project",  "프로젝트 문서 생성",  "/moai project",  ["project", "readme"]),
            ("moai.feedback", "/moai feedback", "피드백 제출",         "/moai feedback", ["issue", "bug report"]),
            ("moai.review",   "/moai review",   "코드 리뷰",           "/moai review",   ["review", "code quality"]),
            ("moai.clean",    "/moai clean",    "데드 코드 제거",      "/moai clean",    ["cleanup", "dead code"]),
            ("moai.codemaps", "/moai codemaps", "코드맵 생성",         "/moai codemaps", ["architecture", "diagram"]),
            ("moai.coverage", "/moai coverage", "테스트 커버리지 분석","/moai coverage", ["test", "coverage"]),
            ("moai.e2e",      "/moai e2e",      "E2E 테스트 실행",     "/moai e2e",      ["e2e", "playwright"]),
            ("moai.mx",       "/moai mx",       "MX 태그 스캔",        "/moai mx",       ["annotation", "mx tag"]),
            ("moai.context",  "/moai context",  "이전 컨텍스트 검색",  "/moai context",  ["search", "history"]),
        ]
        for entry in slashCommands {
            let slash = entry.slash
            let cmd = PaletteCommand(
                id: entry.id,
                title: entry.title,
                subtitle: entry.subtitle,
                category: .moai,
                keywords: entry.keywords,
                handler: { [weak self] in self?.onMoaiSlash(slash) }
            )
            commands.append(cmd)
        }
    }

    // @MX:NOTE: [AUTO] Surface 열기 명령어 — TabBarViewModel.newTab(kind:) 을 콜백으로 호출.
    private func registerSurfaceCommands() {
        let surfaceEntries: [(id: String, title: String, subtitle: String, kind: SurfaceKind, keywords: [String])] = [
            ("surface.filetree",  "File Tree 열기",   "파일 트리 Surface",  .filetree, ["file", "directory", "tree"]),
            ("surface.markdown",  "Markdown 파일 열기","Markdown Surface",  .markdown, ["md", "markdown", "문서"]),
            ("surface.image",     "Image 파일 열기",  "Image Surface",     .image,    ["image", "png", "jpg", "사진"]),
            ("surface.browser",   "Browser 열기",     "Browser Surface",   .browser,  ["browser", "web", "localhost"]),
            ("surface.terminal",  "Terminal 열기",    "Terminal Surface",  .terminal, ["shell", "terminal", "cli"]),
        ]
        for entry in surfaceEntries {
            let kind = entry.kind
            let cmd = PaletteCommand(
                id: entry.id,
                title: entry.title,
                subtitle: entry.subtitle,
                category: .surface,
                keywords: entry.keywords,
                handler: { [weak self] in self?.onSurfaceOpen(kind) }
            )
            commands.append(cmd)
        }
    }

    // @MX:NOTE: [AUTO] Workspace 명령어 — WorkspaceViewModel 콜백 연결.
    private func registerWorkspaceCommands() {
        let workspaceEntries: [(id: String, title: String, subtitle: String, keywords: [String])] = [
            ("workspace.new",    "새 Workspace 생성", "Workspace 생성 시트 열기", ["new", "workspace", "create"]),
        ]
        for entry in workspaceEntries {
            let cmd = PaletteCommand(
                id: entry.id,
                title: entry.title,
                subtitle: entry.subtitle,
                category: .workspace,
                keywords: entry.keywords,
                handler: { [weak self] in self?.onWorkspaceCreate() }
            )
            commands.append(cmd)
        }
    }

    // @MX:NOTE: [AUTO] Pane 분할/닫기 명령어 — PaneTreeModel 콜백 연결.
    private func registerPaneCommands() {
        let paneEntries: [(id: String, title: String, subtitle: String, direction: PaneSplitDirection, keywords: [String])] = [
            ("pane.split.horizontal", "Pane 수평 분할", "좌우로 분할 (Cmd+\\)",       .horizontal, ["split", "horizontal", "left", "right"]),
            ("pane.split.vertical",   "Pane 수직 분할", "상하로 분할 (Cmd+Shift+\\)", .vertical,   ["split", "vertical", "top", "bottom"]),
        ]
        for entry in paneEntries {
            let direction = entry.direction
            let cmd = PaletteCommand(
                id: entry.id,
                title: entry.title,
                subtitle: entry.subtitle,
                category: .pane,
                keywords: entry.keywords,
                handler: { [weak self] in self?.onPaneSplit(direction) }
            )
            commands.append(cmd)
        }
        // 팔레트 닫기는 콜백 불필요 — 별도 등록
        // Note: Pane 닫기(Cmd+Shift+W) 는 핸들러가 PaneTreeModel 닫기를 직접 호출해야 하므로
        //       RootSplitView 에서 register() 로 추가한다. 여기서는 placeholder 로 stub.
    }
}
