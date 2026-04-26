//! InstructionsGraphView GPUI Entity — 6-layer instruction tree 렌더 (RG-AD-4, AC-AD-7/8)
//!
//! SPEC-V3-010 REQ-AD-021: indent 기반 layered tree 렌더 (source path + priority).
//! SPEC-V3-010 REQ-AD-022: 노드 클릭 시 OS 기본 editor 로 open.
//! SPEC-V3-010 REQ-AD-023: 본문 편집 금지 (read-only).
//!
//! @MX:ANCHOR: [AUTO] instructions-graph-view-entity
//! @MX:REASON: [AUTO] 6-layer instruction 시각화 단일 진입점. fan_in >= 3:
//!   AgentDashboardView, hook event 루팅, 테스트.
//!   SPEC: SPEC-V3-010 RG-AD-4

use std::path::{Path, PathBuf};
use std::process::Command;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::{
    InstructionNode, InstructionRebuildTrigger, InstructionScanner, ScanPaths,
};

use crate::design::tokens as tok;

/// InstructionsGraphView — 6-layer instruction tree 표시 (REQ-AD-021).
pub struct InstructionsGraphView {
    /// 스캐너 (rebuild 트리거 진입점)
    pub scanner: InstructionScanner,
}

impl InstructionsGraphView {
    /// 프로젝트 루트로부터 새 view 를 생성한다.
    /// 생성 직후에는 트리가 비어 있으며, `rebuild_now` 또는 hook 트리거 호출이 필요하다.
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            scanner: InstructionScanner::new(ScanPaths::from_project_root(project_root)),
        }
    }

    /// 명시적으로 트리 rebuild 를 수행한다.
    pub fn rebuild_now(&mut self, cx: &mut Context<Self>) {
        self.scanner.rebuild();
        cx.notify();
    }

    /// hook event 이름에 따라 트리 rebuild 를 트리거한다 (REQ-AD-019/020).
    /// 트리거가 아닌 hook 이면 noop.
    pub fn rebuild_on_hook(&mut self, hook_name: &str, cx: &mut Context<Self>) {
        if InstructionRebuildTrigger::from_hook_name(hook_name).is_some() {
            self.scanner.rebuild();
            cx.notify();
        }
    }

    /// 현재 캐시된 tree 를 반환한다.
    pub fn tree(&self) -> &[InstructionNode] {
        self.scanner.last_tree()
    }
}

/// 노드 표시용 행 — UI 렌더에 쓸 평탄화된 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionRow {
    pub indent: u8,
    pub label: String,
    pub priority: u8,
    pub source_path: PathBuf,
}

impl InstructionRow {
    fn from_node(node: &InstructionNode, indent: u8) -> Self {
        let label = node
            .source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| node.kind.label())
            .to_string();
        Self {
            indent,
            label,
            priority: node.priority,
            source_path: node.source_path.clone(),
        }
    }
}

/// 트리를 indent 기반 row 목록으로 평탄화한다 (REQ-AD-021).
pub fn flatten_tree(tree: &[InstructionNode]) -> Vec<InstructionRow> {
    let mut rows = Vec::new();
    for node in tree {
        push_node(node, 0, &mut rows);
    }
    rows
}

fn push_node(node: &InstructionNode, depth: u8, rows: &mut Vec<InstructionRow>) {
    rows.push(InstructionRow::from_node(node, depth));
    for child in &node.children {
        push_node(child, depth.saturating_add(1), rows);
    }
}

/// REQ-AD-022: 경로를 OS 기본 editor 로 open 한다.
///
/// macOS: `open <path>` / Linux: `xdg-open <path>` / 그 외: 미지원.
/// 본 함수는 spawn 후 즉시 반환하며 외부 프로세스의 종료를 기다리지 않는다.
pub fn open_in_editor(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = path;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "OS 기본 editor open 은 macOS / Linux 만 지원한다",
        ))
    }
}

