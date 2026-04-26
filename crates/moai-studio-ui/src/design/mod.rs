//! 디자인 토큰 모듈 — `tokens.json` v2.0.0 GPUI Rust 진입점.
//!
//! @MX:ANCHOR: [AUTO] design-module-canonical
//! @MX:REASON: [AUTO] 이 모듈은 `.moai/design/tokens.json` v2.0.0 의 단일 진실 원천(SSOT)이다.
//!   fan_in >= 3: lib.rs (RootView), tabs/container.rs, panes/render.rs, viewer/* 등.
//!   토큰 변경 시 반드시 tokens.json 과 동기화 필수.

pub mod layout;
pub mod tokens;
pub mod typography;

pub use layout::*;
pub use tokens::*;
