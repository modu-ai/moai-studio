//! InstructionNode 도메인 + 6-layer scanner (RG-AD-4, AC-AD-7/8)
//!
//! SPEC-V3-010 REQ-AD-019: SessionStart 진입 시 instruction stack 을 6 layer 로 빌드.
//! SPEC-V3-010 REQ-AD-020: PreCompact / PrePromptInject 시 tree rebuild.
//! SPEC-V3-010 REQ-AD-021: indent 기반 layered tree 렌더는 UI 층 책임.
//! SPEC-V3-010 REQ-AD-023: instruction 본문 편집 금지 (read-only) — UI 가 enforce.
//!
//! 6 layer 우선순위 (낮을수록 더 높은 우선순위 = 더 강한 제약):
//!   1. ManagedPolicy   — 사용자 정책 (system-level)
//!   2. Project         — `<project>/CLAUDE.md`
//!   3. Rules           — `<project>/.claude/rules/**`
//!   4. User            — `~/.claude/CLAUDE.md`
//!   5. Local           — `<project>/CLAUDE.local.md`
//!   6. Memory + Skill  — `~/.claude/projects/<hash>/memory/` + `<project>/.claude/skills/**`

// @MX:ANCHOR: [AUTO] instruction-scanner-domain
// @MX:REASON: [AUTO] instruction tree 단일 진실 원천. fan_in >= 3:
//   InstructionsGraphView, AgentDashboardView, hook event 핸들러.
//   SPEC: SPEC-V3-010 RG-AD-4, REQ-AD-019/020/021

use std::path::{Path, PathBuf};

/// instruction 의 6 layer 분류 (REQ-AD-019).
///
/// priority 는 낮을수록 우선순위가 높다 (managed policy 가 가장 강한 제약).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionKind {
    /// priority 1 — 사용자 system-level managed policy
    ManagedPolicy,
    /// priority 2 — 프로젝트 CLAUDE.md
    Project,
    /// priority 3 — 프로젝트 .claude/rules/**
    Rules,
    /// priority 4 — 사용자 ~/.claude/CLAUDE.md
    User,
    /// priority 5 — 프로젝트 CLAUDE.local.md
    Local,
    /// priority 6 — 메모리 + 스킬 (auto-load)
    Memory,
}

impl InstructionKind {
    /// 1~6 priority 정수 매핑.
    pub fn priority(self) -> u8 {
        match self {
            Self::ManagedPolicy => 1,
            Self::Project => 2,
            Self::Rules => 3,
            Self::User => 4,
            Self::Local => 5,
            Self::Memory => 6,
        }
    }

    /// UI 라벨 (영문 짧은 표시).
    pub fn label(self) -> &'static str {
        match self {
            Self::ManagedPolicy => "managed-policy",
            Self::Project => "project",
            Self::Rules => "rules",
            Self::User => "user",
            Self::Local => "local",
            Self::Memory => "memory",
        }
    }
}

/// instruction 트리의 한 노드 (REQ-AD-019/021).
///
/// children 은 같은 layer 안의 하위 파일 목록을 담는다.
/// 예: Rules 노드는 `.claude/rules/**` 파일들을 children 으로 가진다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionNode {
    /// layer 분류
    pub kind: InstructionKind,
    /// 원본 파일 경로 (REQ-AD-022: 클릭 시 OS editor 로 open)
    pub source_path: PathBuf,
    /// 우선순위 (1~6) — kind.priority() 와 동일하지만 정렬을 위해 캐싱
    pub priority: u8,
    /// 하위 노드 (디렉터리 layer 의 경우)
    pub children: Vec<InstructionNode>,
}

impl InstructionNode {
    /// 단일 파일 노드를 생성한다.
    pub fn leaf(kind: InstructionKind, source_path: PathBuf) -> Self {
        Self {
            kind,
            source_path,
            priority: kind.priority(),
            children: Vec::new(),
        }
    }

    /// children 을 가진 grouping 노드를 생성한다.
    pub fn group(kind: InstructionKind, source_path: PathBuf, children: Vec<Self>) -> Self {
        Self {
            kind,
            source_path,
            priority: kind.priority(),
            children,
        }
    }
}

/// 스캔 진입점 경로 묶음 — 의존성 주입으로 테스트 용이.
#[derive(Debug, Clone)]
pub struct ScanPaths {
    /// 프로젝트 루트 (CLAUDE.md, CLAUDE.local.md, .claude/, .moai/ 검색 기준)
    pub project_root: PathBuf,
    /// 사용자 홈 (~/.claude/CLAUDE.md, managed policy 디렉터리)
    pub user_home: Option<PathBuf>,
    /// managed policy 디렉터리 (있을 경우 직접 지정 — 테스트용)
    pub managed_policy_dir: Option<PathBuf>,
}

