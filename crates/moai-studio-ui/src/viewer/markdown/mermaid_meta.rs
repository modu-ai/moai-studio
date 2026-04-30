//! SPEC-V3-006 MS-4 — Mermaid diagram metadata extraction.
//!
//! Detects the diagram type from the first non-empty line so the placeholder
//! header can show "MERMAID (flowchart)" instead of just "MERMAID". Full
//! Mermaid rendering remains deferred to wry WebView (REQ-MV-011 USER-DECISION).

/// Recognised Mermaid diagram types per the official Mermaid spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MermaidDiagramType {
    Flowchart,
    Sequence,
    Class,
    State,
    Er,
    Journey,
    Gantt,
    Pie,
    Mindmap,
    Timeline,
    Gitgraph,
    /// Unknown or empty — first token did not match any known type.
    Unknown,
}

impl MermaidDiagramType {
    /// Human-readable label suitable for the placeholder header.
    pub fn label(self) -> &'static str {
        match self {
            Self::Flowchart => "flowchart",
            Self::Sequence => "sequenceDiagram",
            Self::Class => "classDiagram",
            Self::State => "stateDiagram",
            Self::Er => "erDiagram",
            Self::Journey => "journey",
            Self::Gantt => "gantt",
            Self::Pie => "pie",
            Self::Mindmap => "mindmap",
            Self::Timeline => "timeline",
            Self::Gitgraph => "gitGraph",
            Self::Unknown => "diagram",
        }
    }
}

/// Detects the Mermaid diagram type from the source code.
///
/// Inspects the first non-empty line. `graph` and `flowchart` keywords both
/// map to `Flowchart`. Direction modifiers (TD, LR, etc.) are ignored.
pub fn detect_diagram_type(source: &str) -> MermaidDiagramType {
    let first_line = source
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");

    let first_token = first_line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    match first_token.as_str() {
        "graph" | "flowchart" => MermaidDiagramType::Flowchart,
        "sequencediagram" => MermaidDiagramType::Sequence,
        "classdiagram" | "classdiagram-v2" => MermaidDiagramType::Class,
        "statediagram" | "statediagram-v2" => MermaidDiagramType::State,
        "erdiagram" => MermaidDiagramType::Er,
        "journey" => MermaidDiagramType::Journey,
        "gantt" => MermaidDiagramType::Gantt,
        "pie" => MermaidDiagramType::Pie,
        "mindmap" => MermaidDiagramType::Mindmap,
        "timeline" => MermaidDiagramType::Timeline,
        "gitgraph" => MermaidDiagramType::Gitgraph,
        _ => MermaidDiagramType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flowchart_via_graph_keyword() {
        assert_eq!(
            detect_diagram_type("graph TD\nA-->B"),
            MermaidDiagramType::Flowchart
        );
    }

    #[test]
    fn flowchart_via_flowchart_keyword() {
        assert_eq!(
            detect_diagram_type("flowchart LR\nA-->B"),
            MermaidDiagramType::Flowchart
        );
    }

    #[test]
    fn sequence_diagram() {
        assert_eq!(
            detect_diagram_type("sequenceDiagram\nA->>B: hi"),
            MermaidDiagramType::Sequence
        );
    }

    #[test]
    fn class_diagram() {
        assert_eq!(
            detect_diagram_type("classDiagram\nClass01 <|-- Class02"),
            MermaidDiagramType::Class
        );
    }

    #[test]
    fn class_diagram_v2() {
        assert_eq!(
            detect_diagram_type("classDiagram-v2"),
            MermaidDiagramType::Class
        );
    }

    #[test]
    fn er_diagram() {
        assert_eq!(
            detect_diagram_type("erDiagram\nCUSTOMER ||--o{ ORDER : places"),
            MermaidDiagramType::Er
        );
    }

    #[test]
    fn gantt_chart() {
        assert_eq!(
            detect_diagram_type("gantt\ndateFormat YYYY-MM-DD"),
            MermaidDiagramType::Gantt
        );
    }

    #[test]
    fn pie_chart() {
        assert_eq!(
            detect_diagram_type("pie\n\"A\" : 30"),
            MermaidDiagramType::Pie
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            detect_diagram_type("FLOWCHART TD"),
            MermaidDiagramType::Flowchart
        );
        assert_eq!(
            detect_diagram_type("SequenceDiagram"),
            MermaidDiagramType::Sequence
        );
    }

    #[test]
    fn skips_empty_leading_lines() {
        assert_eq!(
            detect_diagram_type("\n\n  \ngraph TD"),
            MermaidDiagramType::Flowchart
        );
    }

    #[test]
    fn unknown_for_empty_source() {
        assert_eq!(detect_diagram_type(""), MermaidDiagramType::Unknown);
    }

    #[test]
    fn unknown_for_unrecognised_keyword() {
        assert_eq!(
            detect_diagram_type("randomKeyword something"),
            MermaidDiagramType::Unknown
        );
    }

    #[test]
    fn label_consistency() {
        assert_eq!(MermaidDiagramType::Flowchart.label(), "flowchart");
        assert_eq!(MermaidDiagramType::Sequence.label(), "sequenceDiagram");
        assert_eq!(MermaidDiagramType::Unknown.label(), "diagram");
    }
}
