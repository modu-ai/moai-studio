//
//  EARSFormatter.swift
//  EARS 패턴 감지 및 HTML 강조 (SPEC-M2-001 MS-5 T-058).
//
//  @MX:NOTE: [AUTO] EARS(Easy Approach to Requirements Syntax) 패턴 변환:
//            - [Ubiquitous] + shall → ears-ubiquitous 클래스 div
//            - [Event-Driven] When ... shall → ears-event 클래스 div
//            - SPEC-XXX-NNN ID → spec-id span
//            Regex 기반 텍스트 치환; HTML 인젝션 방지를 위해 처리 전 HTML 이스케이프 불필요
//            (마크다운 파이프라인 앞단에서 실행).

import Foundation

/// EARS/SPEC 패턴 강조 마킹 유틸리티.
public enum EARSFormatter {

    // MARK: - 공개 API

    /// 마크다운 텍스트에서 EARS 패턴과 SPEC-ID 를 감지하여 강조 마킹을 추가한다.
    ///
    /// - Parameter markdown: 원본 마크다운 문자열.
    /// - Returns: 강조 마킹이 추가된 마크다운 문자열.
    public static func enhance(markdown: String) -> String {
        guard !markdown.isEmpty else { return markdown }

        var result = markdown

        // 1. SPEC-ID 패턴: SPEC-XXX-NNN (예: SPEC-M2-001, SPEC-SLQG-001)
        result = markSpecIds(in: result)

        // 2. Ubiquitous EARS 패턴: [Ubiquitous] ... shall ...
        result = markUbiquitous(in: result)

        // 3. Event-Driven EARS 패턴: [Event-Driven] When ..., ... shall ...
        result = markEventDriven(in: result)

        return result
    }

    // MARK: - 내부 변환

    /// SPEC-ID 를 `<span class="spec-id">` 로 감싼다.
    ///
    /// 패턴: SPEC- + 대문자/숫자 1~6자 + - + 숫자 3자
    /// 예: SPEC-M2-001, SPEC-M1-001, SPEC-SLQG-001
    private static func markSpecIds(in text: String) -> String {
        // @MX:NOTE: [AUTO] 패턴: SPEC-[A-Z0-9]{1,6}-[0-9]{3}
        //           마크다운 내 ``코드 블록``은 처리하지 않음 (MS-5 범위 내 허용).
        let pattern = #"SPEC-[A-Z0-9]{1,6}-[0-9]{3,4}"#
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return text }

        let nsText = text as NSString
        let range = NSRange(location: 0, length: nsText.length)
        var result = text

        // 뒤에서부터 치환하여 range 오프셋 유지
        let matches = regex.matches(in: text, range: range).reversed()
        for match in matches {
            let matchRange = match.range
            let specId = nsText.substring(with: matchRange)
            let replacement = "<span class=\"spec-id\">\(specId)</span>"
            result = (result as NSString).replacingCharacters(in: matchRange, with: replacement)
        }
        return result
    }

    /// **[Ubiquitous]** 패턴을 `<div class="ears-ubiquitous">` 로 감싼다.
    private static func markUbiquitous(in text: String) -> String {
        // 패턴: **[Ubiquitous]** (임의 내용) **shall** (임의 내용)
        let pattern = #"\*\*\[Ubiquitous\]\*\*[^\n]+"#
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return text }

        let nsText = text as NSString
        let range = NSRange(location: 0, length: nsText.length)
        var result = text

        let matches = regex.matches(in: text, range: range).reversed()
        for match in matches {
            let matchRange = match.range
            let original = nsText.substring(with: matchRange)
            let wrapped = "<div class=\"ears-ubiquitous\">\(original)</div>"
            result = (result as NSString).replacingCharacters(in: matchRange, with: wrapped)
        }
        return result
    }

    /// **[Event-Driven]** 패턴을 `<div class="ears-event">` 로 감싼다.
    private static func markEventDriven(in text: String) -> String {
        let pattern = #"\*\*\[Event-Driven\]\*\*[^\n]+"#
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return text }

        let nsText = text as NSString
        let range = NSRange(location: 0, length: nsText.length)
        var result = text

        let matches = regex.matches(in: text, range: range).reversed()
        for match in matches {
            let matchRange = match.range
            let original = nsText.substring(with: matchRange)
            let wrapped = "<div class=\"ears-event\">\(original)</div>"
            result = (result as NSString).replacingCharacters(in: matchRange, with: wrapped)
        }
        return result
    }
}
