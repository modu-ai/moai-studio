# SPEC-V3-007 Implementation Plan — Embedded Web Browser Surface

작성: MoAI (manager-spec, 2026-04-25)
브랜치 (산출 위치): `feature/SPEC-V3-004-render` (orchestrator branch — 본 산출물은 docs only)
브랜치 (구현 예정): `feature/SPEC-V3-007-webview` (orchestrator 가 후속 분리)
범위: SPEC-V3-007 spec.md 의 7 RG × 3 MS × 10 AC × 4 USER-DECISION 의 task 분해 + 3 Spike.
선행: SPEC-V3-004 가 RootView::tab_container 배선까지 완료. SPEC-V3-005/006 의 머지 선후로 `LeafKind::Web` variant 도입 위치 결정.

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|------|----------------------|------|----|
| **T0** | 전체 | USER-DECISION 4 게이트 동시 호출 + Spike 0 (wry+GPUI handshake) + Spike 1 (Linux webkit2gtk-4.1 빌드) + Spike 2 (bridge round-trip) | (Spike 보고서 inline → progress.md), `Cargo.toml` 가능 시 | — | (게이트) — |
| **T1** | MS-1 | `web/backend.rs` — `WebViewBackend` trait + `WebViewState` enum + `WebViewError` 정의 | `crates/moai-studio-ui/src/web/backend.rs` (신규), `crates/moai-studio-ui/src/web/mod.rs` (신규) | T0 | AC-WB-1 |
| **T2** | MS-1 | `web/wry_backend.rs` — `WryBackend` impl + GPUI native handle attach (Spike 0 결과 반영) | `crates/moai-studio-ui/src/web/wry_backend.rs` (신규), `crates/moai-studio-ui/Cargo.toml` (`wry = "0.50"`) | T1 | AC-WB-1 |
| **T3** | MS-1 | `web/surface.rs` — `WebViewSurface` struct + impl Render placeholder + LeafKind::Web variant 추가 (필요 시) | `crates/moai-studio-ui/src/web/surface.rs` (신규), `crates/moai-studio-ui/src/tabs/container.rs` 또는 `leaf.rs` (V3-006 미머지 시 enum variant 추가) | T1, T2 | AC-WB-1, AC-WB-10 |
| **T4** | MS-2 | `web/history.rs` — `NavigationHistory` 자료구조 + 단위 테스트 | `crates/moai-studio-ui/src/web/history.rs` (신규) | T1 | AC-WB-2 |
| **T5** | MS-2 | URL bar 입력 핸들러 + URL sanitize + scheme allowlist + status bar | `crates/moai-studio-ui/src/web/surface.rs` (확장) | T2, T4 | AC-WB-2, AC-WB-3 |
| **T6** | MS-2 | DevTools 토글 단축키 + cfg(debug_assertions) 분기 (USER-DECISION-B 결과 반영) | `crates/moai-studio-ui/src/web/surface.rs` (단축키 핸들러 확장), `crates/moai-studio-ui/src/lib.rs` (RootView 의 key_down forwarding) | T2 | AC-WB-4 |
| **T7** | MS-2 | Sandbox profile 디렉터리 + navigation_handler scheme 검증 + mixed content reject | `crates/moai-studio-ui/src/web/wry_backend.rs` (확장: with_data_directory + with_navigation_handler), `crates/moai-studio-ui/src/web/config.rs` (신규) | T2 | AC-WB-7 |
| **T8** | MS-2 | `.moai/config/sections/web.yaml` 디폴트 + `WebConfig` 로드 helper | `.moai/config/sections/web.yaml` (신규), `crates/moai-studio-ui/src/web/config.rs` (확장) | T7 | AC-WB-7 |
| **T9** | MS-3 | `web/bridge.rs` — `BridgeMessage` + `BridgeRouter` + payload 크기 검증 + trusted_domains 매칭 | `crates/moai-studio-ui/src/web/bridge.rs` (신규) | T2, T8 | AC-WB-8, AC-WB-9 |
| **T10** | MS-3 | wry IPC handler 등록 + evaluate_script 응답 round-trip + window.studio shim 주입 | `crates/moai-studio-ui/src/web/wry_backend.rs` (확장: with_ipc_handler + on_page_load shim 주입), `crates/moai-studio-ui/src/web/surface.rs` (bridge 라우터 보유) | T9 | AC-WB-8, AC-WB-9 |
| **T11** | MS-3 | `web/url_detector.rs` — regex match + `UrlDetectionDebouncer` (5s window + 30분 silence) + 단위 테스트 | `crates/moai-studio-ui/src/web/url_detector.rs` (신규) | T1 | AC-WB-5 |
| **T12** | MS-3 | `terminal/mod.rs` 의 stdout observer hook 추가 (공개 API 보존) + RootView 의 url_detector 통합 + toast element 렌더 | `crates/moai-studio-ui/src/terminal/mod.rs` (확장: `add_stdout_observer`), `crates/moai-studio-ui/src/lib.rs` (RootView 필드 + toast_overlay) | T11 | AC-WB-5, AC-WB-6 |
| **T13** | MS-3 | Toast 클릭 → 새 탭 + LeafKind::Web mount → navigate dispatch | `crates/moai-studio-ui/src/lib.rs` (toast click handler), `crates/moai-studio-ui/src/web/surface.rs` (open_new_tab 헬퍼) | T3, T12 | AC-WB-6 |
| **T14** | MS-3 | Persistence schema 의 `LeafState::Web { url }` variant 추가 (forward-compat) + serde round-trip 테스트 | `crates/moai-studio-workspace/src/persistence.rs` (확장: enum variant) | T3 | AC-WB-10 |
| **T15** | MS-3 | Crash recovery — `WebViewState::Crashed` 전이 + 토스트 element + reload 핸들러 | `crates/moai-studio-ui/src/web/surface.rs` (state 핸들링), `crates/moai-studio-ui/src/web/wry_backend.rs` (crash 콜백) | T2 | (NFR-WB-7), AC-WB-1 의 panic 방지 |
| **T16** | 전체 | `tests/integration_web.rs` — RG-WB-1/2/3/4/5/6 e2e (USER-DECISION-gpui-test-support 결과에 따라 깊이 분기) | `crates/moai-studio-ui/tests/integration_web.rs` (신규) | T1~T15 | AC-WB-1~9 |
| **T17** | 전체 | regression sweep + clippy/fmt + smoke + progress.md 갱신 + commit | (git 작업) | T1~T16 | AC-WB-10 (regression gate) |

---

## 2. T0 — USER-DECISION 4 게이트 + Spike 0 / Spike 1 / Spike 2

### 2.1 호출 (AskUserQuestion 라운드 분리)

