//
//  TabBarView.swift
//  pane 상단 탭 바 컴포넌트 (SPEC-M2-001 MS-3 T-046).
//
//  @MX:NOTE: [AUTO] tab_order 불변식: tabs 배열은 항상 tabOrder 오름차순으로 정렬되어야 한다.
//            TabBarViewModel.reorder() 호출 후 순서 갱신이 보장된다.

import SwiftUI

// MARK: - TabItem

/// 하나의 탭을 나타내는 경량 데이터 모델.
///
/// - id: Rust DB surface id (Int64).
/// - kind: SurfaceKind — 탭 아이콘 결정에 사용.
/// - title: 탭 헤더 텍스트.
/// - tabOrder: DB 저장 순서 값.
public struct TabItem: Identifiable, Equatable, Sendable {
    public let id: Int64
    public let kind: SurfaceKind
    public let title: String
    public let tabOrder: Int64

    public init(id: Int64, kind: SurfaceKind, title: String, tabOrder: Int64) {
        self.id = id
        self.kind = kind
        self.title = title
        self.tabOrder = tabOrder
    }
}

// MARK: - TabBarView

/// pane 상단의 탭 바.
///
/// 탭 선택, 추가, 닫기, 드래그 재배치를 지원한다.
@MainActor
public struct TabBarView: View {
    // MARK: 입력

    let items: [TabItem]
    @Binding var activeId: Int64?
    let onNewTab: () -> Void
    let onCloseTab: (Int64) -> Void
    let onReorder: (_ from: Int, _ to: Int) -> Void
    let onSelect: (Int64) -> Void

    // MARK: 드래그 상태

    @State private var draggingItem: TabItem?
    @State private var dragTargetId: Int64?

    public init(
        items: [TabItem],
        activeId: Binding<Int64?>,
        onNewTab: @escaping () -> Void,
        onCloseTab: @escaping (Int64) -> Void,
        onReorder: @escaping (Int, Int) -> Void,
        onSelect: @escaping (Int64) -> Void
    ) {
        self.items = items
        self._activeId = activeId
        self.onNewTab = onNewTab
        self.onCloseTab = onCloseTab
        self.onReorder = onReorder
        self.onSelect = onSelect
    }

    // MARK: body

    public var body: some View {
        HStack(spacing: 0) {
            // 탭 목록 (가로 스크롤)
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 1) {
                    ForEach(items) { tab in
                        tabCell(for: tab)
                    }
                }
                .padding(.horizontal, 4)
            }

            Divider()
                .frame(height: 20)

            // + 새 탭 버튼
            newTabButton
        }
        .frame(height: 32)
        .background(Color(NSColor.controlBackgroundColor))
        // T-046: Cmd+T — 새 탭 (hidden button 으로 처리)
        .background(
            Button("") { onNewTab() }
                .keyboardShortcut("t", modifiers: .command)
                .frame(width: 0, height: 0)
                .opacity(0)
        )
        // T-046: Cmd+W — 활성 탭 닫기
        .background(
            Button("") {
                if let id = activeId { onCloseTab(id) }
            }
            .keyboardShortcut("w", modifiers: .command)
            .frame(width: 0, height: 0)
            .opacity(0)
        )
    }

    // MARK: - 탭 셀

    @ViewBuilder
    private func tabCell(for tab: TabItem) -> some View {
        let isActive = activeId == tab.id
        let isDragTarget = dragTargetId == tab.id && draggingItem?.id != tab.id

        HStack(spacing: 4) {
            // Surface 아이콘
            Image(systemName: tab.kind.systemImage)
                .font(.system(size: 11))
                .foregroundStyle(isActive ? Color.primary : Color.secondary)

            // 탭 제목
            Text(tab.title)
                .font(.system(size: 12, weight: isActive ? .medium : .regular))
                .lineLimit(1)
                .foregroundStyle(isActive ? Color.primary : Color.secondary)

            // 닫기 버튼
            closeButton(for: tab)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .frame(minWidth: 80, maxWidth: 180)
        .background(tabBackground(isActive: isActive, isDragTarget: isDragTarget))
        .overlay(alignment: .bottom) {
            // 활성 탭 밑줄 표시기 (AC-2.5)
            if isActive {
                Rectangle()
                    .fill(Color.accentColor)
                    .frame(height: 2)
            }
        }
        .contentShape(Rectangle())
        .onTapGesture {
            onSelect(tab.id)
        }
        // 드래그 재배치 (T-046 AC-2.4: 같은 pane 내 재배치)
        .onDrag {
            draggingItem = tab
            return NSItemProvider(object: "\(tab.id)" as NSString)
        }
        .onDrop(
            of: [.plainText],
            delegate: TabDropDelegate(
                targetTab: tab,
                items: items,
                draggingItem: $draggingItem,
                dragTargetId: $dragTargetId,
                onReorder: onReorder
            )
        )
    }

    @ViewBuilder
    private func closeButton(for tab: TabItem) -> some View {
        Button {
            onCloseTab(tab.id)
        } label: {
            Image(systemName: "xmark")
                .font(.system(size: 9, weight: .bold))
                .foregroundStyle(Color.secondary)
                .frame(width: 14, height: 14)
                .background(
                    Circle()
                        .fill(Color.secondary.opacity(0.001))
                )
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Close tab \(tab.title)")
    }

    @ViewBuilder
    private func tabBackground(isActive: Bool, isDragTarget: Bool) -> some View {
        if isDragTarget {
            Color.accentColor.opacity(0.15)
        } else if isActive {
            Color(NSColor.selectedContentBackgroundColor).opacity(0.15)
        } else {
            Color.clear
        }
    }

    // MARK: - 새 탭 버튼

    private var newTabButton: some View {
        Button {
            onNewTab()
        } label: {
            Image(systemName: "plus")
                .font(.system(size: 12))
                .foregroundStyle(Color.secondary)
                .frame(width: 28, height: 28)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("New tab")
        .padding(.horizontal, 4)
    }
}

// MARK: - TabDropDelegate

/// 탭 드래그-앤-드롭 재배치 처리.
private struct TabDropDelegate: DropDelegate {
    let targetTab: TabItem
    let items: [TabItem]
    @Binding var draggingItem: TabItem?
    @Binding var dragTargetId: Int64?
    let onReorder: (Int, Int) -> Void

    func dropEntered(info: DropInfo) {
        dragTargetId = targetTab.id
    }

    func dropExited(info: DropInfo) {
        if dragTargetId == targetTab.id {
            dragTargetId = nil
        }
    }

    func performDrop(info: DropInfo) -> Bool {
        defer {
            draggingItem = nil
            dragTargetId = nil
        }
        guard let source = draggingItem,
              source.id != targetTab.id,
              let fromIndex = items.firstIndex(where: { $0.id == source.id }),
              let toIndex = items.firstIndex(where: { $0.id == targetTab.id })
        else { return false }

        onReorder(fromIndex, toIndex)
        return true
    }

    func dropUpdated(info: DropInfo) -> DropProposal? {
        DropProposal(operation: .move)
    }

    func validateDrop(info: DropInfo) -> Bool {
        draggingItem != nil
    }
}
