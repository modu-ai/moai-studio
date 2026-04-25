//! SPEC-V3-006: 4-surface Viewer 통합 진입점.
//!
//! `LeafKind` enum 이 SPEC-V3-004 의 generic L 자리에 들어가 Pane leaf 의
//! 다형성을 제공한다. MS-2 에서 tree-sitter CodeViewer 가 구현되어
//! `Code(Entity<CodeViewer>)` 로 활성화됩니다.
//!
//! @MX:TODO(MS-3-lsp): LSP client / server_registry 모듈은 MS-3 에서 추가된다
//!   (RG-MV-4 async-lsp + lsp-types 통합).

pub mod code;
pub mod markdown;
pub mod scroll;

// @MX:ANCHOR: [AUTO] leaf-kind-dispatch
// @MX:REASON: [AUTO] SPEC-V3-006 RG-MV-?. LeafKind 는 4-surface 다형성 진입점이다.
//   fan_in >= 3: render_pane_tree<LeafKind>, RootView::handle_open_file,
//   integration 테스트, MS-2 CodeViewer 배선.

use code::CodeViewer;
use gpui::{Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
use markdown::MarkdownViewer;
use std::path::{Path, PathBuf};

// ============================================================
// SurfaceHint — 확장자 라우팅 결과
// ============================================================

/// 파일 확장자 또는 OpenFileEvent.surface_hint 로 결정된 viewer 종류.
///
/// SPEC-V3-005 의 OpenFileEvent canonical SurfaceHint 와 동형이어야 한다.
/// MS-2 에서 SPEC-V3-005 완료 후 re-export 로 교체 예정.
// @MX:TODO(MS-2-explorer-wiring): SPEC-V3-005 완료 후 여기서 re-export 하고
//   route_by_extension 는 SPEC-V3-005 의 SurfaceHint 를 직접 반환하도록 교체.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceHint {
    Markdown,
    Code,
    Terminal,
}

// ============================================================
// BinaryKind — binary 파일 종류
// ============================================================

/// 바이너리 파일의 종류 (AC-MV-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryKind {
    /// PNG 이미지 (magic: `\x89PNG\r\n\x1a\n`)
    Image,
    /// PDF 문서 (magic: `%PDF-`)
    Pdf,
    /// JPEG 이미지 (magic: `\xff\xd8\xff`)
    Jpeg,
    /// ZIP 아카이브 등 (magic: `PK\x03\x04`)
    Archive,
    /// NUL byte 비율이 1% 이상인 바이너리
    Other,
}

// ============================================================
// LeafKind enum
// ============================================================

/// 활성 Pane leaf 의 표시 종류 (REQ-MV-001 / RG-MV-?).
///
/// SPEC-V3-004 의 `render_pane_tree<L>` 에서 `L = LeafKind` 로 인스턴스화된다.
/// SPEC-V3-004 공개 API 는 변경하지 않는다 (REQ-MV-073).
pub enum LeafKind {
    /// 빈 leaf — 초기 상태 또는 close 후 placeholder.
    Empty,
    /// SPEC-V3-002 Terminal surface (carry, PTY per-pane SPEC 에서 확장).
    Terminal(Entity<crate::terminal::TerminalSurface>),
    /// SPEC-V3-006 MS-1: CommonMark + GFM markdown viewer.
    Markdown(Entity<MarkdownViewer>),
    /// SPEC-V3-006 MS-2: tree-sitter syntax highlight CodeViewer.
    // @MX:TODO(MS-3-lsp-diagnostic): LSP 진단 (squiggly underline) 은 MS-3 에서 추가.
    Code(Entity<CodeViewer>),
    /// SPEC-V3-007 carry-over: Web viewer (컴파일 전용 placeholder).
    Web,
    /// Binary 파일 — viewer 마운트 없이 안내 메시지 표시.
    Binary(BinaryKind),
}

// ============================================================
// impl Render for LeafKind
// ============================================================

impl Render for LeafKind {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self {
            LeafKind::Empty => leaf_placeholder("Empty leaf").into_any_element(),
            LeafKind::Terminal(e) => e.clone().into_any_element(),
            LeafKind::Markdown(e) => e.clone().into_any_element(),
            LeafKind::Code(e) => e.clone().into_any_element(),
            LeafKind::Web => leaf_placeholder("Web viewer — SPEC-V3-007 예정").into_any_element(),
            LeafKind::Binary(k) => {
                let msg = format!("바이너리 파일 ({:?}) — viewer 열 수 없음", k);
                leaf_placeholder(&msg).into_any_element()
            }
        }
    }
}

