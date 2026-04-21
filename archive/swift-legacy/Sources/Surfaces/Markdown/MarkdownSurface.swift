//
//  MarkdownSurface.swift
//  Markdown Surface — AttributedString 렌더링 + EARS 포맷터 + 파일 감시
//  (SPEC-M2-001 MS-5 T-057, T-058, T-059, T-060).
//
//  @MX:ANCHOR: [AUTO] MarkdownViewModel — Markdown 탭 상태의 유일한 소스 (fan_in>=3)
//  @MX:REASON: [AUTO] MarkdownSurface, MarkdownViewModelTests, SurfaceRouter 세 경로에서 참조.
//              파일 로드·렌더링·파일감시 상태를 모두 소유한다.
//
//  @MX:NOTE: [AUTO] 렌더링 방식: Foundation AttributedString(markdown:) + WKWebView 하이브리드.
//            수식(KaTeX)/다이어그램(Mermaid) 전용 WebView 블록, 나머지는 AttributedString.
//            CDN 의존(KaTeX, Mermaid): 오프라인 환경에서 수식/다이어그램 미렌더링.
//            MS-6+ 에서 번들 내 정적 리소스로 교체 예정.
//
//  @MX:WARN: [AUTO] DispatchSource 파일 감시: fileDescriptor 생명주기 관리 필요.
//  @MX:REASON: [AUTO] stopWatching() 이 deinit 전에 호출되지 않으면 fd 누수 발생.
//              ARC 소멸 순서와 DispatchSource cancel 비동기성으로 인한 경쟁조건 방지를 위해
//              stopWatching() 을 명시적으로 호출한다.

import AppKit
import Foundation
import Observation
import SwiftUI
import WebKit

// MARK: - MarkdownViewModel

/// Markdown Surface 의 상태를 관리하는 Observable ViewModel.
///
// @MX:ANCHOR: [AUTO] Markdown 탭 상태 유일 소스 (fan_in>=3)
// @MX:REASON: [AUTO] MarkdownSurface, MarkdownViewModelTests, SurfaceRouter 세 경로 참조
@Observable
@MainActor
public final class MarkdownViewModel {
    /// 원본 마크다운 소스 텍스트.
    public private(set) var source: String = ""

    /// WKWebView 렌더링용 HTML 문자열.
    public private(set) var renderedHTML: String = ""

    /// 파일 절대 경로.
    public let filePath: String

    /// 다크 모드 여부.
    public var isDarkMode: Bool = false

    // MARK: - 파일 감시 (DispatchSource)
    // @MX:WARN: [AUTO] fd 누수 위험: stopWatching() 반드시 호출
    // @MX:REASON: [AUTO] deinit 전 DispatchSource cancel 비동기 완료 보장 위해 명시적 stopWatching 필요
    private var fileDescriptor: Int32 = -1
    private var watchSource: DispatchSourceFileSystemObject?

    // MARK: - 초기화

    public init(filePath: String) {
        self.filePath = filePath
    }

    // MARK: - 공개 메서드

    /// 파일을 읽어 HTML 로 변환한다.
    public func load() async throws {
        let content = try String(contentsOfFile: filePath, encoding: .utf8)
        source = content
        renderedHTML = buildHTML(from: content)
    }

    /// 파일 변경 감지 시 다시 로드한다 (파일 감시 이벤트 핸들러).
    public func reload() async {
        guard let content = try? String(contentsOfFile: filePath, encoding: .utf8) else { return }
        source = content
        renderedHTML = buildHTML(from: content)
    }

    // MARK: - 파일 감시

