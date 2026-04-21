// swift-tools-version: 6.0
import PackageDescription

// Rust 정적 라이브러리 경로 (core/target/release/libmoai_ffi.a)
// Package.swift 기준 상대 경로: ../core/target/release
let rustLibPath = "../core/target/release"

let package = Package(
    name: "MoAIStudio",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "MoAIStudio", targets: ["MoAIStudio"]),
    ],
    targets: [
        // C 타겟: Rust FFI 헤더 노출 + 정적 라이브러리 링크
        .target(
            name: "MoaiCoreFFI",
            path: "Sources/MoaiCoreFFI",
            publicHeadersPath: "include",
            linkerSettings: [
                // libmoai_ffi.a 를 링크
                .linkedLibrary("moai_ffi"),
                // macOS 시스템 프레임워크: Rust 런타임이 필요로 함
                .linkedFramework("Security"),
                .linkedFramework("CoreFoundation"),
                // Rust 정적 라이브러리 탐색 경로
                .unsafeFlags(["-L\(rustLibPath)"]),
            ]
        ),

        // Swift 래퍼: C FFI를 안전한 Swift API로 감쌈
        .target(
            name: "MoaiCore",
            dependencies: ["MoaiCoreFFI"],
            path: "Sources/MoaiCore"
        ),

        // macOS CLI 앱: FFI 브릿지 검증
        .executableTarget(
            name: "MoAIStudio",
            dependencies: ["MoaiCore"],
            path: "Sources/MoAIStudio"
        ),
    ]
)
