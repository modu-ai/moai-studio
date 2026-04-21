//! C-6: 회전형 인증 토큰 (M1 carry-over).
//!
//! @MX:ANCHOR: [AUTO] RotatingAuthToken — hook endpoint 인증 단일 소스 (fan_in>=3)
//! @MX:REASON: [AUTO] HookEndpoint, auth_mw, 외부 설정 생성 3곳에서 사용. 토큰 노출 시 RCE 벡터.
//! @MX:WARN: [AUTO] TTL 만료 전 previous 토큰도 grace period 동안 유효 — 클라이언트 롤오버 보장
//! @MX:REASON: [AUTO] Claude Code가 settings.json에 토큰을 캐싱하므로, 즉시 무효화 시 훅 손실 가능.

use ring::rand::SecureRandom;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

struct TokenState {
    current: String,
    previous: Option<String>,
    issued_at: Instant,
}

/// TTL 기반 회전형 인증 토큰.
///
/// - `current_token()`: 현재 유효한 토큰 반환
/// - `rotate_if_needed()`: TTL 경과 시 토큰 교체 (이전 토큰은 grace period 보관)
/// - `validate(token)`: 현재 또는 이전 토큰과 일치 여부 확인
pub struct RotatingAuthToken {
    inner: Arc<RwLock<TokenState>>,
    ttl: Duration,
}

impl RotatingAuthToken {
    /// 기본 TTL 1시간으로 생성.
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(3600))
    }

    /// 지정 TTL로 생성.
    pub fn with_ttl(ttl: Duration) -> Self {
        let current = generate_token();
        Self {
            inner: Arc::new(RwLock::new(TokenState {
                current,
                previous: None,
                issued_at: Instant::now(),
            })),
            ttl,
        }
    }

    /// 현재 활성 토큰 반환.
    pub fn current_token(&self) -> String {
        self.inner
            .read()
            .expect("RotatingAuthToken RwLock 읽기 실패")
            .current
            .clone()
    }

    /// TTL이 경과했으면 토큰을 교체한다. 반환값: true = 교체됨.
    pub fn rotate_if_needed(&self) -> bool {
        // 읽기 잠금으로 TTL 확인
        {
            let state = self
                .inner
                .read()
                .expect("RotatingAuthToken RwLock 읽기 실패");
            if state.issued_at.elapsed() < self.ttl {
                return false;
            }
        }
        // 쓰기 잠금으로 교체
        let mut state = self
            .inner
            .write()
            .expect("RotatingAuthToken RwLock 쓰기 실패");
        // double-check: 다른 스레드가 이미 교체했을 수 있음
        if state.issued_at.elapsed() < self.ttl {
            return false;
        }
        let new_token = generate_token();
        state.previous = Some(std::mem::replace(&mut state.current, new_token));
        state.issued_at = Instant::now();
        true
    }

    /// 토큰 유효성 검사 (현재 또는 이전 토큰 허용 — grace period).
    pub fn validate(&self, token: &str) -> bool {
        let state = self
            .inner
            .read()
            .expect("RotatingAuthToken RwLock 읽기 실패");
        if state.current == token {
            return true;
        }
        if let Some(ref prev) = state.previous
            && prev == token
        {
            return true;
        }
        false
    }

    /// 강제로 즉시 토큰을 교체한다 (테스트 또는 보안 이벤트 시 사용).
    pub fn force_rotate(&self) -> String {
        let mut state = self
            .inner
            .write()
            .expect("RotatingAuthToken RwLock 쓰기 실패");
        let new_token = generate_token();
        state.previous = Some(std::mem::replace(&mut state.current, new_token.clone()));
        state.issued_at = Instant::now();
        new_token
    }
}

impl Default for RotatingAuthToken {
    fn default() -> Self {
        Self::new()
    }
}

/// ring::SystemRandom 기반 32바이트 토큰 (hex 64자).
fn generate_token() -> String {
    let mut buf = [0u8; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut buf)
        .expect("ring SystemRandom::fill 실패");
    hex::encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_token_is_non_empty() {
        let rot = RotatingAuthToken::new();
        assert_eq!(rot.current_token().len(), 64);
    }

    #[test]
    fn validate_current_token() {
        let rot = RotatingAuthToken::new();
        let tok = rot.current_token();
        assert!(rot.validate(&tok));
    }

    #[test]
    fn validate_rejects_unknown_token() {
        let rot = RotatingAuthToken::new();
        assert!(!rot.validate("invalid-token"));
    }

    #[test]
    fn rotate_if_needed_no_rotation_before_ttl() {
        let rot = RotatingAuthToken::with_ttl(Duration::from_secs(3600));
        let rotated = rot.rotate_if_needed();
        assert!(!rotated);
    }

    #[test]
    fn rotate_if_needed_rotates_after_ttl() {
        let rot = RotatingAuthToken::with_ttl(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        let before = rot.current_token();
        let rotated = rot.rotate_if_needed();
        assert!(rotated);
        let after = rot.current_token();
        assert_ne!(before, after);
    }

    #[test]
    fn previous_token_valid_during_grace_period() {
        let rot = RotatingAuthToken::with_ttl(Duration::from_millis(1));
        let first = rot.current_token();
        std::thread::sleep(Duration::from_millis(5));
        rot.rotate_if_needed();
        // 이전 토큰도 grace period 동안 유효
        assert!(
            rot.validate(&first),
            "이전 토큰이 grace period 동안 유효해야 함"
        );
    }

    #[test]
    fn force_rotate_changes_token() {
        let rot = RotatingAuthToken::new();
        let before = rot.current_token();
        let new_tok = rot.force_rotate();
        assert_ne!(before, new_tok);
        assert_eq!(rot.current_token(), new_tok);
        // 이전 토큰도 여전히 유효
        assert!(rot.validate(&before));
    }
}