[USER-DECISION-REQUIRED: webview-backend-choice-v3-007]
[USER-DECISION-REQUIRED: linux-webkit2gtk-version-v3-007]
[USER-DECISION-REQUIRED: devtools-activation-policy-v3-007]
[USER-DECISION-REQUIRED: webview-sandbox-profile-v3-007]

AskUserQuestion 의 4 questions/round 제한 안에 4 게이트가 들어가므로 **단일 라운드**로 묶어 호출 가능.

라운드 1 — 4 게이트 동시:
- A (`webview-backend-choice`): (a)wry (권장) / (b)servo / (c)tao+자체구현
- B (`linux-webkit2gtk-version`): (a)4.1 only (권장) / (b)4.0+4.1 cfg 분기 / (c)4.1 only + release-note 차단
- C (`devtools-activation-policy`): (a)항상 (권장) / (b)debug-only / (c)config 토글
- D (`webview-sandbox-profile`): (a)per-workspace (권장) / (b)global / (c)incognito

각 옵션 description 에 비용/이점/위험 명기.

### 2.2 Spike 0 — wry + GPUI handshake (≤ 4h)

목표: GPUI Window 의 native handle 노출 + wry::WebViewBuilder::new_as_child 또는 동등 API 로 attach 가능 검증.

검증 단계:
1. `crates/moai-studio-app/examples/webview_spike.rs` 신규 — minimal app: GPUI Window 1 개 + wry::WebView attach + navigate("https://example.com").
2. macOS 실행 → 윈도우 안에 example.com 페이지 가시.
3. Linux 실행 → 동일.
4. 실패 시: GPUI 측에 native handle 노출 PR 가능성 검토 → 미가능 시 SPEC 보류 + 사용자 통지 (T0 fail-fast).

검증 결과 → progress.md 의 "Spike 0" 섹션에 (PASS/FAIL, 사용한 GPUI API, wry 버전) 기록.

### 2.3 Spike 1 — Linux webkit2gtk-4.1 CI 빌드 (≤ 1h)

목표: `.github/workflows/ci-rust.yml` 의 ubuntu-22.04 job 에 다음 단계 추가:

```yaml
- name: Install webkit2gtk dev deps
  if: runner.os == 'Linux'
  run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libsoup-3.0-dev
```

검증: `cargo build -p moai-studio-ui --features web` (또는 wry 의존성이 enable 된 상태) 가 빌드 통과.

실패 시: USER-DECISION-Linux-pin 의 (b) (cfg 분기) 로 fallback + progress.md 기록.

### 2.4 Spike 2 — bridge round-trip (≤ 2h)

목표: wry 의 `with_ipc_handler` 등록 + JS 측 `window.ipc.postMessage("ping")` → Rust 수신 + `evaluate_script("console.log('pong')")` 응답 round-trip 검증.

검증 단계:
1. webview_spike.rs 의 페이지에 `<button onclick="window.ipc.postMessage('ping')">Send</button>` 추가.
2. Rust 측 `with_ipc_handler` 콜백에서 받은 string 이 "ping" 인지 검증.
3. `evaluate_script("console.log('pong')")` 호출 후 wry::WebView::set_console_message_handler 로 'pong' 수신 검증 (또는 DevTools 가시 확인).

실패 시: bridge 설계 재검토 (T9 시작 시점 deferred).

### 2.5 Spike 결과 기록

`progress.md` (또는 inline 보고) 의 USER-DECISION 섹션 + Spike 결과 섹션에:
- (A/B/C/D) × 선택 결과
- Spike 0/1/2 PASS/FAIL + 실측 영향 + 우회 사실 여부
- T1 부터 결정 결과를 가정하고 진행

---

## 3. T1 — WebViewBackend trait + WebViewState (RG-WB-1 REQ-WB-003)

### 3.1 변경 대상

신규 `crates/moai-studio-ui/src/web/backend.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-007 RG-WB-1 REQ-WB-003. WebViewBackend trait 은
//!   wry 와 미래 backend (servo 등) 의 swap 지점. fan_in >= 2: WryBackend 구현체,
//!   WebViewSurface 의 backend 필드.
//! @MX:SPEC: SPEC-V3-007

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebViewState {
    Loading,
    Ready,
    Error,
    Crashed,
}

#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    #[error("backend init failed: {0}")]
    InitFailed(String),
    #[error("navigation rejected: {0}")]
    NavigationRejected(String),
    #[error("io: {0}")]
    Io(String),
}

pub trait WebViewBackend: Send + Sync {
    fn navigate(&self, url: &str) -> Result<(), WebViewError>;
    fn back(&self) -> Result<(), WebViewError>;
    fn forward(&self) -> Result<(), WebViewError>;
    fn reload(&self) -> Result<(), WebViewError>;
    fn evaluate_script(&self, script: &str) -> Result<(), WebViewError>;
    fn set_bounds(&self, x: f32, y: f32, width: f32, height: f32);
    fn open_devtools(&self);
    fn close_devtools(&self);
    fn is_devtools_open(&self) -> bool;
    fn current_state(&self) -> WebViewState;
}
```

신규 `crates/moai-studio-ui/src/web/mod.rs`:

```rust
//! @MX:ANCHOR: [AUTO] web-module-root
//! @MX:REASON: [AUTO] SPEC-V3-007 의 7 개 서브모듈 (backend/wry_backend/surface/history/url_detector/bridge/config) 의 단일 진입점.
//!   fan_in >= 3: lib.rs (RootView 필드), tabs::container (LeafKind::Web), integration_web.rs.
//! @MX:SPEC: SPEC-V3-007

pub mod backend;
pub mod bridge;
pub mod config;
pub mod history;
pub mod surface;
pub mod url_detector;
pub mod wry_backend;

pub use backend::{WebViewBackend, WebViewError, WebViewState};
pub use surface::WebViewSurface;
```

### 3.2 단위 테스트 (AC-WB-1 부분)

- `test_state_default_loading`: WebViewState::default() == Loading.
- `test_error_display`: 4 variants display string 검증.

---

## 4. T2 — WryBackend impl + native handle attach (Spike 0 결과 반영)

### 4.1 변경 대상

신규 `crates/moai-studio-ui/src/web/wry_backend.rs`:

