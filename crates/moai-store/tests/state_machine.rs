//! T-006: 6-state 상태 머신 전이 규칙 검증 (SPEC-M1-001 RG-M1-4).

use moai_store::WorkspaceStatus::*;

#[test]
fn valid_bootup_flow() {
    assert!(Created.can_transition_to(Starting));
    assert!(Starting.can_transition_to(Running));
}

#[test]
fn valid_pause_and_resume() {
    assert!(Running.can_transition_to(Paused));
    assert!(Paused.can_transition_to(Starting));
}

#[test]
fn valid_error_from_any_active_state() {
    assert!(Created.can_transition_to(Error));
    assert!(Starting.can_transition_to(Error));
    assert!(Running.can_transition_to(Error));
    assert!(Paused.can_transition_to(Error));
}

#[test]
fn error_can_restart() {
    assert!(Error.can_transition_to(Starting));
}

#[test]
fn deleted_is_reachable_from_all() {
    for s in [Created, Starting, Running, Paused, Error] {
        assert!(
            s.can_transition_to(Deleted),
            "{s:?} -> Deleted 는 허용되어야 한다"
        );
    }
}

#[test]
fn deleted_is_terminal() {
    for s in [Created, Starting, Running, Paused, Error, Deleted] {
        assert!(
            !Deleted.can_transition_to(s),
            "Deleted 는 어떤 상태로도 전이할 수 없다: -> {s:?}"
        );
    }
}

#[test]
fn invalid_backward_transitions() {
    assert!(!Running.can_transition_to(Created));
    assert!(!Running.can_transition_to(Starting));
    assert!(!Starting.can_transition_to(Created));
    assert!(!Paused.can_transition_to(Running)); // 반드시 Starting 을 경유 (lazy restart)
    assert!(!Error.can_transition_to(Running)); // Error → Starting → Running
}

#[test]
fn self_transitions_are_rejected() {
    // no-op self-transition 은 명시적 거부 (UPDATE 남용 방지)
    for s in [Created, Starting, Running, Paused, Error] {
        assert!(!s.can_transition_to(s), "{s:?} -> {s:?} self 전이 금지");
    }
}

#[test]
fn transition_returns_error_for_invalid() {
    let err = Running.transition(Created).unwrap_err();
    assert_eq!(err.from, Running);
    assert_eq!(err.to, Created);
}

#[test]
fn transition_returns_ok_for_valid() {
    let next = Created.transition(Starting).unwrap();
    assert_eq!(next, Starting);
}

#[test]
fn roundtrip_string_parse() {
    for s in [Created, Starting, Running, Paused, Error, Deleted] {
        let parsed: moai_store::WorkspaceStatus = s.as_str().parse().unwrap();
        assert_eq!(parsed, s);
    }
}
