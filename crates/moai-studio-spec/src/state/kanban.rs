//! SPEC-V3-009 RG-SU-3 — Kanban stage 타입 (MS-1 scaffold, MS-2 에서 persistence 구현).

/// Kanban 4 stage (REQ-SU-021).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KanbanStage {
    Todo,
    InProgress,
    Review,
    Done,
}

impl KanbanStage {
    /// `.kanban-stage` sidecar 파일의 라벨 문자열로부터 변환 (REQ-SU-021).
    ///
    /// 인식 불가 값은 `Todo` fallback (REQ-SU-021: "unrecognized → fallback to todo").
    pub fn from_sidecar(label: &str) -> Self {
        match label.trim().to_lowercase().as_str() {
            "todo" => KanbanStage::Todo,
            "in-progress" | "in_progress" => KanbanStage::InProgress,
            "review" => KanbanStage::Review,
            "done" => KanbanStage::Done,
            other => {
                tracing::warn!("알 수 없는 kanban-stage '{other}', Todo 로 fallback (REQ-SU-021)");
                KanbanStage::Todo
            }
        }
    }

    /// sidecar 파일에 쓸 라벨 문자열.
    pub fn to_sidecar(&self) -> &'static str {
        match self {
            KanbanStage::Todo => "todo",
            KanbanStage::InProgress => "in-progress",
            KanbanStage::Review => "review",
            KanbanStage::Done => "done",
        }
    }

    /// 다음 stage (REQ-SU-022 순환: Todo → InProgress → Review → Done → Todo).
    pub fn next(&self) -> KanbanStage {
        match self {
            KanbanStage::Todo => KanbanStage::InProgress,
            KanbanStage::InProgress => KanbanStage::Review,
            KanbanStage::Review => KanbanStage::Done,
            KanbanStage::Done => KanbanStage::Todo,
        }
    }

    /// UI 레이블 (lane header 표시용).
    pub fn label(&self) -> &'static str {
        match self {
            KanbanStage::Todo => "TODO",
            KanbanStage::InProgress => "IN-PROGRESS",
            KanbanStage::Review => "REVIEW",
            KanbanStage::Done => "DONE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_sidecar_all_variants() {
        assert_eq!(KanbanStage::from_sidecar("todo"), KanbanStage::Todo);
        assert_eq!(
            KanbanStage::from_sidecar("in-progress"),
            KanbanStage::InProgress
        );
        assert_eq!(KanbanStage::from_sidecar("review"), KanbanStage::Review);
        assert_eq!(KanbanStage::from_sidecar("done"), KanbanStage::Done);
    }

    #[test]
    fn from_sidecar_unknown_falls_back_to_todo() {
        assert_eq!(KanbanStage::from_sidecar("UNKNOWN"), KanbanStage::Todo);
        assert_eq!(KanbanStage::from_sidecar(""), KanbanStage::Todo);
    }

    #[test]
    fn to_sidecar_roundtrip() {
        for stage in [
            KanbanStage::Todo,
            KanbanStage::InProgress,
            KanbanStage::Review,
            KanbanStage::Done,
        ] {
            assert_eq!(KanbanStage::from_sidecar(stage.to_sidecar()), stage);
        }
    }

    #[test]
    fn next_cycles_all_stages() {
        assert_eq!(KanbanStage::Todo.next(), KanbanStage::InProgress);
        assert_eq!(KanbanStage::InProgress.next(), KanbanStage::Review);
        assert_eq!(KanbanStage::Review.next(), KanbanStage::Done);
        assert_eq!(KanbanStage::Done.next(), KanbanStage::Todo); // 순환
    }
}