```rust
//! @MX:ANCHOR: [AUTO] wry-backend-impl
//! @MX:REASON: [AUTO] SPEC-V3-007 RG-WB-1. wry 와 GPUI 의 통합 지점. native handle attach 의
//!   유일한 위치 — Spike 0 결과 반영. fan_in >= 2: WebViewSurface, integration_web.rs.
//! @MX:WARN: [AUTO] native handle 추출 / set_bounds 동기화 / crash 콜백 의 3 축에서 race 가능.
//! @MX:SPEC: SPEC-V3-007

use super::backend::{WebViewBackend, WebViewError, WebViewState};
use std::path::Path;
use std::sync::{Arc, Mutex};
use wry::{WebView, WebViewBuilder};

pub struct WryBackend {
    inner: Arc<Mutex<WebView>>,
    state: Arc<Mutex<WebViewState>>,
}

impl WryBackend {
    pub fn new(
        parent_handle: /* GPUI native handle type — Spike 0 결과 결정 */,
        initial_url: &str,
        data_dir: &Path,
        with_devtools: bool,
        navigation_handler: impl Fn(&str) -> bool + Send + Sync + 'static,
        ipc_handler: impl Fn(String) + Send + Sync + 'static,
    ) -> Result<Self, WebViewError> {
        let webview = WebViewBuilder::new()
            .with_url(initial_url)
            .with_devtools(with_devtools)
            .with_navigation_handler(move |url| navigation_handler(&url))
            .with_ipc_handler(move |msg| ipc_handler(msg.body().to_string()))
            // .with_data_directory(data_dir)  ← API 정확명 Spike 0 시점 확인
            // Spike 0 결과 따라 build_as_child(parent_handle) 또는 build()
            .build()
            .map_err(|e| WebViewError::InitFailed(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(webview)),
            state: Arc::new(Mutex::new(WebViewState::Loading)),
        })
    }
}

impl WebViewBackend for WryBackend {
    fn navigate(&self, url: &str) -> Result<(), WebViewError> { /* inner.load_url(url) */ }
    fn back(&self) -> Result<(), WebViewError> { /* evaluate_script("history.back()") */ }
    fn forward(&self) -> Result<(), WebViewError> { /* evaluate_script("history.forward()") */ }
    fn reload(&self) -> Result<(), WebViewError> { /* inner.reload() */ }
    fn evaluate_script(&self, script: &str) -> Result<(), WebViewError> { /* inner.evaluate_script(script) */ }
    fn set_bounds(&self, x: f32, y: f32, width: f32, height: f32) { /* inner.set_bounds(...) */ }
    fn open_devtools(&self) { /* inner.open_devtools() */ }
    fn close_devtools(&self) { /* inner.close_devtools() */ }
    fn is_devtools_open(&self) -> bool { /* inner.is_devtools_open() */ }
    fn current_state(&self) -> WebViewState { *self.state.lock().unwrap() }
}
```

### 4.2 Cargo.toml 변경

`crates/moai-studio-ui/Cargo.toml` `[dependencies]`:

```toml
wry = "0.50"   # Spike 0 시점 latest stable 으로 정확히 lock
```

### 4.3 단위 테스트 (AC-WB-1 부분)

- mock backend (WryBackend 가 아닌 별도 `MockBackend`) 를 통한 trait 동작 검증.
- 실제 WryBackend 의 빌드 검증은 Spike 0 + 통합 테스트 (T16) 로 위임.

---

## 5. T3 — WebViewSurface struct + impl Render placeholder + LeafKind::Web (RG-WB-1 REQ-WB-001/002)

### 5.1 변경 대상 1: 신규 `crates/moai-studio-ui/src/web/surface.rs`

```rust
//! @MX:ANCHOR: [AUTO] web-view-surface
//! @MX:REASON: [AUTO] SPEC-V3-007 RG-WB-1. WebViewSurface 는 RootView 의 webview leaf 의
//!   진입점이며 history/bridge/devtools 의 mutation 이 모두 이 Entity 로 수렴한다.
//!   fan_in >= 5: tabs::container (LeafKind::Web), url 입력 핸들러, bridge router, devtools 단축키, persistence restore.
//! @MX:SPEC: SPEC-V3-007

use super::backend::{WebViewBackend, WebViewState};
use super::bridge::BridgeRouter;
use super::history::NavigationHistory;
use gpui::{Context, IntoElement, Render, Window};
use std::sync::Arc;

pub struct WebViewSurface {
    pub url: String,
    pub history: NavigationHistory,
    pub state: WebViewState,
    pub backend: Option<Box<dyn WebViewBackend>>,
    pub bridge: Arc<BridgeRouter>,
    pub devtools_open: bool,
    pub bounds: Option<gpui::Bounds<f32>>,
}

impl WebViewSurface {
    pub fn new(initial_url: String) -> Self { /* ... */ }
    pub fn navigate(&mut self, url: &str, cx: &mut Context<Self>) { /* T5 가 추가 */ }
    pub fn back(&mut self, cx: &mut Context<Self>) { /* T5 가 추가 */ }
    pub fn forward(&mut self, cx: &mut Context<Self>) { /* T5 가 추가 */ }
    pub fn toggle_devtools(&mut self, cx: &mut Context<Self>) { /* T6 가 추가 */ }
}

impl Render for WebViewSurface {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // T3 placeholder: URL bar + 빈 회색 영역 + status bar.
        // T5 가 URL bar enter 핸들러 추가, T6 가 단축키 핸들러 추가, T15 가 Crashed 토스트 추가.
        // backend 가 None 이면 "WebView unavailable" 메시지 (REQ-WB-005).
        // ...
    }
}
```

### 5.2 변경 대상 2: `crates/moai-studio-ui/src/tabs/container.rs` 또는 SPEC-V3-006 의 `leaf.rs`

`LeafKind` enum 에 variant 추가:

```rust
pub enum LeafKind {
    Terminal(Entity<TerminalSurface>),
    Markdown(Entity<MarkdownViewer>),  // SPEC-V3-006
    Code(Entity<CodeViewer>),          // SPEC-V3-006
    Web(Entity<WebViewSurface>),       // ← SPEC-V3-007 (본 task)
    Empty,
}
```

**OD-WB5 결정**: 본 SPEC 의 머지 시점에 SPEC-V3-006 가 먼저 머지되어 있으면 그쪽 enum 에 variant 추가만, 아니면 본 SPEC 이 enum 자체를 도입.

### 5.3 변경 대상 3: `crates/moai-studio-ui/src/lib.rs`

`pub mod web;` 추가 (16~19 line 근처).

### 5.4 단위 테스트 (AC-WB-1)

- `test_web_view_surface_new_default_state`: url == initial_url, history.entries.len() == 0, state == Loading, backend == None (또는 Some 이지만 mock).
- `test_render_does_not_panic_when_backend_none`: REQ-WB-005 — backend = None 상태에서 render 호출 시 panic 없음, "WebView unavailable" 텍스트 element 검증.
- (USER-DECISION-gpui-test-support=(a) 시) GPUI integration: `cx.new(|cx| WebViewSurface::new(...))` 로 Entity 생성 가능.

---

## 6. T4 — NavigationHistory (RG-WB-2 REQ-WB-010, REQ-WB-014)

### 6.1 변경 대상

