//! SPEC-V3-006 RG-MV-6: 가상 스크롤 자료구조.
//!
//! MS-1 에서는 `VirtualScroll` 자료구조와 `visible_range()` 계산만 제공한다.
//! 실제 GPUI 가상화 마운트 (element pool) 는 MS-3 T25 에서 통합된다.

use std::ops::Range;

// @MX:NOTE: [AUTO] virtual-scroll-line-height-default
// MS-1 에서 line_height_px = 0.0 (기본값) 이면 최대 50 라인 fallback 을 반환한다.
// MS-3 T25 에서 design token 18.0px 로 고정되거나 동적 측정값으로 교체된다.

/// 가상 스크롤 자료구조 — 뷰포트에 보이는 라인 범위만 계산한다 (REQ-MV-060).
///
/// line_height_px 가 0.0 이면 전체 혹은 최대 50 라인의 기본 범위를 반환한다.
#[derive(Default, Clone, Copy)]
pub struct VirtualScroll {
    /// 전체 라인 수
    pub line_count: usize,
    /// 한 라인의 높이 (픽셀 단위). 0.0 이면 fallback 동작.
    pub line_height_px: f32,
    /// 뷰포트 상단의 스크롤 오프셋 (픽셀 단위)
    pub scroll_offset_px: f32,
    /// 뷰포트 높이 (픽셀 단위). 0.0 이면 fallback 동작.
    pub viewport_height_px: f32,
}

impl VirtualScroll {
    /// 현재 뷰포트에서 보이는 라인 인덱스 범위를 반환한다 (REQ-MV-060, REQ-MV-061).
    ///
    /// - line_height_px <= 0.0 이면 `0..min(50, line_count)` 를 반환한다.
    /// - 그 외에는 스크롤 오프셋 / 뷰포트 크기 기반으로 계산한다.
    /// - 항상 line_count 를 넘지 않도록 clamp 된다.
    pub fn visible_range(&self) -> Range<usize> {
        if self.line_height_px <= 0.0 {
            // fallback: 최대 50 라인
            return 0..self.line_count.min(50);
        }
        let first = (self.scroll_offset_px / self.line_height_px).floor() as usize;
        // 여분 2 라인을 추가해 렌더 경계 flickering 방지
        let count = (self.viewport_height_px / self.line_height_px).ceil() as usize + 2;
        first..(first + count).min(self.line_count)
    }

    /// 스크롤 오프셋을 설정한다. 음수는 0 으로 clamp.
    pub fn set_scroll(&mut self, offset_px: f32) {
        let max_scroll = if self.line_height_px > 0.0 {
            let total_height = self.line_count as f32 * self.line_height_px;
            (total_height - self.viewport_height_px).max(0.0)
        } else {
            0.0
        };
        self.scroll_offset_px = offset_px.clamp(0.0, max_scroll);
    }
}

// ============================================================
// 단위 테스트 — T5 (AC-MV-8 선행)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scroll(
        line_count: usize,
        line_height: f32,
        offset: f32,
        viewport: f32,
    ) -> VirtualScroll {
        VirtualScroll {
            line_count,
            line_height_px: line_height,
            scroll_offset_px: offset,
            viewport_height_px: viewport,
        }
    }

    #[test]
    fn visible_range_at_top_starts_at_zero() {
        // 스크롤 없음 → 첫 라인이 0 번 인덱스
        let vs = make_scroll(100, 18.0, 0.0, 360.0);
        let r = vs.visible_range();
        assert_eq!(r.start, 0);
        // viewport 360px / 18px = 20 라인 + 2 여분 = 22 라인
        assert_eq!(r.end, 22);
    }

    #[test]
    fn visible_range_in_middle_calculates_correctly() {
        // offset=180px, line_height=18px → first=10
        let vs = make_scroll(100, 18.0, 180.0, 360.0);
        let r = vs.visible_range();
        assert_eq!(r.start, 10);
        assert_eq!(r.end, 32); // 10 + 22
    }

    #[test]
    fn visible_range_at_end_clamps_to_line_count() {
        // 마지막 근처에서 line_count 를 넘지 않아야 한다
        let vs = make_scroll(25, 18.0, 100_000.0, 360.0);
        let r = vs.visible_range();
        assert!(r.end <= 25, "end={} must be <= 25", r.end);
    }

    #[test]
    fn visible_range_with_zero_line_height_returns_default() {
        // line_height = 0.0 → fallback 최대 50 라인
        let vs = make_scroll(200, 0.0, 0.0, 0.0);
        let r = vs.visible_range();
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 50); // min(200, 50)
    }

    #[test]
    fn visible_range_zero_height_caps_at_line_count() {
        // line_count < 50 인 경우 line_count 까지만
        let vs = make_scroll(10, 0.0, 0.0, 0.0);
        let r = vs.visible_range();
        assert_eq!(r.end, 10);
    }

    #[test]
    fn set_scroll_negative_clamps_to_zero() {
        let mut vs = make_scroll(100, 18.0, 500.0, 360.0);
        vs.set_scroll(-100.0);
        assert_eq!(vs.scroll_offset_px, 0.0);
    }

    #[test]
    fn set_scroll_beyond_max_clamps_to_max() {
        // total_height = 100 * 18 = 1800, viewport = 360 → max_scroll = 1440
        let mut vs = make_scroll(100, 18.0, 0.0, 360.0);
        vs.set_scroll(99_999.0);
        assert_eq!(vs.scroll_offset_px, 1440.0);
    }
}
