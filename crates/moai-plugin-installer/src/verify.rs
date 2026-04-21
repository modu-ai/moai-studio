//! 플러그인 무결성 검증 (T-018)
//!
//! 설치 디렉토리의 매니페스트 파일들이 올바르게 구성되었는지 검증한다.
//! - `.claude-plugin/plugin.json`: 필수 필드 (name, version, claude_min_version)
//! - `hooks/hooks.json`: E5 wrapper 포맷 `{"hooks": {...}}`
//! - `mcp-config.json`: JSON 파싱 가능

// @MX:ANCHOR: verify_plugin 공개 API — 설치 후 무결성 검증 진입점
// @MX:REASON: installer/FFI/CLI에서 호출되는 고-fan-in 검증 경로

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

/// 무결성 검증 에러
#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("필수 파일 누락: {0}")]
    MissingFile(PathBuf),

    #[error("JSON 파싱 실패 ({path}): {source}")]
    InvalidJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("plugin.json 필수 필드 누락: {0}")]
    MissingField(&'static str),

    #[error("hooks.json E5 wrapper 포맷 위반: 최상위에 'hooks' 객체가 필요합니다")]
    InvalidHooksWrapper,

    #[error("I/O 에러 ({path}): {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Deserialize)]
struct RequiredManifest {
    name: Option<String>,
    version: Option<String>,
    claude_min_version: Option<String>,
}

/// 설치된 플러그인 디렉토리의 무결성을 검증한다.
pub fn verify_plugin(install_dir: &Path) -> Result<(), VerifyError> {
    verify_manifest(&install_dir.join(".claude-plugin/plugin.json"))?;
    verify_hooks(&install_dir.join("hooks/hooks.json"))?;
    verify_mcp_config(&install_dir.join("mcp-config.json"))?;
    Ok(())
}

fn read_json(path: &Path) -> Result<Value, VerifyError> {
    if !path.exists() {
        return Err(VerifyError::MissingFile(path.to_path_buf()));
    }
    let content = fs::read_to_string(path).map_err(|e| VerifyError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::from_str(&content).map_err(|e| VerifyError::InvalidJson {
        path: path.to_path_buf(),
        source: e,
    })
}

fn verify_manifest(path: &Path) -> Result<(), VerifyError> {
    if !path.exists() {
        return Err(VerifyError::MissingFile(path.to_path_buf()));
    }
    let content = fs::read_to_string(path).map_err(|e| VerifyError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let m: RequiredManifest =
        serde_json::from_str(&content).map_err(|e| VerifyError::InvalidJson {
            path: path.to_path_buf(),
            source: e,
        })?;
    if m.name.as_deref().unwrap_or("").is_empty() {
        return Err(VerifyError::MissingField("name"));
    }
    if m.version.as_deref().unwrap_or("").is_empty() {
        return Err(VerifyError::MissingField("version"));
    }
    if m.claude_min_version.as_deref().unwrap_or("").is_empty() {
        return Err(VerifyError::MissingField("claude_min_version"));
    }
    Ok(())
}

fn verify_hooks(path: &Path) -> Result<(), VerifyError> {
    let v = read_json(path)?;
    // E5 wrapper: 최상위는 객체 + "hooks" 키가 객체여야 한다
    let obj = v.as_object().ok_or(VerifyError::InvalidHooksWrapper)?;
    let hooks = obj.get("hooks").ok_or(VerifyError::InvalidHooksWrapper)?;
    if !hooks.is_object() {
        return Err(VerifyError::InvalidHooksWrapper);
    }
    Ok(())
}

fn verify_mcp_config(path: &Path) -> Result<(), VerifyError> {
    let _ = read_json(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_valid(dir: &Path) {
        fs::create_dir_all(dir.join(".claude-plugin")).unwrap();
        fs::write(
            dir.join(".claude-plugin/plugin.json"),
            r#"{"name":"moai-studio","version":"0.1.0","claude_min_version":"1.0.0"}"#,
        )
        .unwrap();
        fs::create_dir_all(dir.join("hooks")).unwrap();
        fs::write(
            dir.join("hooks/hooks.json"),
            r#"{"hooks":{"PreToolUse":[]}}"#,
        )
        .unwrap();
        fs::write(dir.join("mcp-config.json"), r#"{"mcpServers":{}}"#).unwrap();
    }

    #[test]
    fn valid_plugin_passes() {
        let d = tempdir().unwrap();
        write_valid(d.path());
        verify_plugin(d.path()).expect("valid plugin must pass");
    }

    #[test]
    fn missing_claude_min_version_fails() {
        let d = tempdir().unwrap();
        write_valid(d.path());
        fs::write(
            d.path().join(".claude-plugin/plugin.json"),
            r#"{"name":"x","version":"0.1.0"}"#,
        )
        .unwrap();
        let err = verify_plugin(d.path()).unwrap_err();
        assert!(matches!(
            err,
            VerifyError::MissingField("claude_min_version")
        ));
    }

    #[test]
    fn hooks_array_format_rejected() {
        // E5 이전 포맷: hooks가 배열이면 거부
        let d = tempdir().unwrap();
        write_valid(d.path());
        fs::write(d.path().join("hooks/hooks.json"), r#"{"hooks":[]}"#).unwrap();
        let err = verify_plugin(d.path()).unwrap_err();
        assert!(matches!(err, VerifyError::InvalidHooksWrapper));
    }

    #[test]
    fn hooks_missing_wrapper_rejected() {
        let d = tempdir().unwrap();
        write_valid(d.path());
        fs::write(d.path().join("hooks/hooks.json"), r#"{"PreToolUse":[]}"#).unwrap();
        let err = verify_plugin(d.path()).unwrap_err();
        assert!(matches!(err, VerifyError::InvalidHooksWrapper));
    }

    #[test]
    fn invalid_json_rejected() {
        let d = tempdir().unwrap();
        write_valid(d.path());
        fs::write(d.path().join("mcp-config.json"), "not json").unwrap();
        let err = verify_plugin(d.path()).unwrap_err();
        assert!(matches!(err, VerifyError::InvalidJson { .. }));
    }

    #[test]
    fn missing_file_rejected() {
        let d = tempdir().unwrap();
        write_valid(d.path());
        fs::remove_file(d.path().join("mcp-config.json")).unwrap();
        let err = verify_plugin(d.path()).unwrap_err();
        assert!(matches!(err, VerifyError::MissingFile(_)));
    }
}
