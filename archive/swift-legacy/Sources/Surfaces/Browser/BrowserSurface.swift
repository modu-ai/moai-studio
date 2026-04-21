//
//  BrowserSurface.swift
//  Browser Surface — WKWebView + URL 바 + 네비게이션 + 개발 서버 자동 감지
//  (SPEC-M2-001 MS-5 T-063, T-064, T-065).
//
//  @MX:ANCHOR: [AUTO] BrowserViewModel — Browser 탭 상태의 유일한 소스 (fan_in>=3)
//  @MX:REASON: [AUTO] BrowserSurface, BrowserViewModelTests, SurfaceRouter 세 경로에서 참조.
//
//  @MX:NOTE: [AUTO] 링크 처리 정책 (T-065):
//            localhost / 127.x.x.x / 상대 URL → WKWebView 내부 탐색 허용.
//            외부 URL (https://example.com) → NSWorkspace.open() 으로 기본 브라우저 위임.

import AppKit
import Foundation
import Observation
import SwiftUI
import WebKit

// MARK: - BrowserViewModel

/// Browser Surface 상태를 관리하는 Observable ViewModel.
///
// @MX:ANCHOR: [AUTO] Browser 탭 상태 유일 소스 (fan_in>=3)
// @MX:REASON: [AUTO] BrowserSurface, BrowserViewModelTests, SurfaceRouter 참조
@Observable
@MainActor
public final class BrowserViewModel {
    /// 현재 URL 문자열 (URL 바 바인딩 및 load 트리거에 사용).
    public var currentURL: String = ""

    /// 뒤로 이동 가능 여부 (WKWebView 상태 반영).
    public private(set) var canGoBack: Bool = false

    /// 앞으로 이동 가능 여부 (WKWebView 상태 반영).
    public private(set) var canGoForward: Bool = false

    /// 로딩 중 여부 (WKWebView 상태 반영).
    public private(set) var isLoading: Bool = false

    // WKWebView 참조 (BrowserWebViewRepresentable 에서 주입)
    weak var webView: WKWebView?

    // MARK: - 공개 메서드

    /// URL 을 로드한다. 빈 문자열은 무시.
    public func load(_ url: String) {
        currentURL = url
        guard !url.isEmpty,
              let parsedURL = URL(string: url),
              let webView
        else { return }
        webView.load(URLRequest(url: parsedURL))
    }

    /// 뒤로 이동.
    public func goBack() {
        webView?.goBack()
    }

    /// 앞으로 이동.
    public func goForward() {
        webView?.goForward()
    }

    /// 현재 페이지 새로고침.
    public func reload() {
        webView?.reload()
    }

    // MARK: - 상태 업데이트 (WKWebView delegate 에서 호출)

    /// WKWebView 네비게이션 상태를 동기화한다.
    public func setNavigationState(canGoBack: Bool, canGoForward: Bool) {
        self.canGoBack = canGoBack
        self.canGoForward = canGoForward
    }

    /// 로딩 상태를 동기화한다.
    public func setLoading(_ loading: Bool) {
        isLoading = loading
    }
}

// MARK: - BrowserSurface

/// Browser Surface SwiftUI 뷰 (T-063).
public struct BrowserSurface: View {
    @State private var viewModel = BrowserViewModel()

    public init() {}

    public var body: some View {
        VStack(spacing: 0) {
            // URL 바
            HStack(spacing: 4) {
                Button(action: viewModel.goBack) {
                    Image(systemName: "chevron.left")
                }
                .disabled(!viewModel.canGoBack)
                .buttonStyle(.plain)

                Button(action: viewModel.goForward) {
                    Image(systemName: "chevron.right")
                }
                .disabled(!viewModel.canGoForward)
                .buttonStyle(.plain)

                Button(action: viewModel.reload) {
                    Image(systemName: "arrow.clockwise")
                }
                .buttonStyle(.plain)

                TextField("URL 입력", text: $viewModel.currentURL)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit {
                        viewModel.load(viewModel.currentURL)
                    }
            }
            .padding(6)

            Divider()

            // WebView 콘텐츠
            BrowserWebViewRepresentable(viewModel: viewModel)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .task {
            // 개발 서버 자동 감지
            if let port = await DevServerDetector.detect() {
                let url = "http://localhost:\(port)"
                viewModel.load(url)
            }
        }
    }
}

// MARK: - SurfaceProtocol 준수

extension BrowserSurface: SurfaceProtocol {
    public var surfaceKind: SurfaceKind { .browser }

    public var toolbarItems: [SurfaceToolbarItem] {
        [
            SurfaceToolbarItem(id: "devtools", label: "DevTools", systemImage: "hammer")
        ]
    }
}

// MARK: - BrowserWebViewRepresentable

/// WKWebView NSViewRepresentable 래퍼 (T-065 링크 처리 포함).
struct BrowserWebViewRepresentable: NSViewRepresentable {
    @Bindable var viewModel: BrowserViewModel

    func makeNSView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        let webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = context.coordinator
        // ViewModel 에 webView 참조 주입
        Task { @MainActor in viewModel.webView = webView }
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        // viewModel.load() 에서 직접 제어하므로 여기서는 no-op
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(viewModel: viewModel)
    }

    // MARK: - Coordinator (WKNavigationDelegate)

    final class Coordinator: NSObject, WKNavigationDelegate {
        let viewModel: BrowserViewModel

        init(viewModel: BrowserViewModel) {
            self.viewModel = viewModel
        }

        /// T-065: 링크 처리 — localhost 는 허용, 외부 URL 은 기본 브라우저로 위임.
        func webView(
            _ webView: WKWebView,
            decidePolicyFor navigationAction: WKNavigationAction,
            decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
        ) {
            guard let url = navigationAction.request.url else {
                decisionHandler(.cancel)
                return
            }

            // localhost / 127.0.0.1 / 로컬 파일 → 내부 탐색
            if url.host == "localhost"
                || url.host?.hasPrefix("127.") == true
                || url.scheme == "file"
                || url.scheme == nil
            {
                decisionHandler(.allow)
            } else {
                // 외부 URL → 기본 브라우저
                NSWorkspace.shared.open(url)
                decisionHandler(.cancel)
            }
        }

        func webView(_ webView: WKWebView, didStartProvisionalNavigation navigation: WKNavigation!) {
            Task { @MainActor in viewModel.setLoading(true) }
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            Task { @MainActor in
                viewModel.setLoading(false)
                viewModel.setNavigationState(
                    canGoBack: webView.canGoBack,
                    canGoForward: webView.canGoForward
                )
                viewModel.currentURL = webView.url?.absoluteString ?? viewModel.currentURL
            }
        }

        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
            Task { @MainActor in viewModel.setLoading(false) }
        }
    }
}