impl ScanPaths {
    /// 표준 환경에서 기본 경로를 채운다 (project_root + $HOME).
    pub fn from_project_root(project_root: PathBuf) -> Self {
        let user_home = std::env::var("HOME").ok().map(PathBuf::from);
        Self {
            project_root,
            user_home,
            managed_policy_dir: None,
        }
    }
}

/// 6-layer scanner — hook event 트리거 시 rebuild (REQ-AD-019/020).
///
/// 내부적으로 rebuild 횟수를 카운트하여 테스트 용이.
// @MX:ANCHOR: [AUTO] instruction-scanner-impl
// @MX:REASON: [AUTO] hook 트리거 rebuild 의 단일 진입점. fan_in >= 3:
//   AgentDashboardView, InstructionsGraphView, hook event 처리 코드.
//   SPEC: SPEC-V3-010 REQ-AD-019/020
#[derive(Debug, Clone)]
pub struct InstructionScanner {
    paths: ScanPaths,
    rebuild_count: u64,
    last_tree: Vec<InstructionNode>,
}

impl InstructionScanner {
    /// 새 scanner 를 생성한다 (rebuild 미수행).
    pub fn new(paths: ScanPaths) -> Self {
        Self {
            paths,
            rebuild_count: 0,
            last_tree: Vec::new(),
        }
    }

    /// 지금까지 누적된 rebuild 호출 횟수.
    pub fn rebuild_count(&self) -> u64 {
        self.rebuild_count
    }

    /// 최근 빌드 결과 (rebuild 이후 캐시).
    pub fn last_tree(&self) -> &[InstructionNode] {
        &self.last_tree
    }

    /// 6 layer 를 모두 스캔하여 tree 를 빌드한다 (REQ-AD-019).
    ///
    /// 정렬: priority 오름차순. 같은 priority 내에서는 path lexicographic.
    pub fn rebuild(&mut self) -> &[InstructionNode] {
        self.rebuild_count = self.rebuild_count.saturating_add(1);

        let mut nodes = Vec::new();

        // Layer 1: managed policy
        if let Some(dir) = self.paths.managed_policy_dir.clone().or_else(|| {
            self.paths
                .user_home
                .as_ref()
                .map(|h| h.join(".claude/managed"))
        }) && dir.is_dir()
        {
            let children = scan_markdown_files(&dir, InstructionKind::ManagedPolicy);
            if !children.is_empty() {
                nodes.push(InstructionNode::group(
                    InstructionKind::ManagedPolicy,
                    dir,
                    children,
                ));
            }
        }

        // Layer 2: project CLAUDE.md
        let project_claude = self.paths.project_root.join("CLAUDE.md");
        if project_claude.is_file() {
            nodes.push(InstructionNode::leaf(
                InstructionKind::Project,
                project_claude,
            ));
        }

        // Layer 3: project .claude/rules/**
        let rules_dir = self.paths.project_root.join(".claude/rules");
        if rules_dir.is_dir() {
            let children = scan_markdown_files(&rules_dir, InstructionKind::Rules);
            if !children.is_empty() {
                nodes.push(InstructionNode::group(
                    InstructionKind::Rules,
                    rules_dir,
                    children,
                ));
            }
        }

        // Layer 4: user ~/.claude/CLAUDE.md
        if let Some(home) = &self.paths.user_home {
            let user_claude = home.join(".claude/CLAUDE.md");
            if user_claude.is_file() {
                nodes.push(InstructionNode::leaf(InstructionKind::User, user_claude));
            }
        }

        // Layer 5: project CLAUDE.local.md
        let local_claude = self.paths.project_root.join("CLAUDE.local.md");
        if local_claude.is_file() {
            nodes.push(InstructionNode::leaf(InstructionKind::Local, local_claude));
        }

        // Layer 6: memory + skills (auto-load 영역)
        let mut memory_children = Vec::new();

        // 6a. ~/.claude/projects/*/memory/MEMORY.md (project hash 별)
        if let Some(home) = &self.paths.user_home {
            let projects_dir = home.join(".claude/projects");
            if projects_dir.is_dir()
                && let Ok(read) = std::fs::read_dir(&projects_dir)
            {
                for entry in read.flatten() {
                    let memory_md = entry.path().join("memory/MEMORY.md");
                    if memory_md.is_file() {
                        memory_children
                            .push(InstructionNode::leaf(InstructionKind::Memory, memory_md));
                    }
                }
            }
        }

        // 6b. project .claude/skills/**/SKILL.md
        let skills_dir = self.paths.project_root.join(".claude/skills");
        if skills_dir.is_dir() {
            collect_skill_files(&skills_dir, &mut memory_children);
        }

        if !memory_children.is_empty() {
            // memory layer 의 그룹 path 는 user home 의 .claude 또는 project skills 중 첫 번째
            let group_path = self
                .paths
                .user_home
                .as_ref()
                .map(|h| h.join(".claude"))
                .unwrap_or_else(|| skills_dir.clone());
            nodes.push(InstructionNode::group(
                InstructionKind::Memory,
                group_path,
                memory_children,
            ));
        }

        // priority 오름차순 정렬 (이미 삽입 순서가 priority 오름차순이지만 안전장치)
        nodes.sort_by_key(|n| n.priority);

        self.last_tree = nodes;
        &self.last_tree
    }
}