신규 `crates/moai-studio-ui/src/web/history.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-007 RG-WB-2. WebViewSurface 의 history 자료구조.
//!   wry 의 OS-level history 와 별개로 Studio 가 자체 추적 — back/forward 의 일관성 보장.
//! @MX:SPEC: SPEC-V3-007

use std::time::SystemTime;

pub const MAX_HISTORY_ENTRIES: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub url: String,
    pub title: Option<String>,
    pub visited_at: SystemTime,
}

#[derive(Debug, Clone, Default)]
pub struct NavigationHistory {
    entries: Vec<HistoryEntry>,
    cursor: usize,
}

impl NavigationHistory {
    pub fn new() -> Self { /* ... */ }
    pub fn navigate(&mut self, url: &str) {
        // cursor 이후 entries 절단 + 새 entry 추가 + cursor = entries.len() - 1
        // entries.len() > MAX_HISTORY_ENTRIES 시 oldest drop + cursor -= 1
    }
    pub fn back(&mut self) -> Option<&HistoryEntry> {
        if self.cursor > 0 { self.cursor -= 1; Some(&self.entries[self.cursor]) } else { None }
    }
    pub fn forward(&mut self) -> Option<&HistoryEntry> {
        if self.cursor + 1 < self.entries.len() { self.cursor += 1; Some(&self.entries[self.cursor]) } else { None }
    }
    pub fn current(&self) -> Option<&HistoryEntry> { self.entries.get(self.cursor) }
    pub fn set_title(&mut self, title: String) {
        if let Some(entry) = self.entries.get_mut(self.cursor) { entry.title = Some(title); }
    }
    pub fn clear(&mut self) { self.entries.clear(); self.cursor = 0; }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn cursor(&self) -> usize { self.cursor }
}
```

### 6.2 단위 테스트 (AC-WB-2 부분)

- `test_navigate_adds_entry_and_advances_cursor`: empty 에서 navigate → entries.len()=1, cursor=0.
- `test_back_decrements_cursor`: 3 navigate 후 back → cursor=1.
- `test_forward_increments_cursor`: back 후 forward → cursor=2.
- `test_navigate_truncates_after_cursor`: cursor=1 (3 entries 중 가운데) 에서 navigate → entries.len()=2, cursor=1, 마지막 entry == 새 URL.
- `test_max_entries_caps_at_100`: 105 navigate → entries.len()=100, cursor=99, 처음 5 개 drop.
- `test_set_title_records_in_current`: current 의 title 업데이트.
- `test_clear_resets`: clear → entries.len()=0, cursor=0.

---

## 7. T5 — URL bar 핸들러 + sanitize + scheme allowlist (RG-WB-2 REQ-WB-011~015)

### 7.1 변경 대상

`crates/moai-studio-ui/src/web/surface.rs` 확장:

```rust
impl WebViewSurface {
    pub fn navigate(&mut self, raw_url: &str, cx: &mut Context<Self>) {
        let url = sanitize_url(raw_url);
        if !is_safe_scheme(&url) {
            self.set_status_bar(&format!("Blocked: unsafe scheme — {}", url));
            tracing::warn!(?url, "blocked unsafe scheme navigation");
            return;
        }
        self.history.navigate(&url);
        self.url = url.clone();
        if let Some(ref backend) = self.backend {
            let _ = backend.navigate(&url);
        }
        cx.notify();
    }

    pub fn back(&mut self, cx: &mut Context<Self>) {
        if let Some(entry) = self.history.back() {
            self.url = entry.url.clone();
            if let Some(ref backend) = self.backend { let _ = backend.navigate(&entry.url.clone()); }
            cx.notify();
        }
    }
    // forward 대칭
}

fn sanitize_url(raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("file://") {
        raw.to_string()
    } else if raw.starts_with("localhost") || raw.starts_with("127.0.0.1") || raw.starts_with("[::1]") {
        format!("http://{}", raw)
    } else {
        format!("https://{}", raw)
    }
}

fn is_safe_scheme(url: &str) -> bool {
    !(url.starts_with("javascript:") || url.starts_with("data:text/html") ||
      url.starts_with("chrome:") || url.starts_with("about:") || url.starts_with("view-source:"))
}
```

URL bar element 의 enter 핸들러는 GPUI 의 텍스트 input element + on_submit 콜백 패턴 (Spike 0 시점 확인된 GPUI 0.2.2 API 사용).

### 7.2 단위 테스트 (AC-WB-2, AC-WB-3)

- `test_sanitize_localhost_adds_http`: `"localhost:8080"` → `"http://localhost:8080"`.
- `test_sanitize_bare_domain_adds_https`: `"github.com"` → `"https://github.com"`.
- `test_sanitize_full_url_passthrough`: `"https://example.com"` → 동일.
- `test_is_safe_scheme_javascript_blocked`: `"javascript:alert(1)"` → false.
- `test_is_safe_scheme_data_text_html_blocked`: `"data:text/html,..."` → false.
- `test_is_safe_scheme_chrome_blocked`: `"chrome://flags"` → false.
- `test_is_safe_scheme_https_allowed`: `"https://example.com"` → true.
- `test_navigate_unsafe_scheme_logs_and_skips_history`: javascript:alert 입력 → history.entries.len() 변경 없음 + tracing warn 1.

---

## 8. T6 — DevTools 토글 단축키 (RG-WB-3 REQ-WB-020~023)

### 8.1 변경 대상

`crates/moai-studio-ui/src/web/surface.rs` 의 WebViewSurface 에:

```rust
impl WebViewSurface {
    pub fn toggle_devtools(&mut self, cx: &mut Context<Self>) {
        // USER-DECISION-B 결과에 따라 분기
        #[cfg(not(debug_assertions))]
        if matches!(DEVTOOLS_POLICY, DevToolsPolicy::DebugOnly) {
            return;
        }
        if let Some(ref backend) = self.backend {
            if self.devtools_open { backend.close_devtools(); } else { backend.open_devtools(); }
            self.devtools_open = !self.devtools_open;
            cx.notify();
        }
    }
}
```

`crates/moai-studio-ui/src/lib.rs` 의 RootView 의 `Window::on_key_down` 핸들러에서 단축키 매칭:

```rust
// SPEC-V3-007 T6: DevTools shortcut
if cfg!(target_os = "macos") {
    if ks.modifiers.command && ks.modifiers.alt && ks.code == "i" {
        if let Some(ref tc) = self.tab_container {
            // 활성 탭의 leaf 가 LeafKind::Web 이면 toggle_devtools 호출
            // ...
            return;
        }
    }
} else {
    if ks.modifiers.control && ks.modifiers.shift && ks.code == "I" {
        // 동일 패턴
    }
}
```

### 8.2 단위 테스트 (AC-WB-4)

- `test_toggle_devtools_opens`: state=closed + 단축키 → backend.open_devtools() 1 회 호출 + devtools_open == true.
- `test_toggle_devtools_closes`: state=open + 단축키 → close_devtools() 1 회 + open=false.
- `test_release_build_ignores_shortcut` (cfg(not(debug_assertions)) + USER-DECISION-B=(b) 시): 단축키 입력 → backend 호출 없음.

