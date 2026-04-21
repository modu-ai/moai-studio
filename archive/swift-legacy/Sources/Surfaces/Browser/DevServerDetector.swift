//
//  DevServerDetector.swift
//  로컬 개발 서버 자동 감지 (SPEC-M2-001 MS-5 T-064).
//
//  @MX:NOTE: [AUTO] 프로브 포트 목록: [3000, 5173, 8080, 4200, 8000, 5000, 8888]
//            Vite(5173), CRA/Next(3000), Rails(3000), Django(8000), Angular(4200) 등 주요 프레임워크 커버.
//            첫 번째 응답하는 포트 반환 (병렬 프로브). 응답 타임아웃 0.5초.
//            추가 포트 감지가 필요한 경우 commonPorts 배열 확장.

import Foundation

/// 로컬 개발 서버를 자동 감지하는 유틸리티.
public enum DevServerDetector {

    /// 감지 대상 포트 목록 (우선순위 순서).
    // @MX:NOTE: [AUTO] Vite=5173, CRA/Next=3000, generic=8080, Angular=4200, Django=8000 커버
    public static let commonPorts: [Int] = [3000, 5173, 8080, 4200, 8000, 5000, 8888]

    /// 첫 번째 응답하는 localhost 포트를 반환한다.
    ///
    /// - Parameter timeout: 각 포트 응답 대기 시간 (기본값 0.5초).
    /// - Returns: 응답하는 포트 번호, 없으면 nil.
    public static func detect(timeout: TimeInterval = 0.5) async -> Int? {
        await withTaskGroup(of: Int?.self) { group in
            for port in commonPorts {
                group.addTask {
                    await probe(port: port, timeout: timeout) ? port : nil
                }
            }
            for await result in group {
                if let port = result {
                    group.cancelAll()
                    return port
                }
            }
            return nil
        }
    }

    /// 단일 포트에 HTTP 요청을 보내 응답 여부를 확인한다.
    static func probe(port: Int, timeout: TimeInterval) async -> Bool {
        guard let url = URL(string: "http://localhost:\(port)") else { return false }
        var request = URLRequest(url: url)
        request.timeoutInterval = timeout
        request.httpMethod = "HEAD"
        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            return (response as? HTTPURLResponse)?.statusCode != nil
        } catch {
            return false
        }
    }
}
