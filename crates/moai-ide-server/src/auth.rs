//! 인증 토큰 생성 모듈

use ring::rand::{SecureRandom, SystemRandom};

/// 암호학적으로 안전한 32바이트 hex 인증 토큰을 생성한다.
///
/// # Panics
/// 시스템 RNG 실패 시 패닉 (정상 환경에서는 발생하지 않음)
pub fn generate_auth_token() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("RNG 초기화 실패");
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 생성된 토큰이 64자 hex 문자열인지 검증한다
    #[test]
    fn test_generate_auth_token_length() {
        let token = generate_auth_token();
        assert_eq!(token.len(), 64, "토큰은 반드시 64자 hex여야 한다");
        assert!(
            token.chars().all(|c| c.is_ascii_hexdigit()),
            "토큰은 hex 문자만 포함해야 한다"
        );
    }

    /// 연속 호출 시 서로 다른 토큰이 생성되는지 검증한다
    #[test]
    fn test_generate_auth_token_uniqueness() {
        let token1 = generate_auth_token();
        let token2 = generate_auth_token();
        assert_ne!(token1, token2, "토큰은 매 호출마다 고유해야 한다");
    }
}