---

## 9. T7 — Sandbox profile + navigation_handler scheme 검증 (RG-WB-5 REQ-WB-040~044)

### 9.1 변경 대상

`crates/moai-studio-ui/src/web/wry_backend.rs` 의 WryBackend::new 확장:

```rust
let data_dir = workspace_storage_path(workspace_id);  // ~/.moai/webview-data/<ws-id>/
std::fs::create_dir_all(&data_dir).ok();

let webview = WebViewBuilder::new()
    .with_url(initial_url)
    // .with_data_directory(&data_dir)  ← Spike 0 시점 정확한 API 명 확인
    .with_navigation_handler(move |url: String| {
        if !is_safe_scheme_external(&url) {
            tracing::warn!(?url, "blocked unsafe scheme");
            return false;
        }
        if has_mixed_content(&url, &current_origin) {
            tracing::warn!(?url, "blocked mixed content");
            return false;
        }
        true  // allow
    })
    // ...
    .build()?;
```

### 9.2 단위 테스트 (AC-WB-7)

- `test_workspace_storage_path_per_workspace`: workspace_id="ws-1" → `~/.moai/webview-data/ws-1/`.
- `test_storage_dir_created_on_init`: tempdir 기반 mock home → init 후 디렉터리 존재 검증.
- `test_navigation_handler_blocks_javascript`: navigation_handler("javascript:alert") → false return.
- `test_navigation_handler_blocks_mixed_content`: https 페이지에서 http 리소스 → false.
- `test_navigation_handler_allows_https`: "https://example.com" → true.

---

## 10. T8 — web.yaml + WebConfig (RG-WB-5 REQ-WB-045)

### 10.1 변경 대상

신규 `.moai/config/sections/web.yaml`:

```yaml
web:
  trusted_domains:
    - "localhost"
    - "127.0.0.1"
    - "[::1]"
  devtools_enabled: true        # USER-DECISION-C 결과 반영, debug=true / release=false 분기 가능
  max_concurrent_webviews: 10
  url_detector:
    debounce_ms: 5000
    dismiss_silence_minutes: 30
  bridge:
    max_payload_bytes: 1048576  # 1 MB
```

신규 `crates/moai-studio-ui/src/web/config.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-007 T8 — web.yaml 로드 helper.
//! @MX:SPEC: SPEC-V3-007

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WebConfig {
    #[serde(default = "default_trusted_domains")]
    pub trusted_domains: Vec<String>,
    #[serde(default = "default_devtools_enabled")]
    pub devtools_enabled: bool,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_webviews: usize,
    #[serde(default)]
    pub url_detector: UrlDetectorConfig,
    #[serde(default)]
    pub bridge: BridgeConfig,
}

fn default_trusted_domains() -> Vec<String> {
    vec!["localhost".into(), "127.0.0.1".into(), "[::1]".into()]
}
fn default_devtools_enabled() -> bool { true }
fn default_max_concurrent() -> usize { 10 }

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct UrlDetectorConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_dismiss_silence")]
    pub dismiss_silence_minutes: u64,
}
fn default_debounce_ms() -> u64 { 5000 }
fn default_dismiss_silence() -> u64 { 30 }

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct BridgeConfig {
    #[serde(default = "default_payload_max")]
    pub max_payload_bytes: usize,
}
fn default_payload_max() -> usize { 1_048_576 }

impl WebConfig {
    pub fn load_or_default(path: &std::path::Path) -> Self { /* serde_yaml + fallback */ }
}
```

### 10.2 단위 테스트

- `test_load_default_when_missing`: 미존재 path → 디폴트 구조체.
- `test_load_partial_yaml`: 일부 필드만 → 나머지 디폴트.
- `test_load_invalid_yaml_falls_back`: parse 실패 → 디폴트 + tracing warn.
- `test_default_trusted_includes_localhost`: 디폴트 trusted_domains 에 "localhost" 포함.

---

## 11. T9 — BridgeMessage + BridgeRouter (RG-WB-6 REQ-WB-050~054)

### 11.1 변경 대상

신규 `crates/moai-studio-ui/src/web/bridge.rs`:

```rust
//! @MX:ANCHOR: [AUTO] bridge-router
//! @MX:REASON: [AUTO] SPEC-V3-007 RG-WB-6. JS↔Rust 양방향 메시지 라우터.
//!   trusted_domains 기반 origin 검증 + payload 크기 제한 + 채널 allowlist.
//!   fan_in >= 3: WryBackend ipc_handler, WebViewSurface, integration_web.rs.
//! @MX:WARN: [AUTO] 미인증 메시지 / 거대 payload / 미등록 채널 의 3 축에서 보안 위협.
//! @MX:SPEC: SPEC-V3-007

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BridgeKind {
    Request,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    pub id: String,
    pub kind: BridgeKind,
    pub channel: String,
    pub payload: serde_json::Value,
}

pub type BridgeHandler = Box<dyn Fn(&serde_json::Value) -> Option<serde_json::Value> + Send + Sync>;

pub struct BridgeRouter {
    handlers: Mutex<HashMap<String, BridgeHandler>>,
    trusted_domains: Vec<String>,
    max_payload_bytes: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("untrusted origin: {0}")]
    UntrustedOrigin(String),
    #[error("payload too large: {0} > {1}")]
    PayloadTooLarge(usize, usize),
    #[error("unregistered channel: {0}")]
    UnregisteredChannel(String),
    #[error("invalid json: {0}")]
    InvalidJson(String),
    #[error("invalid channel name: {0}")]
    InvalidChannelName(String),
}

impl BridgeRouter {
    pub fn new(trusted_domains: Vec<String>, max_payload_bytes: usize) -> Self { /* ... */ }
    pub fn register<F>(&self, channel: &str, handler: F) -> Result<(), BridgeError>
    where F: Fn(&serde_json::Value) -> Option<serde_json::Value> + Send + Sync + 'static
    {
        // channel 명 정규화 검증 (알파벳 + 하이픈 + 숫자만)
        // handlers.insert(channel, Box::new(handler))
    }
    pub fn dispatch(&self, origin: &str, raw_msg: &str) -> Result<Option<String>, BridgeError> {
        // 1. trusted_domains 매칭 검증
        if !self.is_trusted(origin) {
            tracing::warn!(?origin, "rejected message from untrusted origin");
            return Err(BridgeError::UntrustedOrigin(origin.to_string()));
        }
        // 2. payload 크기 검증
        if raw_msg.len() > self.max_payload_bytes {
            return Err(BridgeError::PayloadTooLarge(raw_msg.len(), self.max_payload_bytes));
        }
        // 3. JSON parse
        let msg: BridgeMessage = serde_json::from_str(raw_msg)
            .map_err(|e| BridgeError::InvalidJson(e.to_string()))?;
        // 4. channel 매칭
        let handlers = self.handlers.lock().unwrap();
        let handler = handlers.get(&msg.channel)
            .ok_or_else(|| BridgeError::UnregisteredChannel(msg.channel.clone()))?;
        // 5. handler 호출
        let result = handler(&msg.payload);
        // 6. request kind 면 reply JS 생성
        if msg.kind == BridgeKind::Request {
            if let Some(value) = result {
                let reply = format!(
                    "window.studio._reply({:?}, {})",
                    msg.id,
                    serde_json::to_string(&value).unwrap()
                );
                return Ok(Some(reply));
            }
        }
        Ok(None)
    }
    fn is_trusted(&self, origin: &str) -> bool { /* domain 매칭 */ }
}

// JS shim — 페이지 로드 시 evaluate_script 로 주입
pub const STUDIO_SHIM_JS: &str = r#"
(function() {
  const pendingRequests = new Map();
  const subscribers = new Map();
  window.studio = {
    send: function(message) {
      const id = crypto.randomUUID();
      const fullMsg = { id: id, kind: message.kind || 'event', channel: message.channel, payload: message.payload };
      window.ipc.postMessage(JSON.stringify(fullMsg));
      if (fullMsg.kind === 'request') {
        return new Promise((resolve) => { pendingRequests.set(id, resolve); });
      }
    },
    on: function(channel, callback) {
      if (!subscribers.has(channel)) subscribers.set(channel, []);
      subscribers.get(channel).push(callback);
    },
    _reply: function(id, result) {
      const resolve = pendingRequests.get(id);
      if (resolve) { resolve(result); pendingRequests.delete(id); }
    }
  };
})();
"#;
```