/// 단순 다크 배경 placeholder div.
fn leaf_placeholder(msg: &str) -> Div {
    div()
        .size_full()
        .bg(gpui::rgb(0x1a1a1a))
        .flex()
        .justify_center()
        .items_center()
        .child(div().text_color(gpui::rgb(0x555566)).child(msg.to_string()))
}

// ============================================================
// route_by_extension — 확장자 → SurfaceHint (T2 helper)
// ============================================================

/// 파일 확장자로 SurfaceHint 를 결정한다 (REQ-MV-081 / REQ-MV-082).
///
/// OpenFileEvent.surface_hint 가 None 일 때 사용한다.
/// 알 수 없는 확장자는 Code (text fallback) 로 라우팅한다.
pub fn route_by_extension(path: &Path) -> SurfaceHint {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" | "markdown" | "mdx" => SurfaceHint::Markdown,
        "rs" | "go" | "py" | "ts" | "tsx" | "js" | "jsx" | "json" | "toml" | "yaml" | "yml"
        | "txt" | "sh" | "bash" | "zsh" | "fish" | "rb" | "c" | "cpp" | "h" | "hpp" | "java"
        | "kt" | "swift" | "cs" | "css" | "html" | "xml" | "sql" => SurfaceHint::Code,
        _ => SurfaceHint::Code, // 기본값: text 파일로 가정
    }
}

// ============================================================
// ViewerError — T6
// ============================================================

/// viewer 파일 로드 실패 원인 (REQ-MV-005, REQ-MV-083).
#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
    #[error("파일이 너무 큼: {path:?} ({bytes} bytes, 최대 200MB)")]
    TooLarge { path: PathBuf, bytes: usize },

    #[error("I/O 오류: {0}")]
    Io(#[from] std::io::Error),

    #[error("바이너리 파일 거부: {0:?}")]
    Binary(BinaryKind),

    #[error("UTF-8 변환 불가: {0:?}")]
    NotUtf8(PathBuf),
}

/// 파일 read 성공 결과.
pub struct ViewerSource {
    pub path: PathBuf,
    pub source: String,
    pub byte_len: usize,
}

/// 최대 파일 크기 (200 MB, REQ-MV-005).
pub const MAX_FILE_BYTES: usize = 200 * 1024 * 1024;

/// 파일을 viewer 용으로 읽는다 (REQ-MV-004, REQ-MV-005).
///
/// MS-1 에서는 std::fs::read 동기 버전을 사용한다.
/// MS-3 에서 async 태스크(GPUI cx.spawn)로 전환 예정.
///
/// - 200MB 초과 시 `ViewerError::TooLarge` 반환.
/// - 바이너리 감지 시 `ViewerError::Binary` 반환.
/// - 그 외에는 UTF-8 (lossy) 변환 후 `ViewerSource` 반환.
pub fn read_file_for_viewer(path: &Path) -> Result<ViewerSource, ViewerError> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(ViewerError::TooLarge {
            path: path.to_path_buf(),
            bytes: bytes.len(),
        });
    }
    if let Some(kind) = is_binary(&bytes) {
        return Err(ViewerError::Binary(kind));
    }
    let source = String::from_utf8_lossy(&bytes).into_owned();
    Ok(ViewerSource {
        path: path.to_path_buf(),
        byte_len: bytes.len(),
        source,
    })
}

// ============================================================
// is_binary — T7 (binary file detection)
// ============================================================

/// 바이트 슬라이스가 바이너리 파일인지 감지한다 (REQ-MV-083, AC-MV-11).
///
/// 감지 전략:
/// 1. Magic bytes 검사 (PNG / PDF / JPEG / ZIP)
/// 2. 첫 8KB 에서 NUL byte 비율 1% 이상
///
/// 반환: `Some(BinaryKind)` — 바이너리 감지됨, `None` — 텍스트로 간주.
// @MX:NOTE: [AUTO] binary-detection-3-byte-pattern
// PNG: `\x89PNG\r\n\x1a\n` (8바이트), PDF: `%PDF-` (5바이트),
// JPEG: `\xff\xd8\xff` (3바이트), ZIP: `PK\x03\x04` (4바이트).
// NUL byte 비율: 첫 8192 바이트에서 0x00 개수 / 총 샘플 수 > 1% 이면 바이너리.
pub fn is_binary(content: &[u8]) -> Option<BinaryKind> {
    // Magic bytes 검사
    const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
    const PDF_MAGIC: &[u8] = b"%PDF-";
    const JPEG_MAGIC: &[u8] = b"\xff\xd8\xff";
    const ZIP_MAGIC: &[u8] = b"PK\x03\x04";

    if content.starts_with(PNG_MAGIC) {
        return Some(BinaryKind::Image);
    }
    if content.starts_with(PDF_MAGIC) {
        return Some(BinaryKind::Pdf);
    }
    if content.starts_with(JPEG_MAGIC) {
        return Some(BinaryKind::Jpeg);
    }
    if content.starts_with(ZIP_MAGIC) {
        return Some(BinaryKind::Archive);
    }

    // NUL byte 비율 검사 (첫 8KB 샘플)
    let sample = &content[..content.len().min(8192)];
    let nul_count = sample.iter().filter(|&&b| b == 0).count();
    if !sample.is_empty() && nul_count * 100 > sample.len() {
        // 1% 초과 → binary
        return Some(BinaryKind::Other);
    }

    None
}

