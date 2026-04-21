//
//  WindowStateStore.swift
//  윈도우 크기/사이드바 너비 UserDefaults 저장·복원 (RG-M1-1 / T-022).
//
//  @MX:ANCHOR: 윈도우 상태 영속화 단일 지점 (fan_in>=3: MainWindow, RootSplitView, App)
//  @MX:REASON: 재실행 시 크기·사이드바 너비 복원은 여러 뷰에서 참조하므로 단일 저장소로 관리.
//

import SwiftUI
import Observation

/// 윈도우 사이즈와 사이드바 너비를 UserDefaults 에 저장·복원.
///
/// - 사이드바 너비는 200..400 범위로 클램프된다 (spec §RG-M1-6).
/// - 윈도우 크기는 최소 800x500 을 보장한다.
@Observable
@MainActor
public final class WindowStateStore {
    public static let sidebarMinWidth: CGFloat = 200
    public static let sidebarMaxWidth: CGFloat = 400
    public static let defaultSidebarWidth: CGFloat = 250

    public static let defaultWindowWidth: CGFloat = 1280
    public static let defaultWindowHeight: CGFloat = 800
    public static let minWindowWidth: CGFloat = 800
    public static let minWindowHeight: CGFloat = 500

    private enum Keys {
        static let windowWidth = "moai.studio.window.width"
        static let windowHeight = "moai.studio.window.height"
        static let sidebarWidth = "moai.studio.sidebar.width"
    }

    private let defaults: UserDefaults

    public private(set) var windowWidth: CGFloat
    public private(set) var windowHeight: CGFloat
    public private(set) var sidebarWidth: CGFloat

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults

        let storedW = defaults.double(forKey: Keys.windowWidth)
        let storedH = defaults.double(forKey: Keys.windowHeight)
        let storedS = defaults.double(forKey: Keys.sidebarWidth)

        self.windowWidth = storedW > 0 ? CGFloat(storedW) : Self.defaultWindowWidth
        self.windowHeight = storedH > 0 ? CGFloat(storedH) : Self.defaultWindowHeight
        self.sidebarWidth = storedS > 0
            ? Self.clampSidebar(CGFloat(storedS))
            : Self.defaultSidebarWidth
    }

    /// 사이드바 너비를 spec 범위로 클램프하여 설정.
    public func setSidebarWidth(_ width: CGFloat) {
        sidebarWidth = Self.clampSidebar(width)
    }

    /// 윈도우 크기 업데이트 (최소값 보장).
    public func setWindowSize(width: CGFloat, height: CGFloat) {
        windowWidth = max(Self.minWindowWidth, width)
        windowHeight = max(Self.minWindowHeight, height)
    }

    /// 현재 상태를 UserDefaults 에 저장.
    public func persist() {
        defaults.set(Double(windowWidth), forKey: Keys.windowWidth)
        defaults.set(Double(windowHeight), forKey: Keys.windowHeight)
        defaults.set(Double(sidebarWidth), forKey: Keys.sidebarWidth)
    }

    /// 테스트 용 초기화.
    public func reset() {
        defaults.removeObject(forKey: Keys.windowWidth)
        defaults.removeObject(forKey: Keys.windowHeight)
        defaults.removeObject(forKey: Keys.sidebarWidth)
        windowWidth = Self.defaultWindowWidth
        windowHeight = Self.defaultWindowHeight
        sidebarWidth = Self.defaultSidebarWidth
    }

    /// 200..400 범위로 클램프.
    public static func clampSidebar(_ v: CGFloat) -> CGFloat {
        min(max(v, sidebarMinWidth), sidebarMaxWidth)
    }
}
