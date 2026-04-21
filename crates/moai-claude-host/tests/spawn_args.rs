//! T-012: workspace_config 빌더가 SPEC-M1-001 §RG-M1-5 필수 인수 세트를
//! 정확히 구성하는지 검증한다. 실제 프로세스 스폰은 수행하지 않는다.
//!
//! AC-18 기준 인자 세트:
//!   claude --bare -p "" --output-format stream-json --include-partial-messages
//!   --verbose --permission-mode acceptEdits --settings <path> --mcp-config <path>
//!   --tools "Read,Edit,Write,Bash,Glob,Grep,mcp__moai__*"

use moai_claude_host::spawn::{DEFAULT_TOOLS, workspace_config};
use std::path::PathBuf;

fn args_of(cfg: &moai_claude_host::ClaudeProcessConfig) -> Vec<String> {
    let cmd = cfg.build_command();
    cmd.as_std()
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn workspace_config_contains_all_mandatory_flags() {
    let cfg = workspace_config(
        "claude",
        "sk-test",
        PathBuf::from("/tmp/work"),
        Some(PathBuf::from("/tmp/settings.json")),
        Some(PathBuf::from("/tmp/mcp.json")),
    );
    let args = args_of(&cfg);

    // 필수 플래그
    for required in [
        "--bare",
        "-p",
        "--output-format",
        "stream-json",
        "--include-partial-messages",
        "--verbose",
        "--permission-mode",
        "acceptEdits",
        "--settings",
        "--mcp-config",
        "--tools",
    ] {
        assert!(
            args.iter().any(|a| a == required),
            "필수 플래그 누락: {required} (전체 인자: {args:?})"
        );
    }
}

#[test]
fn workspace_config_tools_csv_matches_ac18() {
    let cfg = workspace_config("claude", "sk-test", "/tmp/work", None, None);
    let args = args_of(&cfg);
    let idx = args.iter().position(|a| a == "--tools").unwrap();
    let tools_csv = &args[idx + 1];
    for t in DEFAULT_TOOLS {
        assert!(
            tools_csv.split(',').any(|s| s == *t),
            "--tools CSV에 {t} 없음: {tools_csv}"
        );
    }
    // MCP glob 포함 검증
    assert!(
        tools_csv.contains("mcp__moai__*"),
        "AC-18: MCP 와일드카드 (mcp__moai__*) 누락: {tools_csv}"
    );
}

#[test]
fn workspace_config_empty_prompt_flag() {
    // -p "" 인자 다음에 빈 문자열이 와야 한다.
    let cfg = workspace_config("claude", "sk-test", "/tmp/work", None, None);
    let args = args_of(&cfg);
    let p_idx = args.iter().position(|a| a == "-p").expect("-p 없음");
    assert_eq!(args[p_idx + 1], "", "-p 다음은 빈 문자열이어야 함");
}

#[test]
fn workspace_config_optional_paths_respected() {
    // settings/mcp-config 없을 때는 해당 플래그가 없어야 한다.
    let cfg = workspace_config("claude", "sk-test", "/tmp/work", None, None);
    let args = args_of(&cfg);
    assert!(!args.iter().any(|a| a == "--settings"));
    assert!(!args.iter().any(|a| a == "--mcp-config"));
}

#[test]
fn workspace_config_env_has_api_key_and_no_scrub() {
    let cfg = workspace_config("claude", "sk-workspace", "/tmp/work", None, None);
    let cmd = cfg.build_command();
    let envs: std::collections::HashMap<String, Option<String>> = cmd
        .as_std()
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.map(|x| x.to_string_lossy().into_owned()),
            )
        })
        .collect();
    assert_eq!(
        envs.get("ANTHROPIC_API_KEY").cloned().flatten().as_deref(),
        Some("sk-workspace")
    );
    assert_eq!(
        envs.get("CLAUDE_CODE_SUBPROCESS_ENV_SCRUB")
            .cloned()
            .flatten()
            .as_deref(),
        Some("0")
    );
}
