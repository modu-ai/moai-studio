//
//  CommandPaletteView.swift
//  Command Palette UI + Controller (SPEC-M2-001 MS-6 T-067).
//
//  @MX:ANCHOR: [AUTO] Command Palette 상태 관리 진입점 (fan_in>=3)
//  @MX:REASON: [AUTO] RootSplitView(오버레이), CommandPaletteView(바인딩), 테스트 세 경로에서 참조.
//              open/close/execute/refreshResults 가 단일 상태 소스.
//
//  @MX:NOTE: [AUTO] Cmd+K 캡처 전략:
//            RootSplitView 의 NavigationSplitView 에 .background(focusable content)
//            + onKeyPress 를 적용하면 포커스와 무관하게 Cmd+K 가 캡처된다.
//            CommandMenu 방식 대신 onKeyPress 사용 — MenuBar 항목 생성 없이 단축키만 처리.

import SwiftUI
import AppKit

// MARK: - CommandPaletteController

/// Command Palette 의 상태와 동작을 관리하는 @Observable Controller.
///
/// RootSplitView 에서 생성되어 Environment 로 주입된다.
@Observable
@MainActor
public final class CommandPaletteController {

    // MARK: 공개 상태

    /// 팔레트 표시 여부
    public var isPresented: Bool = false
    /// 현재 검색어
    public var query: String = ""
    /// 매칭된 결과 목록 (FuzzyMatcher 출력)
    public private(set) var results: [PaletteCommand] = []
    /// 현재 선택된 항목 인덱스
    public var selectedIndex: Int = 0

    // MARK: 내부 상태

    private let registry: CommandRegistry
    /// 최근 실행 명령어 id 목록 (최대 10개, 중복 없음)
    public private(set) var historyIds: [String] = []

    // MARK: 초기화

    public init(registry: CommandRegistry) {
        self.registry = registry
    }

    // MARK: - 공개 메서드

    /// 팔레트를 열고 쿼리를 초기화한다.
    public func open() {
        query = ""
        refreshResults()
        selectedIndex = 0
        isPresented = true
    }

    /// 팔레트를 닫는다.
    public func close() {
        isPresented = false
    }

    /// 명령어를 실행하고 팔레트를 닫는다.
    public func execute(_ command: PaletteCommand) {
        command.handler()
        recordHistory(command.id)
        close()
    }

    /// 현재 query 로 결과를 갱신한다.
    public func refreshResults() {
        let matches = FuzzyMatcher.match(query: query, commands: registry.commands)
        results = matches.map { $0.command }
    }

    /// 선택 인덱스를 delta 만큼 이동한다 (범위 클램프).
    public func moveSelection(_ delta: Int) {
        guard !results.isEmpty else { return }
        let newIndex = selectedIndex + delta
        selectedIndex = max(0, min(newIndex, results.count - 1))
    }

    // MARK: - 내부 헬퍼

    /// 최근 명령어 히스토리에 기록 (최대 10개, 중복 없음).
    private func recordHistory(_ id: String) {
        // 기존 항목 제거 후 앞에 삽입
        historyIds.removeAll { $0 == id }
        historyIds.insert(id, at: 0)
        if historyIds.count > 10 {
            historyIds = Array(historyIds.prefix(10))
        }
    }
}

// MARK: - CommandPaletteView

/// Command Palette 오버레이 뷰.
///
/// RootSplitView 의 .overlay { CommandPaletteView(controller: controller) } 로 삽입된다.
public struct CommandPaletteView: View {
    @Bindable var controller: CommandPaletteController

    public init(controller: CommandPaletteController) {
        self.controller = controller
    }

    public var body: some View {
        ZStack {
            if controller.isPresented {
                // 배경 딤 레이어
                Color.black.opacity(0.3)
                    .ignoresSafeArea()
                    .onTapGesture { controller.close() }

                // 팔레트 패널
                VStack(spacing: 0) {
                    // 검색 입력창
                    TextField("Type a command...", text: $controller.query)
                        .textFieldStyle(.plain)
                        .font(.system(size: 16))
                        .padding(.horizontal, 16)
                        .padding(.vertical, 12)
                        .onChange(of: controller.query) { _, _ in
                            controller.refreshResults()
                            controller.selectedIndex = 0
                        }

                    Divider()

                    // 결과 목록
                    List(selection: .constant(nil as Int?)) {
                        ForEach(Array(controller.results.enumerated()), id: \.element.id) { idx, cmd in
                            CommandRowView(command: cmd, isSelected: idx == controller.selectedIndex)
                                .listRowInsets(EdgeInsets(top: 4, leading: 12, bottom: 4, trailing: 12))
                                .listRowBackground(
                                    idx == controller.selectedIndex
                                    ? Color.accentColor.opacity(0.15)
                                    : Color.clear
                                )
                                .onTapGesture { controller.execute(cmd) }
                        }
                    }
                    .listStyle(.plain)
                }
                .frame(width: 600, height: 400)
                .background(.regularMaterial)
                .cornerRadius(10)
                .shadow(radius: 20)
                // 키보드 네비게이션
                .onKeyPress(.escape) { controller.close(); return .handled }
                .onKeyPress(.return) {
                    if let cmd = controller.results[safe: controller.selectedIndex] {
                        controller.execute(cmd)
                    }
                    return .handled
                }
                .onKeyPress(.upArrow) { controller.moveSelection(-1); return .handled }
                .onKeyPress(.downArrow) { controller.moveSelection(1); return .handled }
            }
        }
        .animation(.easeInOut(duration: 0.15), value: controller.isPresented)
    }
}

// MARK: - CommandRowView

/// Command Palette 단일 행 뷰.
private struct CommandRowView: View {
    let command: PaletteCommand
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 10) {
            // 카테고리 뱃지
            Text(command.category.rawValue)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(Color.secondary.opacity(0.15))
                .cornerRadius(4)

            VStack(alignment: .leading, spacing: 1) {
                Text(command.title)
                    .font(.system(size: 14))
                    .foregroundStyle(.primary)
                if let subtitle = command.subtitle {
                    Text(subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()
        }
        .contentShape(Rectangle())
    }
}

// MARK: - Array safe subscript

private extension Array {
    subscript(safe index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
