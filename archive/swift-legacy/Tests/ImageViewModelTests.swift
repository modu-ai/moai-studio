//
//  ImageViewModelTests.swift
//  Image Surface ViewModel 테스트 (SPEC-M2-001 MS-5 T-066).
//

import XCTest
@testable import MoAIStudio

@MainActor
final class ImageViewModelTests: XCTestCase {

    // MARK: - 인라인 PNG 픽셀 데이터 (1×1 빨간 픽셀)

    /// 최소 PNG 바이트 (1×1 빨간 픽셀).
    private static let minimalPNGData: Data = {
        // PNG 헤더 + IHDR + IDAT(빨간 픽셀) + IEND
        // 실제 유효한 1×1 PNG 파일 바이트열
        let bytes: [UInt8] = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
            0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
            0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC,
            0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
            0x44, 0xAE, 0x42, 0x60, 0x82
        ]
        return Data(bytes)
    }()

    private func makeTempPNG() throws -> String {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".png")
        try Self.minimalPNGData.write(to: url)
        return url.path
    }

    private func cleanup(path: String) {
        try? FileManager.default.removeItem(atPath: path)
    }

    // MARK: - 초기 상태

    func test_init_imageIsNil() {
        let vm = ImageViewModel(filePath: "/a.png")
        XCTAssertNil(vm.image)
    }

    func test_init_zoomDefaultOne() {
        let vm = ImageViewModel(filePath: "/a.png")
        XCTAssertEqual(vm.zoom, 1.0, accuracy: 0.001)
    }

    func test_init_panDefaultZero() {
        let vm = ImageViewModel(filePath: "/a.png")
        XCTAssertEqual(vm.pan.x, 0.0, accuracy: 0.001)
        XCTAssertEqual(vm.pan.y, 0.0, accuracy: 0.001)
    }

    // MARK: - load()

    func test_load_validPNG_setsImage() throws {
        // Arrange
        let path = try makeTempPNG()
        defer { cleanup(path: path) }
        let vm = ImageViewModel(filePath: path)

        // Act
        vm.load()

        // Assert
        XCTAssertNotNil(vm.image)
    }

    func test_load_nonexistentFile_imageRemainsNil() {
        // Arrange
        let vm = ImageViewModel(filePath: "/nonexistent/img.png")

        // Act
        vm.load()

        // Assert
        XCTAssertNil(vm.image)
    }

    // MARK: - resetZoom()

    func test_resetZoom_setsZoomToOne() {
        // Arrange
        let vm = ImageViewModel(filePath: "/a.png")
        vm.zoom = 2.5

        // Act
        vm.resetZoom()

        // Assert
        XCTAssertEqual(vm.zoom, 1.0, accuracy: 0.001)
    }

    func test_resetZoom_resetsPan() {
        // Arrange
        let vm = ImageViewModel(filePath: "/a.png")
        vm.pan = CGPoint(x: 100, y: 50)

        // Act
        vm.resetZoom()

        // Assert
        XCTAssertEqual(vm.pan.x, 0.0, accuracy: 0.001)
        XCTAssertEqual(vm.pan.y, 0.0, accuracy: 0.001)
    }

    // MARK: - zoom / pan 변경

    func test_zoom_canBeModified() {
        let vm = ImageViewModel(filePath: "/a.png")
        vm.zoom = 3.0
        XCTAssertEqual(vm.zoom, 3.0, accuracy: 0.001)
    }

    func test_pan_canBeModified() {
        let vm = ImageViewModel(filePath: "/a.png")
        vm.pan = CGPoint(x: 50, y: -30)
        XCTAssertEqual(vm.pan.x, 50.0, accuracy: 0.001)
        XCTAssertEqual(vm.pan.y, -30.0, accuracy: 0.001)
    }
}
