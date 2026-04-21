//
//  FileTreeSurface.swift
//  파일 트리 Surface — 디렉토리 리스팅 + git status 색상 + expand/collapse
//  (SPEC-M2-001 MS-4 T-052~T-055).
//
//  @MX:ANCHOR: [AUTO] FileTreeViewModel — FileTree 탭 상태의 유일한 소스 (fan_in>=3)
//  @MX:REASON: [AUTO] FileTreeSurface, FileTreeViewModelTests, SurfaceRouter 세 경로에서 참조.
//              MS-4+ 에서 MarkdownSurface, CommandPalette 등도 이 ViewModel 을 통해 파일 경로를 획득한다.
//
//  @MX:NOTE: [AUTO] 폴링 기반 갱신 채택 (500ms Timer).
//            MS-7+ 에서 moai-fs notify-push 이벤트 기반으로 업그레이드 예정.

import Foundation
import Observation
import SwiftUI

// MARK: - GitStatus

/// 파일의 git 상태 및 표시 색상.
///
// @MX:NOTE: [AUTO] git status → 색상 매핑:
//           modified=노랑, added=초록, untracked=회색, deleted=빨강
public enum GitStatus: String, Sendable, Equatable {
    case clean
    case modified
    case added
    case untracked
    case deleted

    /// git_status 문자열에서 안전하게 파싱한다. 알 수 없는 값은 clean 으로 폴백.
    public init(rawString: String) {
        switch rawString {
        case "modified": self = .modified
        case "added": self = .added
        case "untracked": self = .untracked
        case "deleted": self = .deleted
        default: self = .clean
        }
    }

    /// 탭 파일명에 적용할 색상.
    public var color: Color {
        switch self {
        case .clean: return .primary
        case .modified: return .yellow
        case .added: return .green
        case .untracked: return .secondary
        case .deleted: return .red
        }
    }
}

// MARK: - FileTreeEntry

/// FileTree 리스트에 표시되는 단일 파일/디렉토리 항목.
public struct FileTreeEntry: Identifiable, Hashable, Sendable {
    /// 워크스페이스 루트 기준 상대 경로 (id 로 사용)
    public var id: String { path }

    public let path: String
    public let name: String
    public let isDirectory: Bool
    /// 루트 기준 깊이 (0 = 루트 직계)
    public let depth: Int

    public init(path: String, name: String, isDirectory: Bool, depth: Int) {
        self.path = path
        self.name = name
        self.isDirectory = isDirectory
        self.depth = depth
    }
}

// MARK: - FileTreeViewModel

/// FileTree Surface 의 상태를 관리하는 Observable ViewModel.
///
// @MX:ANCHOR: [AUTO] FileTree 탭의 유일한 상태 소스 (fan_in>=3)
// @MX:REASON: [AUTO] FileTreeSurface(body), FileTreeViewModelTests, SurfaceRouter 세 경로에서 참조
@Observable
@MainActor
public final class FileTreeViewModel {
    // MARK: 공개 상태

    /// 현재 표시 중인 파일/디렉토리 항목 목록 (Rust 응답 순서 유지)
    public private(set) var entries: [FileTreeEntry] = []

    /// 펼쳐진 디렉토리 경로 집합
    public private(set) var expandedPaths: Set<String> = []

    /// path → git 상태 맵
    public private(set) var gitStatusMap: [String: GitStatus] = [:]

    /// 워크스페이스 루트 절대 경로
    public let workspacePath: String

    // MARK: 내부 상태

    /// 테스트 접근용 내부 bridge 참조 (테스트에서 bridgeForTest 로 접근)
    internal let bridge: RustCoreBridging

    // MARK: 초기화

    public init(workspacePath: String, bridge: RustCoreBridging) {
        self.workspacePath = workspacePath
        self.bridge = bridge
    }

    // MARK: - 공개 메서드

    /// 루트 디렉토리 리스팅 + git status 를 로드한다.
    public func load() async {
        await fetchEntries()
        await fetchGitStatus()
    }

    /// 디렉토리를 토글(펼치기/접기)한다. 파일이면 no-op.
    public func toggle(path: String) {
        if expandedPaths.contains(path) {
            expandedPaths.remove(path)
        } else {
            expandedPaths.insert(path)
        }
    }

    /// 현재 표시 중인 디렉토리의 목록을 다시 로드한다 (폴링 용도).
    public func refresh() async {
        await fetchEntries()
        await fetchGitStatus()
    }

    // MARK: - 내부 헬퍼

    private func fetchEntries() async {
        let json = bridge.listDirectoryJson(workspacePath: workspacePath, subpath: "")
        entries = parseEntries(json)
    }