### 11.2 단위 테스트 (AC-WB-8, AC-WB-9 부분)

- `test_register_valid_channel`: register("log", ...) → Ok.
- `test_register_invalid_channel_name`: register("foo bar", ...) → Err(InvalidChannelName).
- `test_dispatch_trusted_origin_routes`: origin="http://localhost:8080", valid msg → handler 1 회 호출.
- `test_dispatch_untrusted_origin_rejects`: origin="https://untrusted.com" → Err(UntrustedOrigin) + tracing warn.
- `test_dispatch_unregistered_channel`: msg.channel="foo" 미등록 → Err(UnregisteredChannel) + warn.
- `test_dispatch_oversized_payload`: 2MB raw_msg → Err(PayloadTooLarge).
- `test_dispatch_invalid_json`: raw_msg="not json" → Err(InvalidJson).
- `test_dispatch_request_returns_reply_script`: kind=Request + handler 가 Some(value) 반환 → Ok(Some(reply_script)).
- `test_dispatch_event_returns_no_reply`: kind=Event → Ok(None).

---

## 12. T10 — wry IPC handler 등록 + shim 주입 (RG-WB-6 REQ-WB-051~055)

### 12.1 변경 대상

`crates/moai-studio-ui/src/web/wry_backend.rs` 의 WryBackend::new 확장:

```rust
let bridge = bridge_router.clone();
let webview = WebViewBuilder::new()
    // ... (T7 carry)
    .with_ipc_handler(move |msg: wry::Request<String>| {
        let origin = /* 현재 페이지 origin 추출 — wry API 시점 확인 */;
        let body = msg.body().clone();
        match bridge.dispatch(&origin, &body) {
            Ok(Some(reply_script)) => {
                // Rust 측에서 evaluate_script 호출 — backend handle 필요
                // T10: WryBackend 자기 자신을 Arc 로 보유, weak ref 통해 호출
            }
            Ok(None) => {} // event 무응답
            Err(e) => tracing::warn!(?e, "bridge dispatch failed"),
        }
    })
    // page load 콜백에서 STUDIO_SHIM_JS 주입 (trusted_domain 매치 시만)
    // ...
    .build()?;
```

페이지 로드 후 shim 주입 (trusted_domain 매칭 시만):

```rust
// wry 의 on_page_load 콜백 또는 navigate 후 첫 evaluate_script 시점
if bridge_router.is_trusted(&current_origin) {
    self.evaluate_script(STUDIO_SHIM_JS).ok();
}
```

### 12.2 단위 테스트 (AC-WB-8, AC-WB-9)

- T9 의 BridgeRouter 단위 테스트로 dispatch 검증.
- T10 의 wry IPC handler 통합은 T16 (integration_web.rs) 으로 위임 — TestAppContext + mock IPC 또는 실제 wry round-trip (Spike 2 결과 따라).

---

## 13. T11 — url_detector + Debouncer (RG-WB-4 REQ-WB-030, REQ-WB-035)

### 13.1 변경 대상

신규 `crates/moai-studio-ui/src/web/url_detector.rs`:

```rust
//! @MX:NOTE: [AUTO] SPEC-V3-007 RG-WB-4. PTY stdout 에서 localhost URL 자동 감지.
//!   regex 매치 + 5s window debounce + 30분 dismiss silence.
//! @MX:WARN: [AUTO] regex false positive 시 토스트 폭주. dedupe 필수.
//! @MX:SPEC: SPEC-V3-007

use regex::Regex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DetectedUrl {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub full: String,
}

pub fn detect_local_urls(chunk: &str) -> Vec<DetectedUrl> {
    let re = Regex::new(r"(https?)://(localhost|127\.0\.0\.1|\[::1\]):(\d+)(/[^\s]*)?").unwrap();
    re.captures_iter(chunk)
        .filter_map(|cap| {
            let scheme = cap.get(1)?.as_str();
            let host = cap.get(2)?.as_str().to_string();
            let port: u16 = cap.get(3)?.as_str().parse().ok()?;
            let path = cap.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();
            Some(DetectedUrl {
                host: host.clone(),
                port,
                path: path.clone(),
                full: format!("{}://{}:{}{}", scheme, host, port, path),
            })
        })
        .collect()
}

pub struct UrlDetectionDebouncer {
    debounce_window: Duration,
    dismiss_silence: Duration,
    recent: HashMap<String, Instant>,         // URL -> last seen
    dismissed: HashMap<String, Instant>,      // URL -> dismissed at
}

impl UrlDetectionDebouncer {
    pub fn new(debounce_window: Duration, dismiss_silence: Duration) -> Self { /* ... */ }
    pub fn observe(&mut self, urls: Vec<DetectedUrl>) -> Vec<DetectedUrl> {
        let now = Instant::now();
        urls.into_iter()
            .filter(|u| {
                if let Some(&t) = self.dismissed.get(&u.full) {
                    if now.duration_since(t) < self.dismiss_silence { return false; }
                    self.dismissed.remove(&u.full);
                }
                if let Some(&t) = self.recent.get(&u.full) {
                    if now.duration_since(t) < self.debounce_window { return false; }
                }
                self.recent.insert(u.full.clone(), now);
                true
            })
            .collect()
    }
    pub fn dismiss(&mut self, url: &str) {
        self.dismissed.insert(url.to_string(), Instant::now());
    }
}
```