    /// DispatchSource 기반 파일 감시를 시작한다.
    ///
    // @MX:NOTE: [AUTO] DispatchSourceFileSystemObject 채택 이유:
    //           FSEvents API 는 디렉토리 단위 감시가 기본이므로 단일 파일 감시에 오버스펙.
    //           O_EVTONLY 플래그로 파일 오픈 — 파일 삭제 후 디렉토리 언마운트를 막지 않음.
    //           MS-7+ 에서 moai-fs notify-push 이벤트로 업그레이드 예정.
    func startWatching() {
        fileDescriptor = Darwin.open(filePath, O_EVTONLY)
        guard fileDescriptor >= 0 else { return }

        watchSource = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fileDescriptor,
            eventMask: [.write, .extend],
            queue: .main
        )
        watchSource?.setEventHandler { [weak self] in
            guard let self else { return }
            Task { @MainActor in await self.reload() }
        }
        watchSource?.resume()
    }

    /// 파일 감시를 중지하고 fd 를 닫는다.
    func stopWatching() {
        watchSource?.cancel()
        watchSource = nil
        if fileDescriptor >= 0 {
            Darwin.close(fileDescriptor)
            fileDescriptor = -1
        }
    }

    // MARK: - 내부 헬퍼

    /// 마크다운 소스를 HTML 로 변환한다.
    ///
    /// 변환 전략:
    /// 1. EARSFormatter 로 SPEC-ID/EARS 패턴 강조 마킹
    /// 2. Foundation AttributedString(markdown:) 으로 기본 파싱
    /// 3. NSAttributedString 변환 후 HTML 추출
    /// 4. KaTeX/Mermaid CDN 스크립트가 포함된 HTML 템플릿에 삽입
    private func buildHTML(from markdown: String) -> String {
        // EARS 강조 전처리
        let enhanced = EARSFormatter.enhance(markdown: markdown)

        // AttributedString → HTML 변환 시도
        let bodyHTML: String
        if let attrStr = try? AttributedString(
            markdown: enhanced,
            options: .init(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        ) {
            bodyHTML = attributedStringToHTML(attrStr)
        } else {
            // 폴백: 텍스트를 그대로 pre 태그로 감싼다
            bodyHTML = "<pre>\(escapeHTML(enhanced))</pre>"
        }

        return htmlTemplate(body: bodyHTML, isDark: isDarkMode)
    }

    /// AttributedString 을 간단한 HTML 로 변환한다.
    ///
    // @MX:NOTE: [AUTO] AttributedString → HTML 변환은 Apple 공식 API 미제공.
    //           NSAttributedString.data(from:documentAttributes:) 를 RTF/HTML 로 변환하거나
    //           직접 구현. MS-5 에서는 경량 직접 변환 채택.
    private func attributedStringToHTML(_ attributed: AttributedString) -> String {
        // NSAttributedString 경유 HTML 출력 시도
        let nsAttr = NSAttributedString(attributed)
        let range = NSRange(location: 0, length: nsAttr.length)
        if let data = try? nsAttr.data(
            from: range,
            documentAttributes: [.documentType: NSAttributedString.DocumentType.html]
        ), let html = String(data: data, encoding: .utf8) {
            // body 태그 안쪽만 추출
            if let bodyRange = extractBody(from: html) {
                return bodyRange
            }
        }
        // 폴백: 평문 텍스트
        return "<pre>\(escapeHTML(nsAttr.string))</pre>"
    }

    private func extractBody(from html: String) -> String? {
        guard let start = html.range(of: "<body"),
              let bodyOpen = html.range(of: ">", range: start.upperBound..<html.endIndex),
              let end = html.range(of: "</body>")
        else { return nil }
        return String(html[bodyOpen.upperBound..<end.lowerBound])
    }

    private func escapeHTML(_ text: String) -> String {
        text
            .replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
    }

    /// KaTeX/Mermaid CDN 포함 HTML 템플릿.
    ///
    // @MX:NOTE: [AUTO] KaTeX 0.16.9, Mermaid 10 CDN 의존.
    //           오프라인 환경에서 수식/다이어그램 미렌더링. MS-6+ 번들 내 정적 리소스 교체 예정.
    private func htmlTemplate(body: String, isDark: Bool) -> String {
        let theme = isDark ? "dark" : "light"
        return """
        <!DOCTYPE html>
        <html data-theme="\(theme)">
        <head>
        <meta charset="UTF-8">
        <link rel="stylesheet"
              href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">
        <script defer
                src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
        <script defer
                src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"
                onload="renderMathInElement(document.body);"></script>
        <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
        <script>mermaid.initialize({startOnLoad:true,theme:'\(theme)'});</script>
        <style>
          body { font-family: -apple-system, sans-serif; padding: 16px; }
          .spec-id { background: #e8f4fd; color: #0366d6; padding: 1px 4px;
                     border-radius: 3px; font-family: monospace; }
          .ears-ubiquitous { border-left: 3px solid #28a745; padding-left: 8px;
                             background: #f0fff4; margin: 4px 0; }
          .ears-event { border-left: 3px solid #0366d6; padding-left: 8px;
                        background: #f1f8ff; margin: 4px 0; }
          [data-theme="dark"] body { background: #1e1e1e; color: #d4d4d4; }
          [data-theme="dark"] .spec-id { background: #1e3a5f; color: #79c0ff; }
        </style>
        </head>
        <body class="markdown-body">\(body)</body>
        </html>
        """
    }
}

// MARK: - MarkdownSurface

/// Markdown Surface SwiftUI 뷰 (T-057).
public struct MarkdownSurface: View {
    @State private var viewModel: MarkdownViewModel

    public init(filePath: String) {
        self._viewModel = State(wrappedValue: MarkdownViewModel(filePath: filePath))
    }

    public var body: some View {
        MarkdownWebView(html: viewModel.renderedHTML, isDarkMode: viewModel.isDarkMode)
            .task {
                try? await viewModel.load()
                viewModel.startWatching()
            }
            .onDisappear {
                viewModel.stopWatching()
            }
    }
}

// MARK: - SurfaceProtocol 준수

extension MarkdownSurface: SurfaceProtocol {
    public var surfaceKind: SurfaceKind { .markdown }

    public var toolbarItems: [SurfaceToolbarItem] {
        [
            SurfaceToolbarItem(id: "reload", label: "Reload", systemImage: "arrow.clockwise")
        ]
    }
}

// MARK: - MarkdownWebView (WKWebView NSViewRepresentable)

/// HTML 문자열을 WKWebView 로 렌더링하는 SwiftUI 래퍼.
struct MarkdownWebView: NSViewRepresentable {
    let html: String
    let isDarkMode: Bool

    func makeNSView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        let webView = WKWebView(frame: .zero, configuration: config)
        webView.setValue(false, forKey: "drawsBackground")
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        guard !html.isEmpty else { return }
        webView.loadHTMLString(html, baseURL: nil)
    }
}