    private func fetchGitStatus() async {
        let json = bridge.gitStatusMapJson(workspacePath: workspacePath)
        gitStatusMap = parseStatusMap(json)
    }

    /// JSON 배열 → [FileTreeEntry] 파싱.
    private func parseEntries(_ json: String) -> [FileTreeEntry] {
        guard let data = json.data(using: .utf8),
              let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]]
        else { return [] }

        return arr.compactMap { dict -> FileTreeEntry? in
            guard let path = dict["path"] as? String,
                  let name = dict["name"] as? String,
                  let isDir = dict["is_directory"] as? Bool
            else { return nil }

            let depth = (dict["depth"] as? Int) ?? 0

            return FileTreeEntry(path: path, name: name, isDirectory: isDir, depth: depth)
        }
    }

    /// JSON 객체 → [String: GitStatus] 파싱.
    private func parseStatusMap(_ json: String) -> [String: GitStatus] {
        guard let data = json.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: String]
        else { return [:] }

        return obj.reduce(into: [:]) { result, pair in
            result[pair.key] = GitStatus(rawString: pair.value)
        }
    }

    // MARK: 테스트 헬퍼

    /// 테스트에서 bridge 인스턴스를 꺼내기 위한 접근자.
    internal var bridgeForTest: RustCoreBridging { bridge }
}

// MARK: - FileTreeSurface

/// FileTree Surface SwiftUI 뷰 (T-052).
///
// @MX:ANCHOR: [AUTO] FileTree Surface 렌더링 진입점 (fan_in>=3)
// @MX:REASON: [AUTO] SurfaceRouter, LeafPaneView, FileTreeViewModelTests 세 경로에서 참조
public struct FileTreeSurface: View {
    @State private var viewModel: FileTreeViewModel
    let onFileOpen: (String) -> Void

    public init(workspacePath: String, bridge: RustCoreBridging, onFileOpen: @escaping (String) -> Void) {
        self._viewModel = State(
            wrappedValue: FileTreeViewModel(workspacePath: workspacePath, bridge: bridge)
        )
        self.onFileOpen = onFileOpen
    }

    public var body: some View {
        List {
            ForEach(viewModel.entries) { entry in
                FileTreeRow(
                    entry: entry,
                    gitStatus: viewModel.gitStatusMap[entry.path] ?? .clean,
                    isExpanded: viewModel.expandedPaths.contains(entry.path)
                ) {
                    if entry.isDirectory {
                        viewModel.toggle(path: entry.path)
                    } else {
                        onFileOpen(entry.path)
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .task {
            await viewModel.load()
            startRefreshTimer()
        }
    }

    // @MX:NOTE: [AUTO] 500ms 폴링 타이머 — MS-7+ 에서 notify-push 로 교체 예정
    private func startRefreshTimer() {
        Task {
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 500_000_000) // 500ms
                await viewModel.refresh()
            }
        }
    }
}

// MARK: - SurfaceProtocol 준수

extension FileTreeSurface: SurfaceProtocol {
    public var surfaceKind: SurfaceKind { .filetree }

    public var toolbarItems: [SurfaceToolbarItem] {
        [
            SurfaceToolbarItem(
                id: "refresh",
                label: "Refresh",
                systemImage: "arrow.clockwise"
            )
        ]
    }
}

// MARK: - FileTreeRow (내부)

/// 단일 파일/디렉토리 행 뷰.
private struct FileTreeRow: View {
    let entry: FileTreeEntry
    let gitStatus: GitStatus
    let isExpanded: Bool
    let onTap: () -> Void

    /// 파일 확장자별 SF Symbol 이름 반환.
    private var symbolName: String {
        if entry.isDirectory {
            return isExpanded ? "folder.fill" : "folder"
        }
        let ext = (entry.name as NSString).pathExtension.lowercased()
        switch ext {
        case "md", "markdown", "txt": return "doc.text"
        case "png", "jpg", "jpeg", "gif", "webp", "svg": return "photo"
        default: return "doc"
        }
    }

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 4) {
                // 들여쓰기: depth × 12pt
                Spacer().frame(width: CGFloat(entry.depth) * 12)

                Image(systemName: symbolName)
                    .foregroundStyle(entry.isDirectory ? Color.accentColor : Color.secondary)
                    .frame(width: 16)

                Text(entry.name)
                    .foregroundStyle(gitStatus.color)
                    .lineLimit(1)
                    .truncationMode(.middle)

                Spacer()
            }
        }
        .buttonStyle(.plain)
    }
}
