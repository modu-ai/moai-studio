//! 레이아웃 치수 상수 — `tokens.json` v2.0.0 `ide_layout` + `radius` + `spacing`.
//!
//! @MX:NOTE: [AUTO] design-layout-canonical
//! tokens.json `ide_layout` 섹션의 dimension 값을 GPUI f32 상수로 변환.
//! Source: `.moai/design/tokens.json` + `moai-studio.html` 시안.

// ============================================================
// IDE 앱 셸 치수
// ============================================================

/// IDE 앱 셸 dimension — tokens.json `ide_layout`
pub mod ide {
    /// 상단 툴바 높이 (tokens: `topbar.height` = 38px)
    pub const TOPBAR_HEIGHT_PX: f32 = 38.0;
    /// 탭 바 높이 (tokens: `tabs.height` = 36px)
    pub const TABS_HEIGHT_PX: f32 = 36.0;
    /// 상태 바 높이 (tokens: `status.height` = 24px)
    pub const STATUS_HEIGHT_PX: f32 = 24.0;
    /// 사이드바 기본 너비 (tokens: `sidebar.width` = 240px)
    pub const SIDEBAR_WIDTH_PX: f32 = 240.0;
    /// 사이드바 컴팩트 너비 (tokens: `sidebar.compact` = 56px)
    pub const SIDEBAR_COMPACT_PX: f32 = 56.0;
    /// pane 헤더 높이 (tokens: `pane.head.height` = 28px)
    pub const PANE_HEAD_HEIGHT_PX: f32 = 28.0;
    /// 행 높이 — comfortable (tokens: `row.h.comfortable` = 26px)
    pub const ROW_H_COMFORTABLE_PX: f32 = 26.0;
    /// 행 높이 — compact (tokens: `row.h.compact` = 22px)
    pub const ROW_H_COMPACT_PX: f32 = 22.0;
    /// 코드 거터 너비 (tokens: `code.gutter.width` = 44px)
    pub const CODE_GUTTER_WIDTH_PX: f32 = 44.0;
    /// Markdown @MX 거터 너비 (tokens: `md.gutter.width` = 64px)
    pub const MD_GUTTER_WIDTH_PX: f32 = 64.0;
    /// Markdown 최대 너비 (tokens: `md.maxwidth` = 780px)
    pub const MD_MAX_WIDTH_PX: f32 = 780.0;
    /// Agent 좌측 컬럼 너비 (tokens: `agent.col.left` = 200px)
    pub const AGENT_COL_LEFT_PX: f32 = 200.0;
    /// Agent 우측 컬럼 너비 (tokens: `agent.col.right` = 280px)
    pub const AGENT_COL_RIGHT_PX: f32 = 280.0;
}

// ============================================================
// 반경 (border-radius)
// ============================================================

/// 반경 — tokens.json `radius`
pub mod radius {
    /// 반경 없음 (0px)
    pub const NONE: f32 = 0.0;
    /// 소 (4px)
    pub const SM: f32 = 4.0;
    /// 중 기본값 — 카드/모달/입력 (8px)
    pub const MD: f32 = 8.0;
    /// 대 (16px)
    pub const LG: f32 = 16.0;
    /// 특대 (24px)
    pub const XL: f32 = 24.0;
    /// 알약 (32px)
    pub const PILL: f32 = 32.0;
    /// 완전 원형 (9999px)
    pub const FULL: f32 = 9999.0;
}

// ============================================================
// 간격 (spacing)
// ============================================================

/// 4-base 간격 스케일 — tokens.json `spacing`
pub mod spacing {
    pub const S_0: f32 = 0.0;
    /// 4px
    pub const S_1: f32 = 4.0;
    /// 8px
    pub const S_2: f32 = 8.0;
    /// 12px
    pub const S_3: f32 = 12.0;
    /// 16px
    pub const S_4: f32 = 16.0;
    /// 20px
    pub const S_5: f32 = 20.0;
    /// 24px
    pub const S_6: f32 = 24.0;
    /// 32px
    pub const S_8: f32 = 32.0;
    /// 40px
    pub const S_10: f32 = 40.0;
    /// 48px
    pub const S_12: f32 = 48.0;
    /// 64px
    pub const S_16: f32 = 64.0;
    /// 80px
    pub const S_20: f32 = 80.0;
    /// 96px
    pub const S_24: f32 = 96.0;
    /// 128px
    pub const S_32: f32 = 128.0;
}

// ============================================================
// 단위 테스트 — tokens.json 값 정합 검증
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ide_topbar_height() {
        // tokens.json: `ide_layout.topbar.height` = "38px"
        assert!((ide::TOPBAR_HEIGHT_PX - 38.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ide_tabs_height() {
        // tokens.json: `ide_layout.tabs.height` = "36px"
        assert!((ide::TABS_HEIGHT_PX - 36.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ide_sidebar_width() {
        // tokens.json: `ide_layout.sidebar.width` = "240px"
        assert!((ide::SIDEBAR_WIDTH_PX - 240.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ide_pane_head_height() {
        // tokens.json: `ide_layout.pane.head.height` = "28px"
        assert!((ide::PANE_HEAD_HEIGHT_PX - 28.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ide_md_max_width() {
        // tokens.json: `ide_layout.md.maxwidth` = "780px"
        assert!((ide::MD_MAX_WIDTH_PX - 780.0).abs() < f32::EPSILON);
    }

    #[test]
    fn radius_md_is_8() {
        // tokens.json: `radius.md` = "8px"
        assert!((radius::MD - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn spacing_s_4_is_16() {
        // tokens.json: `spacing.4` = "1rem" = 16px
        assert!((spacing::S_4 - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn spacing_s_8_is_32() {
        // tokens.json: `spacing.8` = "2rem" = 32px
        assert!((spacing::S_8 - 32.0).abs() < f32::EPSILON);
    }
}
