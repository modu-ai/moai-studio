//
//  TabBarViewModel.swift
//  pane 내 surface(탭) 목록 관리 + FFI 연동 (SPEC-M2-001 MS-3 T-047).
//
//  @MX:ANCHOR: [AUTO] pane 내 탭 상태의 유일한 소스 (fan_in>=4)
//  @MX:REASON: [AUTO] TabBarView, LeafPaneView, T-049 테스트, RootSplitView (Command Palette) 네 경로에서 참조.
//              MS-3 에서 Command Palette onSurfaceOpen 콜백이 newTab(kind:) 를 호출하는 경로 추가됨.
//
//  @MX:NOTE: [AUTO] 기본 탭 자동 생성 규칙:
//            load() 후 pane 에 surface 가 하나도 없으면 Terminal surface 를 자동 생성한다.
//            이로써 모든 leaf pane 은 항상 최소 1개의 탭을 보유한다.

import Foundation
import Observation

// MARK: - TabBarViewModel

/// pane 내 탭(surface) 목록을 관리하는 ViewModel.
///
/// 모든 변이(추가/삭제/재배치)는 즉시 FFI 를 통해 DB 에 영속된다 (낙관적 업데이트).
@Observable
@MainActor
public final class TabBarViewModel {
    // MARK: 공개 상태

    /// 현재 pane 의 탭 목록 (tabOrder 오름차순).
    public private(set) var tabs: [TabItem] = []

    /// 현재 활성 탭 id. nil 이면 탭이 없는 상태.
    public var activeTabId: Int64?

    // MARK: 내부 상태

    public let paneId: Int64
    private let bridge: RustCoreBridging

    // MARK: 초기화

    public init(paneId: Int64, bridge: RustCoreBridging) {
        self.paneId = paneId
        self.bridge = bridge
    }

    // MARK: - 공개 메서드

    /// FFI 에서 surface 목록을 읽어 tabs 를 채운다.
    ///
    /// surface 가 없으면 기본 Terminal 탭을 자동 생성한다.
    public func load() async {
        let json = bridge.listSurfacesJson(paneId: paneId)
        let parsed = parseSurfacesJson(json)

        if parsed.isEmpty {
            // @MX:NOTE: [AUTO] 기본 탭 자동 생성 — pane 이 항상 최소 1개 탭 보유를 보장
            let newId = bridge.createSurface(
                paneId: paneId,
                kind: SurfaceKind.terminal.rawValue,
                stateJson: "",
                tabOrder: 0
            )
            if newId > 0 {
                let item = TabItem(
                    id: newId,
                    kind: .terminal,
                    title: SurfaceKind.terminal.defaultTitle,
                    tabOrder: 0
                )
                tabs = [item]
                activeTabId = newId
            }
        } else {
            tabs = parsed
            if activeTabId == nil || !tabs.contains(where: { $0.id == activeTabId }) {
                activeTabId = tabs.first?.id
            }
        }
    }

    /// 새 탭을 생성하고 추가된 surface id 를 반환한다.
    ///
    /// FFI 를 통해 DB 에 즉시 영속한다. 실패 시 nil 반환.
    ///
    /// - Parameters:
    ///   - kind: surface 종류 (기본값: terminal)
    ///   - statePath: 파일 경로 기반 surface 상태 (FileTree → 파일 오픈 시 사용). nil 이면 빈 상태.
    // @MX:NOTE: [AUTO] T-054: FileTree 파일 클릭 → 확장자 기반 surface 오픈.
    //            statePath 는 JSON 직렬화되어 surface state_json 에 저장된다.
    @discardableResult
    public func newTab(kind: SurfaceKind = .terminal, statePath: String? = nil) -> Int64? {
        let nextOrder = (tabs.map { $0.tabOrder }.max() ?? -1) + 1
        let stateJson: String
        if let path = statePath {
            // 경로에 큰따옴표가 포함될 수 있으므로 JSON 직렬화 사용
            let escaped = path.replacingOccurrences(of: "\\", with: "\\\\")
                              .replacingOccurrences(of: "\"", with: "\\\"")
            stateJson = "{\"path\":\"\(escaped)\"}"
        } else {
            stateJson = ""
        }
        let newId = bridge.createSurface(
            paneId: paneId,
            kind: kind.rawValue,
            stateJson: stateJson,
            tabOrder: nextOrder
        )
        guard newId > 0 else { return nil }

        let item = TabItem(
            id: newId,
            kind: kind,
            title: kind.defaultTitle,
            tabOrder: nextOrder
        )
        tabs.append(item)
        activeTabId = newId
        // statePath 캐시 저장
        if let path = statePath {
            statePathCache[newId] = path
        }
        return newId
    }

