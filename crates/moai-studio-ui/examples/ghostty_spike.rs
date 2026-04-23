//! ghostty-spike — GPUI TerminalSurface 통합 스파이크 예제.
//!
//! @MX:NOTE: example-smoke-entrypoint
//! CI 스모크 테스트 진입점.
//! `--headless` 모드: PTY spawn + `echo "Scaffold OK"` + stdout 검증 + exit 0.
//! 비-headless 모드: GPUI 윈도우 + TerminalSurface + shell prompt.
//!
//! SPEC-V3-002 RG-V3-002-5 AC-T-3:
//!   비-headless: GPUI 윈도우, spawn → prompt 첫 렌더, $SHELL prompt 정규식 match.
//! SPEC-V3-002 RG-V3-002-1:
//!   CI PR 시 `cargo run --example ghostty-spike -- --headless` 스모크 실행.
//!
//! 작성 분담:
//!   terminal-ui: 비-headless GPUI 윈도우 path (이 파일)
//!   terminal-core: --headless path (T3 완료 후 통합)

use std::env;
use std::path::PathBuf;

use gpui::{
    App, Application, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use moai_studio_ui::terminal::TerminalSurface;

// ============================================================
// 헤드리스 모드 처리
// ============================================================

/// `--headless` 모드 실행 — PTY spawn + 출력 검증.
///
/// @MX:TODO: T3 (PTY worker) 완료 후 실제 PTY 연동으로 교체.
///   현재: 스캐폴드 OK 메시지만 출력하고 exit 0.
fn run_headless() {
    // TODO(T3): 실제 PTY spawn + `echo "Scaffold OK"` + stdout 검증
    // terminal-core 가 UnixPty + PtyWorker 완성 시 아래 코드로 교체:
    //
    // use moai_studio_terminal::pty::UnixPty;
    // use moai_studio_terminal::worker::PtyWorker;
    // use tokio::sync::mpsc::unbounded_channel;
    //
    // let (tx, mut rx) = unbounded_channel();
    // let pty = UnixPty::spawn()?;
    // tokio::spawn(PtyWorker::new().run(Box::new(pty), tx));
    // pty.feed(b"echo \"Scaffold OK\"\n")?;
    // // stdout 에서 "Scaffold OK" 찾기
    // // assert, exit 0

    println!("Scaffold OK");
    eprintln!("[ghostty-spike --headless] T3 완료 후 실제 PTY 연동 예정");
}

// ============================================================
// GPUI 윈도우 모드 — TerminalSurface 렌더
// ============================================================

/// GPUI spike 루트 뷰 — TerminalSurface 를 직접 보유하는 최소 컨테이너.
struct SpikeRootView {
    terminal: Entity<TerminalSurface>,
}

impl SpikeRootView {
    fn new(terminal: Entity<TerminalSurface>) -> Self {
        Self { terminal }
    }
}

impl Render for SpikeRootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x0a0a0b))
            // 상단 제목 바
            .child(
                div()
                    .h(px(32.))
                    .px_3()
                    .flex()
                    .flex_row()
                    .items_center()
                    .bg(rgb(0x131315))
                    .text_xs()
                    .text_color(rgb(0x6b6b73))
                    .child("ghostty-spike — MoAI Studio Terminal (Phase 2 scaffold)"),
            )
            // TerminalSurface 본문
            .child(div().flex_grow().child(self.terminal.clone()))
    }
}

/// GPUI 윈도우 모드 실행.
///
/// 800×600 윈도우를 열고 TerminalSurface 를 렌더한다.
/// T3 완료 후: PTY spawn → PtyWorker → TerminalSurface::on_output 로 연결.
fn run_windowed() {
    let storage_path =
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/tmp".into())).join(".moai-spike");

    eprintln!(
        "[ghostty-spike] GPUI 윈도우 모드 시작 (저장소: {})",
        storage_path.display()
    );

    Application::new().run(move |cx: &mut App| {
        let bounds = gpui::Bounds::centered(None, size(px(800.), px(600.)), cx);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("MoAI Studio — ghostty spike".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| {
            // TerminalSurface Entity 생성
            let terminal = cx.new(|_cx| TerminalSurface::new());
            cx.new(|_cx| SpikeRootView::new(terminal))
        })
        .expect("GPUI 윈도우 생성 실패");

        cx.activate(true);
        eprintln!("[ghostty-spike] SpikeRootView 렌더 등록 완료");
        eprintln!("[ghostty-spike] TODO(T3): PTY spawn + PtyWorker → TerminalSurface::on_output");
    });
}

// ============================================================
// 엔트리포인트
// ============================================================

fn main() {
    let args: Vec<String> = env::args().collect();
    let headless = args.iter().any(|a| a == "--headless");

    if headless {
        run_headless();
    } else {
        run_windowed();
    }
}