/// hook event 진입점 — REQ-AD-019/020 의 트리거 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionRebuildTrigger {
    /// SessionStart hook (REQ-AD-019)
    SessionStart,
    /// PreCompact hook (REQ-AD-020)
    PreCompact,
    /// PrePromptInject hook (REQ-AD-020)
    PrePromptInject,
}

impl InstructionRebuildTrigger {
    /// hook event_name 문자열에서 trigger 종류로 매핑한다.
    /// 트리거가 아닌 경우 None.
    pub fn from_hook_name(name: &str) -> Option<Self> {
        match name {
            "SessionStart" => Some(Self::SessionStart),
            "PreCompact" => Some(Self::PreCompact),
            "PrePromptInject" => Some(Self::PrePromptInject),
            _ => None,
        }
    }
}

// ----------------------------------------------------------------
// 내부 헬퍼
// ----------------------------------------------------------------

/// 디렉터리에서 .md 파일을 재귀 스캔하여 leaf 노드 목록을 만든다.
fn scan_markdown_files(dir: &Path, kind: InstructionKind) -> Vec<InstructionNode> {
    let mut out = Vec::new();
    walk_md(dir, kind, &mut out);
    // 경로 정렬 (lexicographic) — 일관된 표시 순서
    out.sort_by(|a, b| a.source_path.cmp(&b.source_path));
    out
}

fn walk_md(dir: &Path, kind: InstructionKind, out: &mut Vec<InstructionNode>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_md(&path, kind, out);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            out.push(InstructionNode::leaf(kind, path));
        }
    }
}