impl Render for InstructionsGraphView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let rows = flatten_tree(self.scanner.last_tree());

        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_PANEL))
            .p_3()
            .gap(px(2.))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_MUTED))
                    .child("Instructions"),
            );

        if rows.is_empty() {
            container = container.child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_DISABLED))
                    .child("(empty — SessionStart 진입 대기)"),
            );
            return container;
        }

        for row in rows {
            let indent_px = (row.indent as f32) * 12.0;
            let display = format!("[{}] {}", row.priority, row.label);
            container = container.child(
                div()
                    .flex()
                    .flex_row()
                    .pl(px(indent_px))
                    .gap(px(6.))
                    .py(px(1.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(display),
                    ),
            );
        }

        container
    }
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::InstructionKind;
    use std::fs;

    fn tmp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("moai-spec-v3-010-ui-{}-{}", label, nanos));
        fs::create_dir_all(&dir).expect("temp dir 생성 실패");
        dir
    }

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent 생성 실패");
        }
        fs::write(path, body).expect("파일 쓰기 실패");
    }

    /// AC-AD-7: flatten_tree 가 layered tree 를 indent 1단위씩 표현한다.
    #[test]
    fn flatten_tree_uses_indent_per_depth() {
        let project = tmp_dir("flatten");
        write(&project.join("CLAUDE.md"), "# project");
        write(&project.join(".claude/rules/r1.md"), "# rule");
        write(&project.join(".claude/rules/sub/r2.md"), "# sub-rule");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: None,
            managed_policy_dir: None,
        });
        scanner.rebuild();
        let rows = flatten_tree(scanner.last_tree());

        assert!(!rows.is_empty(), "rebuild 후 rows 가 비어있으면 안된다");
        // 그룹 노드는 indent 0, children 은 indent 1
        let group_row = rows
            .iter()
            .find(|r| r.priority == InstructionKind::Rules.priority())
            .expect("Rules 그룹 누락");
        assert_eq!(group_row.indent, 0);

        let leaf_with_indent = rows.iter().any(|r| r.indent == 1);
        assert!(leaf_with_indent, "rules children 의 indent=1 누락");
    }

    /// AC-AD-8: SessionStart hook 이 rebuild 를 트리거한다 (UI 루팅 검증).
    #[test]
    fn session_start_hook_triggers_rebuild_via_helper() {
        let project = tmp_dir("hook-ui");
        write(&project.join("CLAUDE.md"), "# project");

        let mut view = InstructionsGraphView::new(project);
        assert_eq!(view.scanner.rebuild_count(), 0);

        // Context 가 없는 단위 테스트 환경에서는 scanner 직접 검증 (rebuild_on_hook 의 핵심
        // 분기 로직만 확인). cx.notify() 는 GPUI 통합 테스트에서 검증.
        if InstructionRebuildTrigger::from_hook_name("SessionStart").is_some() {
            view.scanner.rebuild();
        }
        assert_eq!(view.scanner.rebuild_count(), 1);

        // 트리거가 아닌 hook 은 rebuild 하지 않아야 한다
        if InstructionRebuildTrigger::from_hook_name("PostToolUse").is_some() {
            view.scanner.rebuild();
        }
        assert_eq!(
            view.scanner.rebuild_count(),
            1,
            "PostToolUse 는 트리거 아님"
        );
    }

    /// AC-AD-7: row label 이 파일 이름을 표시한다.
    #[test]
    fn row_label_uses_file_name() {
        let project = tmp_dir("label");
        write(&project.join("CLAUDE.md"), "# project");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: None,
            managed_policy_dir: None,
        });
        scanner.rebuild();
        let rows = flatten_tree(scanner.last_tree());

        let project_row = rows
            .iter()
            .find(|r| r.priority == InstructionKind::Project.priority())
            .expect("Project row 누락");
        assert_eq!(project_row.label, "CLAUDE.md");
    }

    /// REQ-AD-022: open_in_editor 는 미지원 OS 에서 Unsupported 를 반환한다.
    /// (단위 테스트 환경에서 실제 spawn 검증은 어려우므로 unsupported 분기만 정적으로 확인.)
    #[test]
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn open_in_editor_unsupported_on_other_os() {
        let res = open_in_editor(Path::new("/tmp/x"));
        assert!(res.is_err());
    }

    /// macOS / Linux 환경에서 `open` / `xdg-open` 호출이 실패해도 panic 하지 않아야 한다
    /// (binary 부재 시 io::Error 반환).
    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn open_in_editor_does_not_panic_on_supported_os() {
        // 임의 경로로 spawn 하여도 panic 없이 Result 반환.
        let _ = open_in_editor(Path::new("/dev/null"));
    }

    /// 빈 트리 상태에서 view 가 panic 없이 동작해야 한다.
    #[test]
    fn empty_tree_does_not_panic() {
        let project = tmp_dir("empty-view");
        let view = InstructionsGraphView::new(project);
        assert!(view.tree().is_empty());
        let rows = flatten_tree(view.tree());
        assert!(rows.is_empty());
    }
}