### 13.2 단위 테스트 (AC-WB-5)

- `test_detect_localhost_with_port`: `"Listening on http://localhost:8080"` → 1 매치.
- `test_detect_127_0_0_1`: `"http://127.0.0.1:3000"` → 1 매치.
- `test_detect_ipv6_loopback`: `"http://[::1]:5000"` → 1 매치.
- `test_detect_with_path`: `"http://localhost:8080/api/v1"` → path="/api/v1".
- `test_detect_no_match_for_external`: `"https://github.com"` → 0 매치.
- `test_debouncer_dedupe_within_window`: 같은 URL 100ms 간격 2 회 observe → 두 번째 결과 비어있음.
- `test_debouncer_after_window`: 같은 URL 6 초 간격 2 회 → 두 번째도 emit (`tokio::time::pause()` + `advance(6s)`).
- `test_debouncer_dismissed_url_silenced`: dismiss 후 31 분 이내 같은 URL observe → 비어있음.
- `test_debouncer_dismissed_after_silence`: dismiss 후 31 분 후 observe → emit.

---

## 14. T12 — TerminalSurface stdout observer hook + RootView 통합 (RG-WB-4 REQ-WB-031, REQ-WB-034)

### 14.1 변경 대상 1: `crates/moai-studio-ui/src/terminal/mod.rs` (공개 API 보존)

```rust
impl TerminalSurface {
    /// SPEC-V3-007 T12: stdout observer hook 신규 추가. 기존 메서드 시그니처 무변경.
    /// @MX:NOTE [AUTO]: PTY stdout 텍스트가 갱신될 때 등록된 콜백을 호출.
    /// @MX:SPEC: SPEC-V3-007
    pub fn add_stdout_observer<F>(&mut self, callback: F)
    where F: Fn(&str) + Send + Sync + 'static {
        self.stdout_observers.push(Arc::new(callback));
    }

    // 기존 stdout 갱신 경로 안에서 (기존 pub 메서드가 아닌 internal):
    fn notify_stdout_chunk(&self, chunk: &str) {
        for obs in &self.stdout_observers {
            obs(chunk);
        }
    }
}
```

REQ-WB-064: 신규 메서드만 추가, 기존 시그니처 보존. `crates/moai-studio-terminal/**` (SPEC-V3-002 frozen) 무관.

### 14.2 변경 대상 2: `crates/moai-studio-ui/src/lib.rs`

```rust
pub struct RootView {
    // ... (SPEC-V3-001/004 carry)
    pub url_detector: web::url_detector::UrlDetectionDebouncer,
    pub pending_toasts: Vec<ToastEntry>,
}

#[derive(Debug, Clone)]
pub enum ToastEntry {
    DevServerUrl { url: String, detected_at: SystemTime },
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ... 기존 layout
        // 마지막에 toast_overlay (우하단 stack) 추가
        // self.pending_toasts 의 각 ToastEntry 를 toast element 로 변환
    }
}
```

TerminalSurface 가 RootView 에 등록될 때 stdout_observer 등록:

```rust
let detector_handle = self.url_detector.clone();
terminal_surface.add_stdout_observer(move |chunk| {
    let detected = detect_local_urls(chunk);
    let to_toast = detector_handle.lock().unwrap().observe(detected);
    // RootView 에 message dispatch — cx 핸들 필요 (실제 구현 시 channel 또는 weak ref)
});
```

### 14.3 단위 테스트 (AC-WB-5)

- `test_stdout_observer_invoked_on_chunk`: TerminalSurface mock + add_stdout_observer + notify_stdout_chunk → observer 1 회 호출.
- `test_toast_added_on_url_detection`: mock chunk = "http://localhost:8080" → RootView.pending_toasts.len() == 1.
- `test_duplicate_url_no_double_toast` (5초 윈도우 내): 같은 chunk 2 회 → toast 1 회.

---

## 15. T13 — Toast click → 새 탭 + LeafKind::Web mount (RG-WB-4 REQ-WB-033)

### 15.1 변경 대상

`crates/moai-studio-ui/src/lib.rs` 의 toast element click handler:

```rust
// toast click → 새 탭 + LeafKind::Web mount
let url = toast.url.clone();
if let Some(ref tc) = self.tab_container {
    tc.update(cx, |tc, cx| {
        tc.new_tab(None);  // SPEC-V3-003 carry
        let active_tab = tc.active_tab_mut();
        let surface = cx.new(|_cx| WebViewSurface::new(url.clone()));
        // active_tab.pane_tree 의 last_focused_pane leaf 를 LeafKind::Web(surface) 로 교체
        // ...
    });
}
```

### 15.2 단위 테스트 (AC-WB-6)

- `test_toast_click_creates_new_tab`: 토스트 click → TabContainer.tabs.len() +1.
- `test_toast_click_mounts_web_leaf`: click 후 활성 탭의 leaf 가 LeafKind::Web 이고 url == 감지된 URL.
- `test_toast_click_initiates_navigation`: backend mock 의 navigate 1 회 호출.

---

## 16. T14 — Persistence schema 의 LeafState::Web variant (RG-WB-7 REQ-WB-063)

### 16.1 변경 대상

`crates/moai-studio-workspace/src/persistence.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LeafState {
    // ... 기존 variants (SPEC-V3-003 MS-3)
    Web { url: String },           // ← SPEC-V3-007 신규 (forward-compat minor extension)
}
```

기존 schema version 의 deserialize 가 `Web` variant 미사용 시에도 정상 동작 (forward-compat).

### 16.2 단위 테스트 (AC-WB-10)

