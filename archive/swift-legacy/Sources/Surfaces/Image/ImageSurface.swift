//
//  ImageSurface.swift
//  Image Surface — NSImage + zoom/pan + Vision feature print 유사도 (SPEC-M2-001 MS-5 T-061, T-062).
//
//  @MX:ANCHOR: [AUTO] ImageViewModel — Image 탭 상태의 유일한 소스 (fan_in>=3)
//  @MX:REASON: [AUTO] ImageSurface, ImageViewModelTests, SurfaceRouter 세 경로에서 참조.
//
//  @MX:NOTE: [AUTO] 지원 포맷: NSImage 가 macOS 14+ 에서 PNG/JPEG/GIF/WebP/SVG 를 네이티브 지원.
//            NSImage(contentsOfFile:) 으로 단일 로딩 경로 사용.
//
//  @MX:NOTE: [AUTO] ImageDiffView 의 유사도: Vision VNGenerateImageFeaturePrintRequest 사용.
//            진정한 SSIM 이 아닌 퍼셉추얼 피처 프린트 거리 기반 근사값.
//            distance 범위 0~2 → similarity = max(0, 1 - distance/2).
//            실제 SSIM 구현은 MS-6+ 로 연기. (@MX:NOTE 참조)

import AppKit
import Foundation
import Observation
import SwiftUI
import Vision

// MARK: - ImageViewModel

/// Image Surface 상태를 관리하는 Observable ViewModel.
///
// @MX:ANCHOR: [AUTO] Image 탭 상태 유일 소스 (fan_in>=3)
// @MX:REASON: [AUTO] ImageSurface, ImageViewModelTests, SurfaceRouter 참조
@Observable
@MainActor
public final class ImageViewModel {
    /// 로드된 이미지.
    public private(set) var image: NSImage?

    /// 줌 배율 (1.0 = 원본 크기).
    public var zoom: CGFloat = 1.0

    /// 팬 오프셋.
    public var pan: CGPoint = .zero

    /// 파일 절대 경로.
    public let filePath: String

    // MARK: - 초기화

    public init(filePath: String) {
        self.filePath = filePath
    }

    // MARK: - 공개 메서드

    /// 파일을 읽어 NSImage 를 로드한다.
    public func load() {
        guard FileManager.default.fileExists(atPath: filePath) else { return }
        image = NSImage(contentsOfFile: filePath)
    }

    /// 줌을 1.0 으로, 팬을 .zero 로 리셋한다 (창 크기에 맞춤).
    public func resetZoom() {
        zoom = 1.0
        pan = .zero
    }
}

// MARK: - ImageSurface

/// Image Surface SwiftUI 뷰 (T-061).
public struct ImageSurface: View {
    @State private var viewModel: ImageViewModel

    public init(filePath: String) {
        self._viewModel = State(wrappedValue: ImageViewModel(filePath: filePath))
    }

    public var body: some View {
        GeometryReader { _ in
            if let image = viewModel.image {
                Image(nsImage: image)
                    .resizable()
                    .scaledToFit()
                    .scaleEffect(viewModel.zoom)
                    .offset(x: viewModel.pan.x, y: viewModel.pan.y)
                    .gesture(
                        DragGesture()
                            .onChanged { value in
                                viewModel.pan = CGPoint(
                                    x: value.translation.width,
                                    y: value.translation.height
                                )
                            }
                    )
                    .gesture(
                        MagnificationGesture()
                            .onChanged { scale in
                                viewModel.zoom = scale
                            }
                    )
            } else {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .task { viewModel.load() }
        .toolbar {
            ToolbarItemGroup {
                Button {
                    viewModel.resetZoom()
                } label: {
                    Image(systemName: "arrow.up.left.and.down.right.magnifyingglass")
                }
                .help("원본 크기로 리셋")
            }
        }
    }
}

// MARK: - SurfaceProtocol 준수

extension ImageSurface: SurfaceProtocol {
    public var surfaceKind: SurfaceKind { .image }

    public var toolbarItems: [SurfaceToolbarItem] {
        [
            SurfaceToolbarItem(id: "fit", label: "Fit", systemImage: "arrow.up.left.and.down.right.magnifyingglass")
        ]
    }
}

// MARK: - ImageDiffView (T-062)

/// 두 이미지를 나란히 표시하고 Vision feature print 유사도를 계산하는 뷰.
///
// @MX:NOTE: [AUTO] Vision VNGenerateImageFeaturePrintRequest 기반 퍼셉추얼 유사도.
//           진정한 SSIM(Structural Similarity Index) 이 아닌 근사값.
//           distance 범위 0~2 기준 similarity = max(0, 1 - distance/2) 매핑.
//           SSIM 구현 연기: MS-6+ 작업 예정. 현재는 VNFeaturePrintObservation.computeDistance 사용.
public struct ImageDiffView: View {
    let leftPath: String
    let rightPath: String

    @State private var similarityScore: Double?

    public var body: some View {
        HStack(spacing: 8) {
            ImageSurface(filePath: leftPath)
            Divider()
            ImageSurface(filePath: rightPath)
        }
        .overlay(alignment: .top) {
            if let score = similarityScore {
                Text(String(format: "유사도: %.1f%%", score * 100))
                    .font(.caption)
                    .padding(8)
                    .background(.ultraThinMaterial)
                    .cornerRadius(6)
            }
        }
        .task { await computeSimilarity() }
    }

    // MARK: - Vision 유사도 계산

    private func computeSimilarity() async {
        guard let left = await featurePrint(for: leftPath),
              let right = await featurePrint(for: rightPath)
        else { return }

        do {
            var distance: Float = 0
            try left.computeDistance(&distance, to: right)
            // distance ≈ 0~2; 0 = 동일, 2 = 완전 다름
            let similarity = max(0.0, 1.0 - Double(distance) / 2.0)
            await MainActor.run { similarityScore = similarity }
        } catch {
            // 거리 계산 실패 시 score nil 유지
        }
    }

    /// Vision 피처 프린트를 비동기로 생성한다.
    private func featurePrint(for path: String) async -> VNFeaturePrintObservation? {
        await withCheckedContinuation { continuation in
            guard let image = NSImage(contentsOfFile: path),
                  let tiff = image.tiffRepresentation,
                  let bitmap = NSBitmapImageRep(data: tiff),
                  let cgImage = bitmap.cgImage
            else {
                continuation.resume(returning: nil)
                return
            }

            let request = VNGenerateImageFeaturePrintRequest()
            let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
            do {
                try handler.perform([request])
                let result = request.results?.first as? VNFeaturePrintObservation
                continuation.resume(returning: result)
            } catch {
                continuation.resume(returning: nil)
            }
        }
    }
}
