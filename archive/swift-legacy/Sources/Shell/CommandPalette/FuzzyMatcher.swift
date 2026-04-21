//
//  FuzzyMatcher.swift
//  Command Palette 퍼지 검색 (SPEC-M2-001 MS-6 T-069).
//
//  @MX:ANCHOR: [AUTO] Command Palette 검색 알고리즘의 유일한 진입점 (fan_in>=3)
//  @MX:REASON: [AUTO] CommandPaletteController.refreshResults(), 테스트, 향후 커스텀 검색에서 참조.
//
//  @MX:NOTE: [AUTO] 알고리즘 설명:
//            1. 빈 쿼리 → 모든 명령어를 기본 점수(0.5)로 반환
//            2. 퍼지 매칭: title + subtitle + keywords 를 공백 합산한 haystack 에서
//               query 문자를 순서대로 찾는 subsequence match
//            3. 점수 = matches / (1 + gaps) + 접두사 보너스(0.5)
//            4. 내림차순 정렬

import Foundation

// MARK: - FuzzyMatcher

public enum FuzzyMatcher {

    // MARK: - Match 결과

    public struct Match: Sendable {
        public let score: Double
        public let command: PaletteCommand
    }

    // MARK: - 공개 API

    /// query 로 commands 를 퍼지 매칭하여 점수 내림차순으로 반환한다.
    ///
    /// - Parameters:
    ///   - query: 검색어 (빈 문자열이면 전체 반환)
    ///   - commands: 검색 대상 명령어 목록
    /// - Returns: 매칭된 결과 배열 (점수 내림차순)
    public static func match(query: String, commands: [PaletteCommand]) -> [Match] {
        guard !query.isEmpty else {
            // @MX:NOTE: [AUTO] 빈 쿼리 → 기본 점수 0.5 로 전체 반환 (최근 사용 정렬 용도로 예약)
            return commands.map { Match(score: 0.5, command: $0) }
        }
        return commands.compactMap { cmd in
            score(query: query, command: cmd).map { Match(score: $0, command: cmd) }
        }.sorted { $0.score > $1.score }
    }

    // MARK: - 내부 점수 계산

    /// 단일 명령어에 대해 query 의 매칭 점수를 반환한다.
    ///
    /// 매칭 실패(모든 문자를 순서대로 찾지 못함) 시 nil 반환.
    private static func score(query: String, command: PaletteCommand) -> Double? {
        // @MX:NOTE: [AUTO] haystack = title + subtitle + keywords 를 공백으로 합산
        let haystack = ([command.title, command.subtitle ?? ""] + command.keywords)
            .joined(separator: " ")
            .lowercased()
        let needle = query.lowercased()

        // Subsequence 매칭: needle 의 모든 문자가 haystack 에 순서대로 존재해야 한다
        var hIdx = haystack.startIndex
        var matches = 0
        var gaps = 0

        for nChar in needle {
            guard let found = haystack[hIdx...].firstIndex(of: nChar) else {
                return nil  // 매칭 실패
            }
            gaps += haystack.distance(from: hIdx, to: found)
            hIdx = haystack.index(after: found)
            matches += 1
        }

        // @MX:NOTE: [AUTO] 점수 공식: matches / (1 + gaps) + prefixBonus
        //           gaps 가 0 이면 연속 매칭 = 최고 점수, gaps 가 클수록 점수 하락
        let prefixBonus: Double = command.title.lowercased().hasPrefix(needle) ? 0.5 : 0
        return Double(matches) / Double(1 + gaps) + prefixBonus
    }
}