- `test_leaf_state_web_serde_roundtrip`: `Web { url: "https://example.com" }` → JSON → 복원.
- `test_legacy_v1_json_still_deserializes`: SPEC-V3-003 시점 JSON 파일 (`Web` variant 없음) → 정상 deserialize.
- `test_unknown_leaf_state_handled_gracefully`: 미래의 `LeafState::Foo` 가 들어와도 panic 없이 fallback (untagged + #[serde(other)]).

---

## 17. T15 — Crash recovery (NFR-WB-7, REQ-WB-006)

### 17.1 변경 대상

`crates/moai-studio-ui/src/web/wry_backend.rs` 에 crash 콜백 등록:

```rust
let state_handle = self.state.clone();
// wry 의 web process crash 콜백 — 정확한 API 시점 확인
webview.on_web_process_crashed(move || {
    *state_handle.lock().unwrap() = WebViewState::Crashed;
});
```

`crates/moai-studio-ui/src/web/surface.rs` 의 Render::render:

```rust
if matches!(self.state, WebViewState::Crashed) {
    // toast element: "Page crashed. Reload?"
    // 클릭 시 self.reload() 호출
}
```

### 17.2 단위 테스트

- `test_state_transitions_to_crashed`: backend crash 콜백 → WebViewSurface.state == Crashed.
- `test_render_shows_crash_toast`: state=Crashed → render 결과에 "Page crashed" 텍스트 포함.
- `test_reload_recovers_from_crashed`: state=Crashed + reload() → 새 backend 인스턴스 생성 + state=Loading.

---

## 18. T16 — Integration test (RG-WB-1/2/3/4/5/6 e2e)

### 18.1 변경 대상

신규 `crates/moai-studio-ui/tests/integration_web.rs`:

```rust
//! Integration tests for SPEC-V3-007 Embedded Web Browser.
//! Test harness:
//!   - USER-DECISION-gpui-test-support=(a) 시: TestAppContext + 실제 wry round-trip (Spike 2 결과 활용)
//!   - USER-DECISION=(b) 시: logic-level mock backend 만 사용

#[tokio::test(flavor = "current_thread")]
async fn test_webview_full_lifecycle() {
    // 1. WebViewSurface 생성 (mock backend)
    // 2. URL bar enter → navigate 호출 검증 + history 추가
    // 3. back/forward 동작
    // 4. javascript:alert 입력 → 차단 + status bar 메시지
    // 5. DevTools 단축키 → toggle
    // 6. mock TerminalSurface stdout chunk → toast 등장
    // 7. toast click → 새 탭 + LeafKind::Web mount
    // 8. bridge dispatch (trusted localhost) → handler 호출
    // 9. bridge dispatch (untrusted origin) → reject
}
```

### 18.2 검증 AC

- AC-WB-1, AC-WB-2, AC-WB-3, AC-WB-4, AC-WB-5, AC-WB-6, AC-WB-7, AC-WB-8, AC-WB-9.

---

## 19. T17 — Regression sweep + clippy/fmt + smoke + commit

### 19.1 검증 명령

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib
cargo test -p moai-studio-ui --test integration_web
cargo build -p moai-studio-app  # smoke
cargo test -p moai-studio-terminal           # SPEC-V3-002 regression
cargo test -p moai-studio-workspace          # persistence regression
cargo test -p moai-studio-ui --lib panes::   # SPEC-V3-003 regression
cargo test -p moai-studio-ui --lib tabs::    # SPEC-V3-003 regression
```

### 19.2 progress.md 갱신

- USER-DECISION-A/B/C/D 결정 결과
- Spike 0/1/2 빌드 결과
- AC-WB-1 ~ AC-WB-10 PASS 여부
- @MX 태그 변경 보고:
  - NEW ANCHOR: web-module-root, wry-backend-impl, web-view-surface, bridge-router (4개)
  - NEW NOTE: webview-backend, navigation-history, url-detector (3개)
  - NEW WARN: wry-native-handle, bridge-security, url-regex-fp (3개)

### 19.3 Commit (현재 브랜치 — feature/SPEC-V3-004-render)

본 SPEC 산출물은 **docs only** (research.md / spec.md / plan.md). 구현 코드는 향후 별도 브랜치 (`feature/SPEC-V3-007-webview`) 에서.

Commit message:
```
docs(spec): SPEC-V3-007 Web Browser Embed v1.0.0 (research/plan/spec)

🗿 MoAI <email@mo.ai.kr>
```

NO PUSH. NO PR.

---

## 20. 위험 요약 (plan 레벨)

| 위험 | 완화 |
|------|------|
| Spike 0 실패 (wry+GPUI handshake 미가능) | T0 fail-fast → SPEC 보류 + GPUI 측 PR 검토 + 사용자 통지 |
| Spike 1 실패 (Linux webkit2gtk-4.1 빌드) | USER-DECISION-Linux=(b) cfg 분기 fallback + ci-rust.yml 두 job |
| Spike 2 실패 (bridge round-trip) | T9/T10 설계 재검토. mock IPC 만으로 검증 가능한 logic-level 분리 |
| USER-DECISION-A=(b) servo 채택 | servo embed API 미공개 → SPEC 보류 + research §2.2 의 wry recommendation 재확인 |
| USER-DECISION-D=(b) global profile 채택 | 워크스페이스 간 cookie 누출 위험 → progress.md 에 명시 + UI 경고 |
| LeafKind::Web variant 추가 위치가 SPEC-V3-006 와 충돌 | OD-WB5 — 먼저 머지된 SPEC 이 도입, 후순위가 variant 만 추가 |
| wry crate CVE | dependabot + 보안 review 정책 (1주 내 반영) |
| 동시 webview 메모리 압력 | max_concurrent_webviews 제한 (디폴트 10) + suspend 로직 (NFR-WB-6) |
| URL detector regex false positive (예: docstring) | dismiss silence 30분 + 사용자 dismiss UI 제공 |
| DevTools release 활성 보안 결함 | USER-DECISION-C=(b) 또는 (c) 로 mitigate |

---

## 21. 영문 보조 요약

This plan decomposes SPEC-V3-007 into 17 tasks across 3 milestones. T0 surfaces 4 USER-DECISION gates (webview backend choice, Linux webkit2gtk version pin, DevTools activation policy, sandbox profile policy) and runs 3 Spikes (wry+GPUI handshake, Linux CI build, bridge round-trip). MS-1 (T1–T3) defines the WebViewBackend trait, WryBackend implementation reflecting Spike 0 results, and the WebViewSurface Entity skeleton with LeafKind::Web variant addition (location depends on V3-006 merge order). MS-2 (T4–T8) layers NavigationHistory, URL bar handler with scheme allowlist, DevTools toggle shortcut respecting build profile, sandbox profile directories, and web.yaml configuration. MS-3 (T9–T15) builds the BridgeRouter with trusted_domains/payload-size/channel-allowlist enforcement, wry IPC handler integration with window.studio JS shim injection, regex-based PTY URL detector with 5s debounce + 30min dismiss silence, TerminalSurface stdout observer hook (preserving SPEC-V3-002 frozen zone), toast click dispatch creating LeafKind::Web tabs, persistence schema minor extension (LeafState::Web { url } variant, forward-compat), and crash recovery toast. T16 provides integration test harness (TestAppContext-based or logic-level fallback per gpui-test-support decision); T17 runs full regression sweep including SPEC-V3-002/003/004 carry tests, formats progress.md, and commits docs-only artifacts on the current branch (`feature/SPEC-V3-004-render`) without push or PR. Each task maps explicitly to AC-WB-1 through AC-WB-10 from spec.md §10.

---

작성 완료: 2026-04-25
산출물: research.md / spec.md / plan.md (3-file 표준 준수).