/// .claude/skills/**/SKILL.md 만 수집한다.
fn collect_skill_files(skills_dir: &Path, out: &mut Vec<InstructionNode>) {
    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_skill_files(&path, out);
        } else if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case("SKILL.md"))
            .unwrap_or(false)
        {
            out.push(InstructionNode::leaf(InstructionKind::Memory, path));
        }
    }
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Instant;

    /// 테스트용 임시 디렉터리 (suffix 로 충돌 회피).
    fn tmp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("moai-spec-v3-010-{}-{}", label, nanos));
        fs::create_dir_all(&dir).expect("temp dir 생성 실패");
        dir
    }

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent dir 생성 실패");
        }
        fs::write(path, body).expect("파일 쓰기 실패");
    }

    /// AC-AD-7: 6 layer scanner 가 priority 1~6 모두 검출한다.
    #[test]
    fn scanner_finds_six_layers() {
        let project = tmp_dir("six-layers-proj");
        let home = tmp_dir("six-layers-home");
        let managed = tmp_dir("six-layers-managed");

        // Layer 1: managed policy
        write(&managed.join("policy.md"), "# managed");
        // Layer 2: project CLAUDE.md
        write(&project.join("CLAUDE.md"), "# project");
        // Layer 3: project .claude/rules
        write(&project.join(".claude/rules/r1.md"), "# rule1");
        // Layer 4: user home CLAUDE.md
        write(&home.join(".claude/CLAUDE.md"), "# user");
        // Layer 5: project CLAUDE.local.md
        write(&project.join("CLAUDE.local.md"), "# local");
        // Layer 6: skill
        write(&project.join(".claude/skills/foo/SKILL.md"), "# skill");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: Some(home),
            managed_policy_dir: Some(managed),
        });
        let tree = scanner.rebuild().to_vec();

        let kinds: Vec<_> = tree.iter().map(|n| n.kind).collect();
        assert!(kinds.contains(&InstructionKind::ManagedPolicy));
        assert!(kinds.contains(&InstructionKind::Project));
        assert!(kinds.contains(&InstructionKind::Rules));
        assert!(kinds.contains(&InstructionKind::User));
        assert!(kinds.contains(&InstructionKind::Local));
        assert!(kinds.contains(&InstructionKind::Memory));
    }

    /// AC-AD-7: priority 1~6 오름차순 정렬을 검증한다.
    #[test]
    fn nodes_sorted_by_priority() {
        let project = tmp_dir("priority-proj");
        let home = tmp_dir("priority-home");

        write(&project.join("CLAUDE.md"), "# project");
        write(&project.join("CLAUDE.local.md"), "# local");
        write(&home.join(".claude/CLAUDE.md"), "# user");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: Some(home),
            managed_policy_dir: None,
        });
        let tree = scanner.rebuild();

        // Project (2) → User (4) → Local (5) 순으로 정렬되어야 한다.
        let priorities: Vec<u8> = tree.iter().map(|n| n.priority).collect();
        let mut sorted = priorities.clone();
        sorted.sort();
        assert_eq!(priorities, sorted, "priority 오름차순 정렬 실패");
    }

    /// AC-AD-8: SessionStart trigger 가 rebuild_count 를 증가시킨다.
    #[test]
    fn session_start_triggers_rebuild() {
        let project = tmp_dir("session-start");
        write(&project.join("CLAUDE.md"), "# project");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: None,
            managed_policy_dir: None,
        });

        assert_eq!(scanner.rebuild_count(), 0);

        let trigger = InstructionRebuildTrigger::from_hook_name("SessionStart");
        assert_eq!(trigger, Some(InstructionRebuildTrigger::SessionStart));
        scanner.rebuild();
        assert_eq!(scanner.rebuild_count(), 1);

        scanner.rebuild();
        assert_eq!(scanner.rebuild_count(), 2);
    }

    /// AC-AD-8: PreCompact / PrePromptInject 도 trigger 로 매핑되어야 한다.
    #[test]
    fn pre_compact_and_pre_prompt_inject_are_triggers() {
        assert_eq!(
            InstructionRebuildTrigger::from_hook_name("PreCompact"),
            Some(InstructionRebuildTrigger::PreCompact)
        );
        assert_eq!(
            InstructionRebuildTrigger::from_hook_name("PrePromptInject"),
            Some(InstructionRebuildTrigger::PrePromptInject)
        );
        // 트리거가 아닌 hook 은 None
        assert_eq!(
            InstructionRebuildTrigger::from_hook_name("PostToolUse"),
            None
        );
    }

    /// AC-AD-7: source_path 가 클릭 가능한 절대 경로로 보존되어야 한다 (REQ-AD-022).
    #[test]
    fn source_path_is_preserved() {
        let project = tmp_dir("source-path");
        let claude = project.join("CLAUDE.md");
        write(&claude, "# project");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: None,
            managed_policy_dir: None,
        });
        let tree = scanner.rebuild();

        let project_node = tree
            .iter()
            .find(|n| n.kind == InstructionKind::Project)
            .expect("Project 노드 누락");
        assert_eq!(project_node.source_path, claude);
    }

    /// AC-AD-7 + NFR-AD-6 가벼운 검증: 작은 트리에서 rebuild 가 < 300ms.
    #[test]
    fn rebuild_completes_within_budget() {
        let project = tmp_dir("rebuild-bench");
        write(&project.join("CLAUDE.md"), "# project");
        for i in 0..10u32 {
            write(&project.join(format!(".claude/rules/r{}.md", i)), "# rule");
        }

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: None,
            managed_policy_dir: None,
        });

        let started = Instant::now();
        scanner.rebuild();
        let elapsed = started.elapsed();
        assert!(
            elapsed.as_millis() < 300,
            "rebuild 가 300ms 를 초과했다: {:?}",
            elapsed
        );
    }

    /// kind.priority() 가 1~6 매핑이 정확해야 한다.
    #[test]
    fn priority_mapping_is_one_through_six() {
        assert_eq!(InstructionKind::ManagedPolicy.priority(), 1);
        assert_eq!(InstructionKind::Project.priority(), 2);
        assert_eq!(InstructionKind::Rules.priority(), 3);
        assert_eq!(InstructionKind::User.priority(), 4);
        assert_eq!(InstructionKind::Local.priority(), 5);
        assert_eq!(InstructionKind::Memory.priority(), 6);
    }

    /// 빈 프로젝트는 빈 트리를 반환해야 한다 (panic 금지).
    #[test]
    fn empty_project_returns_empty_tree() {
        let project = tmp_dir("empty-proj");

        let mut scanner = InstructionScanner::new(ScanPaths {
            project_root: project,
            user_home: None,
            managed_policy_dir: None,
        });
        let tree = scanner.rebuild();
        assert!(tree.is_empty());
    }
}