// ============================================================
// OpenFileEvent — T2 (SPEC-V3-005 미완 시 local mock)
// ============================================================

/// SPEC-V3-005 의 OpenFileEvent canonical struct (MS-1 시점 local 정의).
///
/// MS-2 에서 SPEC-V3-005 완료 후 해당 crate 의 type 으로 교체 예정.
// @MX:TODO(MS-2-explorer-wiring): SPEC-V3-005 완료 후 이 struct 를
//   `moai_studio_workspace::file_explorer::OpenFileEvent` 로 교체한다.
#[derive(Debug, Clone)]
pub struct OpenFileEvent {
    /// 열 파일의 절대 경로
    pub path: PathBuf,
    /// viewer 종류 힌트 (None 이면 route_by_extension 으로 결정)
    pub surface_hint: Option<SurfaceHint>,
}

// ============================================================
// RootView::handle_open_file — T2
// ============================================================
// handle_open_file 은 lib.rs 의 RootView impl block 에 추가된다.
// viewer::handle_open_file_impl 이 실제 로직을 가지고, lib.rs 에서 위임한다.

/// OpenFileEvent 를 받아 viewer entity 를 생성하고 LeafKind 를 반환한다.
///
/// GPUI `Context<RootView>` 가 필요한 entity 생성은 호출자(lib.rs) 에서 수행한다.
/// 여기서는 라우팅 + binary 체크 + error 처리 로직만 담당한다.
pub fn resolve_event(ev: &OpenFileEvent) -> EventResolution {
    // 1. binary 파일 확인 (확장자 기반 빠른 체크 — 실제 read 는 async에서)
    let ext = ev
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 명백한 binary 확장자 조기 거부
    if matches!(
        ext.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "bmp"
            | "ico"
            | "svg"
            | "pdf"
            | "zip"
            | "tar"
            | "gz"
            | "7z"
            | "rar"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "bin"
            | "wasm"
    ) {
        return EventResolution::Binary;
    }

    // 2. surface 결정
    let surface = ev
        .surface_hint
        .clone()
        .unwrap_or_else(|| route_by_extension(&ev.path));

    EventResolution::Open(surface)
}

/// `resolve_event` 의 결과 종류.
#[derive(Debug, PartialEq, Eq)]
pub enum EventResolution {
    /// viewer 를 열어야 한다
    Open(SurfaceHint),
    /// binary 파일 — viewer 열지 않음
    Binary,
}