    /// 탭을 닫는다.
    ///
    /// - Returns: 닫기 성공 시 true. 마지막 탭이면 false (닫기 거부).
    @discardableResult
    public func closeTab(_ surfaceId: Int64) -> Bool {
        // 마지막 탭은 닫을 수 없다 (AC-2.3: pane 닫기는 상위에서 처리)
        guard tabs.count > 1 else { return false }

        _ = bridge.deleteSurface(surfaceId: surfaceId)
        tabs.removeAll { $0.id == surfaceId }
        statePathCache.removeValue(forKey: surfaceId)

        // 닫힌 탭이 활성이었으면 인접 탭으로 이동
        if activeTabId == surfaceId {
            activeTabId = tabs.first?.id
        }
        return true
    }

    /// 탭 순서를 재배치하고 모든 영향받은 탭의 tab_order 를 DB 에 업데이트한다.
    public func reorder(from fromIndex: Int, to toIndex: Int) {
        guard fromIndex != toIndex,
              tabs.indices.contains(fromIndex),
              tabs.indices.contains(toIndex)
        else { return }

        // 낙관적 업데이트: 배열 먼저 이동
        var reordered = tabs
        let moved = reordered.remove(at: fromIndex)
        reordered.insert(moved, at: toIndex)

        // tab_order 를 인덱스 순서로 재할당
        tabs = reordered.enumerated().map { index, tab in
            TabItem(
                id: tab.id,
                kind: tab.kind,
                title: tab.title,
                tabOrder: Int64(index)
            )
        }

        // DB 에 모든 변경된 tab_order 를 반영
        for tab in tabs {
            _ = bridge.updateSurfaceTabOrder(surfaceId: tab.id, tabOrder: tab.tabOrder)
        }
    }

    /// 활성 탭을 변경한다.
    public func selectTab(_ surfaceId: Int64) {
        guard tabs.contains(where: { $0.id == surfaceId }) else { return }
        activeTabId = surfaceId
    }

    /// 현재 활성 탭의 SurfaceKind 를 반환한다.
    public func activeTabKind() -> SurfaceKind? {
        guard let id = activeTabId else { return nil }
        return tabs.first { $0.id == id }?.kind
    }

    /// 현재 활성 탭의 state_json 에서 파일 경로를 추출한다.
    ///
    // @MX:NOTE: [AUTO] state_json 형식: {"path":"/absolute/path"} 또는 빈 문자열.
    //           MS-6+ 에서 surface state 스키마가 확장될 경우 이 메서드를 업데이트한다.
    public func activeStatePath() -> String {
        guard let id = activeTabId,
              let tab = tabs.first(where: { $0.id == id })
        else { return "" }
        return extractPath(from: tab)
    }

    /// TabItem 의 stateJson 에서 경로를 추출한다.
    private func extractPath(from tab: TabItem) -> String {
        // TabItem 에는 stateJson 이 없으므로 FFI 에서 재조회한다.
        // 단순화: TabBarViewModel 이 statePath 캐시를 보유하도록 확장 예정.
        // MS-5 에서는 newTab(kind:statePath:) 호출 시 내부 캐시에 저장.
        return statePathCache[tab.id] ?? ""
    }

    // @MX:NOTE: [AUTO] statePathCache: surfaceId → 파일 경로 캐시.
    //           createSurface 시 저장, closeTab 시 제거. MS-5 범위 내 메모리 캐시 허용.
    private(set) var statePathCache: [Int64: String] = [:]

    // MARK: - 내부 헬퍼

    /// JSON 문자열을 파싱하여 TabItem 배열로 변환한다.
    private func parseSurfacesJson(_ json: String) -> [TabItem] {
        guard let data = json.data(using: .utf8),
              let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]]
        else { return [] }

        return arr.compactMap { dict -> TabItem? in
            guard let id = dict["id"] as? Int64 ?? (dict["id"] as? Int).map(Int64.init),
                  let kindStr = dict["kind"] as? String,
                  let tabOrderRaw = dict["tab_order"] as? Int64 ?? (dict["tab_order"] as? Int).map(Int64.init)
            else { return nil }

            let kind = SurfaceKind(rawString: kindStr)
            return TabItem(
                id: id,
                kind: kind,
                title: kind.defaultTitle,
                tabOrder: tabOrderRaw
            )
        }
        .sorted { $0.tabOrder < $1.tabOrder }
    }
}