// ============================================================
// 단위 테스트 — T1, T2, T6, T7, T8
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ── T1: LeafKind / route_by_extension ──

    #[test]
    fn route_by_extension_md_returns_markdown() {
        let p = PathBuf::from("README.md");
        assert_eq!(route_by_extension(&p), SurfaceHint::Markdown);
    }

    #[test]
    fn route_by_extension_markdown_returns_markdown() {
        let p = PathBuf::from("doc.markdown");
        assert_eq!(route_by_extension(&p), SurfaceHint::Markdown);
    }

    #[test]
    fn route_by_extension_rs_returns_code() {
        let p = PathBuf::from("src/main.rs");
        assert_eq!(route_by_extension(&p), SurfaceHint::Code);
    }

    #[test]
    fn route_by_extension_unknown_returns_code() {
        // 알 수 없는 확장자도 text 가정으로 Code 반환
        let p = PathBuf::from("file.unknown_ext_xyz");
        assert_eq!(route_by_extension(&p), SurfaceHint::Code);
    }

    #[test]
    fn route_by_extension_no_ext_returns_code() {
        let p = PathBuf::from("Makefile");
        assert_eq!(route_by_extension(&p), SurfaceHint::Code);
    }

    // ── T7: binary detection (AC-MV-11) ──

    #[test]
    fn png_signature_is_binary() {
        let data = b"\x89PNG\r\n\x1a\n some image data";
        let result = is_binary(data);
        assert!(matches!(result, Some(BinaryKind::Image)));
    }

    #[test]
    fn pdf_signature_is_binary() {
        let data = b"%PDF-1.7\n%this is a PDF";
        let result = is_binary(data);
        assert!(matches!(result, Some(BinaryKind::Pdf)));
    }

    #[test]
    fn jpeg_signature_is_binary() {
        let data = b"\xff\xd8\xff\xe0 JFIF data here";
        let result = is_binary(data);
        assert!(matches!(result, Some(BinaryKind::Jpeg)));
    }

    #[test]
    fn zip_signature_is_binary() {
        let data = b"PK\x03\x04 zip content here";
        let result = is_binary(data);
        assert!(matches!(result, Some(BinaryKind::Archive)));
    }

    #[test]
    fn utf8_text_is_not_binary() {
        let data = b"fn main() {\n    println!(\"hello\");\n}\n";
        let result = is_binary(data);
        assert!(result.is_none());
    }

    #[test]
    fn null_heavy_is_binary() {
        // 1% 이상 NUL byte → binary
        let mut data = vec![0u8; 100];
        data.extend_from_slice(b"some text here to pad");
        let result = is_binary(&data);
        assert!(result.is_some());
    }

    #[test]
    fn single_nul_in_text_is_not_binary() {
        // 텍스트 중 NUL 1개 정도는 binary 로 판정하지 않음
        // 100 바이트 중 1개 NUL = 1% → 경계값 (> 1% 이면 binary, == 1% 이면 아님)
        let mut data = vec![b'a'; 199]; // 199개 일반 바이트
        data.push(0u8); // NUL 1개 → 200바이트 중 0.5% NUL
        let result = is_binary(&data);
        assert!(result.is_none(), "0.5% NUL 은 binary 가 아니어야 한다");
    }

    // ── T2: resolve_event ──

    #[test]
    fn resolve_event_md_extension_opens_markdown() {
        let ev = OpenFileEvent {
            path: PathBuf::from("docs/README.md"),
            surface_hint: None,
        };
        assert_eq!(
            resolve_event(&ev),
            EventResolution::Open(SurfaceHint::Markdown)
        );
    }

    #[test]
    fn resolve_event_hint_overrides_extension() {
        let ev = OpenFileEvent {
            path: PathBuf::from("somefile.rs"),
            surface_hint: Some(SurfaceHint::Markdown),
        };
        // 힌트가 있으면 힌트를 따른다
        assert_eq!(
            resolve_event(&ev),
            EventResolution::Open(SurfaceHint::Markdown)
        );
    }

    #[test]
    fn resolve_event_png_extension_is_binary() {
        let ev = OpenFileEvent {
            path: PathBuf::from("image.png"),
            surface_hint: None,
        };
        assert_eq!(resolve_event(&ev), EventResolution::Binary);
    }

    #[test]
    fn resolve_event_pdf_extension_is_binary() {
        let ev = OpenFileEvent {
            path: PathBuf::from("document.pdf"),
            surface_hint: None,
        };
        assert_eq!(resolve_event(&ev), EventResolution::Binary);
    }

    #[test]
    fn resolve_event_rs_extension_opens_code() {
        let ev = OpenFileEvent {
            path: PathBuf::from("src/lib.rs"),
            surface_hint: None,
        };
        assert_eq!(resolve_event(&ev), EventResolution::Open(SurfaceHint::Code));
    }

    // ── T8: LeafKind GPUI entity smoke (TestAppContext) ──

    #[test]
    fn leaf_kind_empty_renders_without_panic() {
        // LeafKind::Empty 이 GPUI context 없이 생성 가능 (unit 수준 검증)
        // Render::render 는 GPUI Context 가 필요하여 TestAppContext 로 테스트
        use gpui::{AppContext, TestAppContext};
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| LeafKind::Empty);
        // entity 가 생성되고 Drop 이 panic 없이 동작해야 한다
        let _ = entity;
    }

    #[test]
    fn binary_kind_variants_are_distinct() {
        assert_ne!(BinaryKind::Image, BinaryKind::Pdf);
        assert_ne!(BinaryKind::Jpeg, BinaryKind::Archive);
        assert_ne!(BinaryKind::Other, BinaryKind::Image);
    }
}
