---
id: SPEC-V3-007
version: 1.0.0
status: draft
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-004]
parallel_with: [SPEC-V3-005, SPEC-V3-006, SPEC-V3-008, SPEC-V3-009]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, ui, gpui, webview, wry, browser, in-app-docs, dev-server-preview, oauth]
revision: v1.0.0 (initial draft, 4-surface vision 의 4번째 surface)
---

# SPEC-V3-007: Embedded Web Browser — wry 기반 in-app webview surface (URL navigation + DevTools + dev server auto-detect + JS bridge)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-studio v3 비전 4 대 surface (Terminal / FileExplorer / Markdown+CodeViewer / **Web Browser**) 중 4 번째 surface 의 단일 SPEC. wry 0.50.x 채택 (Tauri-tested, OS-native: WKWebView/webkit2gtk-4.1). RG-WB-1 ~ RG-WB-7, AC-WB-1 ~ AC-WB-10, MS-1/MS-2/MS-3, USER-DECISION 4 게이트. SPEC-V3-004 (render layer) 선행, SPEC-V3-005/006/008/009 와 병행. terminal/panes/tabs core 무변경 carry. |

---

## 1. 개요

### 1.1 목적

moai-studio v3 의 4 대 surface (Terminal, File Explorer, Markdown/Code Viewer, **Web Browser**) 중 마지막 surface 인 **Embedded Web Browser** 를 GPUI 0.2.2 위에 도입한다. 외부 브라우저로 컨텍스트 스위치 없이 인앱에서 다음 3 가지 시나리오를 처리한다:

1. **In-app documentation viewer** — SPEC 문서 / Markdown 본문의 외부 링크 클릭 시 활성 탭의 leaf 가 webview 로 교체되어 페이지를 가시.
2. **Dev server preview** — `vite dev` / `next dev` / `python -m http.server` 등이 출력하는 `localhost:NNNN` URL 을 자동 감지하여 사용자에게 토스트 → 클릭 시 새 webview 탭으로 미리보기.
3. **OAuth flow** — GitHub / Google OAuth callback 을 인앱에서 처리, redirect 결과를 Studio 의 token store 로 안전 전달 (별도 SPEC 으로 확장 가능).

본 SPEC 의 채택 라이브러리는 `wry` (tauri-apps) — production-tested, OS-native 백엔드 (macOS WKWebView / Linux webkit2gtk-4.1 / Windows WebView2 [본 SPEC 비대상]).

### 1.2 SPEC 분리 / escape-hatch 전략

- 본 SPEC 은 SPEC-V3-004 의 `LeafKind` generic 자리에 `Web(Entity<WebViewSurface>)` variant 를 추가하는 형태로 통합.
- SPEC-V3-006 가 동일 leaf payload generic 을 사용하므로 enum variant 추가 시점은 둘 중 먼저 머지되는 SPEC 이 결정. 후순위 SPEC 은 variant 만 추가 (research §11.3 참조).
- 본 SPEC v1 은 **macOS 14+ + Ubuntu 22.04+** 만 (Windows 는 SPEC-V3-002/003 carry). webview backend 는 USER-DECISION 으로 `wry` 디폴트.

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-007/research.md` — 라이브러리 분석, GPUI 통합 전략, USER-DECISION 게이트, 위험 분석, Spike 계획.
- `.moai/specs/SPEC-V3-004/spec.md` §7 — RootView ↔ TabContainer ↔ PaneTree leaf payload generic.
- `.moai/specs/SPEC-V3-006/research.md` §1.3 — `LeafKind` 도입 합의 (선/후 머지 정책).
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 무변경 원칙.
- `.moai/specs/SPEC-V3-003/spec.md` — Pane/Tab logic 의 leaf payload 추상.
- [Tauri wry README](https://github.com/tauri-apps/wry) — API reference.
- `.claude/rules/moai/core/lsp-client.md` — third-party Rust crate 채택 패턴 reference.

---

## 2. 배경 및 동기

상세 분석은 `.moai/specs/SPEC-V3-007/research.md` §1 ~ §10 참조. SPEC 독자가 요구사항 진입 전에 알아야 할 최소 맥락:

- **4-surface 비전 완성**: Terminal/FileTree/Viewer 까지 만들어진 IDE 에 외부 페이지 렌더링이 빠지면 사용자는 docs / OAuth / dev server preview 를 매번 외부 브라우저로 떠나야 한다. 컨텍스트 스위치 비용 제거가 핵심 motivation.
- **wry 채택 근거** (research §2): production-tested (Tauri 가 100k+ 앱에서 사용), OS-native 렌더링으로 보안 패치 자동 반영, 바이너리 크기 영향 ~5MB, dual MIT/Apache-2.0 라이센스로 moai-studio MIT 와 호환.
- **GPUI ↔ wry 통합 전략** (research §3): native subview 모델. GPUI 가 chrome (URL bar / status / history button) 을 그리고, wry::WebView 를 OS native child view 로 attach. 화면 영역 동기화는 GPUI layout 후 deferred `set_bounds` 로 처리.
- **변경 금지 zone**: SPEC-V3-002 의 terminal core 와 SPEC-V3-003 의 panes/tabs logic 모두 carry. 본 SPEC 은 **enum variant 추가 + 신규 모듈** 으로만 자라며, 기존 시그니처는 보존.
- **보안 우선 설계**: bridge 는 trusted-domain allowlist 기반, sandbox profile 은 per-workspace 격리, navigation 은 scheme allowlist + mixed-content reject.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `WebViewSurface` 가 GPUI `Entity` 로 생성 가능하며 `impl Render for WebViewSurface` 가 chrome (URL bar + history button + status bar) 와 webview 자식 영역을 그린다.
- G2. URL navigation API (`navigate(url)` / `back()` / `forward()` / `reload()` / `stop_loading()`) 가 동작하며, NavigationHistory 가 최대 100 entry 까지 추적된다.
- G3. DevTools 토글 단축키 (`Cmd+Opt+I` / `Ctrl+Shift+I`) 가 동작한다 (USER-DECISION-B 결과에 따라 build profile 분기 가능).
- G4. PTY 출력 watch 로 `localhost:NNNN` URL 을 자동 감지, debounce 후 사용자 토스트 → 클릭 시 새 탭의 leaf 로 webview 마운트.
- G5. JS ↔ Rust bidirectional bridge 가 동작 (`window.studio.send(...)` JS-side, `BridgeRouter::register(...)` Rust-side). trusted_domains allowlist 기반.
- G6. WebView 가 OS-level 보안 격리 (per-workspace sandbox profile) 안에서 동작하며, navigation handler 가 위험 scheme (`javascript:`, `chrome:` 등) 을 reject.
- G7. SPEC-V3-002 (terminal core), SPEC-V3-003 (pane/tab logic), SPEC-V3-004 (render layer) 의 공개 API 무변경 carry.
- G8. macOS 14+ + Ubuntu 22.04+ 양 플랫폼에서 빌드 + 실행 + 동일 사용자 가시 동작.

### 3.2 비목표 (Non-Goals)

- N1. **Windows 빌드** — SPEC-V3-002/003 carry. WebView2 (Windows) 는 wry 가 지원하나 GPUI 0.2.2 의 Windows 미지원으로 본 SPEC 비대상.
- N2. **iOS / Android** — wry 가 모바일 지원하나 본 SPEC 은 데스크톱만.
- N3. **광고 차단 / 사용자 스크립트 / 확장 API** — 별도 SPEC (SPEC-V3-WEB-EXTENSIONS).
- N4. **인앱 print preview / PDF export** — 별도 SPEC (SPEC-V3-WEB-PRINT).
- N5. **Tab reorder / detach to new window** — SPEC-V3-003 N6/N7 carry.
- N6. **History persistence (탭 close 후 history 복원)** — 본 SPEC v1 은 마지막 URL 만 persistence schema 에 추가. history 전체는 별도 SPEC.
- N7. **다중 webview 탭의 동시 active** — 활성 탭만 시각 활성, unfocused webview 는 background 로 존재 (메모리 압력 시 suspend, research §9.2).
- N8. **OAuth 자동 token 추출 / store 통합** — bridge 는 도구만 제공, 실제 OAuth 흐름 + token persistence 는 별도 SPEC.
- N9. **WebRTC / 카메라 / 마이크 접근 권한 prompt** — 본 SPEC v1 은 OS webview 의 디폴트 정책 그대로 사용 (즉, 대부분 거부).
- N10. **사용자 스크립트 주입** — `evaluate_script` 는 internal API, end-user 노출 없음.
- N11. **Servo migration** — 별도 SPEC (장기).
- N12. **새 design token 추가** — 기존 토큰 재사용 (`toolbar.tab.active.background` 등).

---

## 4. 사용자 스토리

- **US-WB1**: 개발자가 SPEC 문서 본문의 `https://github.com/zed-industries/zed` 링크를 더블클릭한다 → 활성 탭 leaf 가 webview 로 교체되어 GitHub 페이지가 가시. 외부 브라우저로 떠나지 않음.
- **US-WB2**: 개발자가 새 webview leaf 의 주소창에 `https://docs.rs/wry` 입력 + Enter → navigate. 같은 webview 안에서 페이지가 바뀌고 history 에 추가되어 back 버튼 동작.
- **US-WB3**: 개발자가 `Cmd+Opt+I` (macOS) / `Ctrl+Shift+I` (Linux) 누름 → DevTools 패널이 별도 창으로 열려 페이지 inspect 가능.
- **US-WB4**: 개발자가 별도 터미널 pane 에서 `python -m http.server 8080` 실행 → stdout 에 `Serving HTTP on 0.0.0.0 port 8080` 출력 → moai-studio 가 `http://localhost:8080` 자동 감지 → 화면 우하단에 "Open localhost:8080 in Studio?" 토스트 → 사용자 클릭 시 새 webview 탭 마운트.
- **US-WB5**: 개발자가 webview 안에서 도메인이 신뢰 리스트 (`localhost`) 인 페이지 JS 콘솔에 `window.studio.send({ channel: 'log', payload: { level: 'info', message: 'hi' } })` 입력 → Rust 의 `BridgeRouter` 가 메시지 수신 → tracing log 로 `info: hi` 출력.
- **US-WB6**: 개발자가 webview 가 crash 한 후 화면에 "Page crashed. Reload?" 토스트 표시 → 클릭 시 동일 URL 로 reload + bridge 재등록.

---

## 5. 기능 요구사항 (EARS)

### RG-WB-1 — WebViewSurface Entity (SPEC-V3-004 패턴 차용)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-WB-001 | Ubiquitous | 시스템은 `WebViewSurface` 를 GPUI `cx.new(\|cx\| WebViewSurface::new(initial_url))` 호출로 `Entity<WebViewSurface>` 생성 가능하도록 한다. | The system **shall** allow `WebViewSurface` to be instantiated as `Entity<WebViewSurface>` via `cx.new`. |
| REQ-WB-002 | Ubiquitous | 시스템은 `WebViewSurface` 에 대해 `impl Render` 트레잇을 제공한다. 해당 구현은 (a) URL bar, (b) navigation buttons (back/forward/reload), (c) webview 영역 placeholder, (d) status bar 를 세로로 쌓아 `IntoElement` 로 반환한다. | The system **shall** implement `Render for WebViewSurface` returning a vertical stack of URL bar, navigation buttons, webview placeholder, and status bar. |
| REQ-WB-003 | Ubiquitous | 시스템은 `WebViewBackend` trait 을 정의한다. trait 은 `navigate(url)`, `back`, `forward`, `reload`, `evaluate_script(s)`, `set_bounds(rect)`, `open_devtools`, `close_devtools` 메서드를 가진다. wry 구현체 `WryBackend` 가 trait 을 구현한다. | The system **shall** define a `WebViewBackend` trait and provide a `WryBackend` implementation. |
| REQ-WB-004 | Event-Driven | `WebViewSurface::render` 에서 webview 영역의 GPUI bounds 가 결정되면, 시스템은 backend 의 `set_bounds(rect)` 를 deferred 호출로 동기화한다. | When the GPUI layout assigns bounds to the webview area, the system **shall** synchronize via deferred `backend.set_bounds(rect)`. |
| REQ-WB-005 | Unwanted | 시스템은 `WebViewSurface::render` 가 backend 미초기화 (예: Spike 0 검증 실패 fallback) 상태에서도 panic 하지 않도록 한다. 빈 placeholder + "WebView unavailable" 메시지로 fall through 한다. | The system **shall not** panic when the backend is unavailable; it must fall through to a placeholder. |
| REQ-WB-006 | State-Driven | `WebViewSurface.state == Crashed` 인 동안, 시스템은 webview 영역 자리에 "Page crashed. Reload?" 토스트 element 를 렌더한다. | While `state == Crashed`, the system **shall** render a "Page crashed. Reload?" toast in place of the webview area. |

### RG-WB-2 — URL Navigation + History

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-WB-010 | Ubiquitous | 시스템은 `NavigationHistory` 자료구조를 정의한다. `entries: Vec<HistoryEntry>` (최대 100), `cursor: usize` 를 가지며 `navigate/back/forward/clear_history` 메서드를 제공한다. | The system **shall** define `NavigationHistory` with capped entries and a cursor. |
| REQ-WB-011 | Event-Driven | 사용자가 URL bar 에 URL 입력 후 Enter 를 누르면, 시스템은 (a) URL sanitize (`localhost:NNNN` → `http://localhost:NNNN`), (b) NavigationHistory 에 entry 추가 + cursor 갱신, (c) backend 의 `navigate(url)` 호출, (d) `cx.notify()` 발화 순서로 처리한다. | When the user submits a URL, the system **shall** sanitize, push to history, call `backend.navigate`, and notify. |
| REQ-WB-012 | Event-Driven | 사용자가 back 버튼 클릭 또는 `Cmd+[` 단축키 입력 시, `history.cursor > 0` 인 조건에서 시스템은 cursor 를 -1 하고 backend 의 `navigate(history.entries[cursor].url)` 를 호출한다. forward 도 대칭. | When back/forward is triggered, the system **shall** move the cursor and re-navigate to the historical URL. |
| REQ-WB-013 | Event-Driven | webview 의 페이지 로드가 완료되면 (backend 의 `on_page_load` 콜백), 시스템은 페이지 `<title>` 을 추출하여 `HistoryEntry::title` 에 기록하고 `cx.notify()` 발화한다. | When page load completes, the system **shall** record the title in the current history entry. |
| REQ-WB-014 | Unwanted | 시스템은 NavigationHistory.entries.len() 이 100 을 초과하지 않도록 한다. 100 도달 시 가장 오래된 entry 가 drop 된다 (FIFO). cursor 는 변동에 맞춰 -1 보정. | The system **shall not** allow `entries.len()` to exceed 100; oldest entry drops on overflow with cursor correction. |
| REQ-WB-015 | Unwanted | 시스템은 `javascript:`, `data:text/html,`, `chrome:`, `about:`, `view-source:` 스킴 URL navigation 시도를 reject 하고 status bar 에 "Blocked: unsafe scheme" 표시 + tracing warn 1 건 기록한다. | The system **shall not** allow navigation to `javascript:`, `data:text/html,`, `chrome:`, `about:`, or `view-source:` schemes. |

### RG-WB-3 — DevTools 토글

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-WB-020 | Ubiquitous | 시스템은 `WebViewBuilder` 빌드 시 `with_devtools(true)` 옵션을 활성화한다 (USER-DECISION-B 결과에 따라 build profile 분기). | The system **shall** enable `with_devtools(true)` according to USER-DECISION-B. |
| REQ-WB-021 | Event-Driven | 사용자가 webview leaf 가 활성인 상태에서 `Cmd+Opt+I` (macOS) 또는 `Ctrl+Shift+I` (Linux) 단축키를 입력하면, 시스템은 `WebViewSurface.devtools_open` 토글 + `backend.open_devtools()` 또는 `close_devtools()` 호출을 한다. | When the DevTools shortcut is pressed, the system **shall** toggle the DevTools state and call the backend accordingly. |
| REQ-WB-022 | State-Driven | `devtools_open == true` 인 동안, 시스템은 OS-level DevTools 창이 열려 있는 상태를 유지한다 (별도 OS window). | While `devtools_open == true`, the system **shall** maintain the OS-level DevTools window. |
| REQ-WB-023 | Unwanted | USER-DECISION-B 가 (b) debug-only 로 결정된 경우, 시스템은 release build 에서 단축키 입력을 무시한다 (`#[cfg(debug_assertions)]` 분기). | When USER-DECISION-B selects debug-only, the system **shall not** respond to the shortcut in release builds. |

### RG-WB-4 — Dev Server URL 자동 감지

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-WB-030 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/web/url_detector.rs` (신규) 에 `detect_local_urls(stdout_chunk: &str) -> Vec<DetectedUrl>` 함수를 제공한다. 정규식 `r"https?://(localhost\|127\.0\.0\.1\|\[::1\]):(\d+)(/[^\s]*)?"` 매치. | The system **shall** provide `detect_local_urls` in `web/url_detector.rs` matching the regex. |
| REQ-WB-031 | Event-Driven | `crates/moai-studio-ui/src/terminal/` 측 stdout text 갱신 시 (TerminalSurface stdout hook), 시스템은 `detect_local_urls` 호출 → 매치된 URL 을 `UrlDetectionDebouncer` 에 push 한다. 5 초 윈도우 내 같은 URL 중복 무시. | When PTY stdout updates, the system **shall** call `detect_local_urls` and debounce duplicates for 5 seconds. |
| REQ-WB-032 | Event-Driven | `UrlDetectionDebouncer` 가 새 URL 을 emit 하면, 시스템은 RootView 에 토스트 element ("Open <URL> in Studio?") 를 표시한다. | When the debouncer emits a new URL, the system **shall** show a toast element. |
| REQ-WB-033 | Event-Driven | 사용자가 토스트를 클릭하면, 시스템은 활성 TabContainer 에 새 탭을 생성 (`TabContainer::new_tab(...)`) 하고 그 leaf 를 `LeafKind::Web(Entity<WebViewSurface>)` 로 마운트하여 URL 을 navigate 한다. | When the user clicks the toast, the system **shall** create a new tab with a `LeafKind::Web` leaf and navigate to the URL. |
| REQ-WB-034 | Unwanted | 시스템은 `crates/moai-studio-terminal/**` 의 어떤 파일도 수정하지 않는다 (SPEC-V3-002 RG-P-7 carry). PTY stdout watch 는 `crates/moai-studio-ui/src/terminal/` 의 wrapper 에서만 처리. | The system **shall not** modify any file in `crates/moai-studio-terminal/**`. |
| REQ-WB-035 | Unwanted | 시스템은 토스트 알림 폭주 (예: dev server log 에 URL 이 매 frame 출력) 를 방지하기 위해 동일 URL 의 토스트는 5 초 윈도우 내 1 회만 표시한다. 사용자가 dismiss 한 URL 은 30 분 동안 재토스트 없음. | The system **shall not** flood the user with toasts; same-URL toasts are deduped within 5s and dismissed URLs are silenced for 30 minutes. |

### RG-WB-5 — Sandboxing + CSP + Same-Origin (보안)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-WB-040 | Ubiquitous | 시스템은 워크스페이스별 sandbox profile 디렉터리 (`~/.moai/webview-data/<workspace-id>/`) 를 webview 의 storage 위치로 지정한다 (USER-DECISION-A=(a) 채택 시). | The system **shall** assign per-workspace storage directories under `~/.moai/webview-data/<workspace-id>/`. |
| REQ-WB-041 | Event-Driven | 새 페이지로 navigate 시도 시, 시스템은 `with_navigation_handler` 콜백에서 URL scheme 을 검증 한다. 허용 scheme: `http`, `https`, `file://` (단 워크스페이스 root prefix 만). | When navigation is attempted, the system **shall** validate the URL scheme via `with_navigation_handler`. |
| REQ-WB-042 | Unwanted | 시스템은 mixed content (https 페이지 안의 http subresource) 의 자동 로딩을 reject 한다. tracing warn 1 건 + status bar "Blocked mixed content: <url>" 표시. | The system **shall not** auto-load mixed content; rejection is logged and surfaced to the status bar. |
| REQ-WB-043 | Ubiquitous | 시스템은 webview 의 cookie / localStorage / IndexedDB 영역이 호스트 OS 시스템 브라우저의 storage 와 분리되도록 wry 의 ephemeral / per-profile 옵션을 사용한다. | The system **shall** isolate webview storage from the host OS browser. |
| REQ-WB-044 | State-Driven | webview 의 origin 이 `config.web.trusted_domains` 리스트에 매치되지 않는 동안, 시스템은 JS bridge 의 `with_ipc_handler` 콜백에서 들어오는 메시지를 reject 한다 (RG-WB-6 참조). | While the page's origin is not in `trusted_domains`, the system **shall** reject incoming bridge messages. |
| REQ-WB-045 | Ubiquitous | 시스템은 `config.web.trusted_domains` 의 디폴트로 `["localhost", "127.0.0.1", "[::1]"]` 를 가진다. 사용자는 `.moai/config/sections/web.yaml` 로 도메인을 추가할 수 있다. | The system **shall** default `trusted_domains` to localhost variants. |

### RG-WB-6 — JS ↔ Rust Bidirectional Bridge

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-WB-050 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/web/bridge.rs` (신규) 에 `BridgeMessage { id, kind, channel, payload }` 자료구조와 `BridgeRouter` 라우터를 정의한다. `BridgeRouter::register("channel-name", handler)` API 로 채널 등록. | The system **shall** define `BridgeMessage` and `BridgeRouter` with `register` API. |
| REQ-WB-051 | Event-Driven | wry 의 `with_ipc_handler` 가 JS 측 메시지를 수신하면, 시스템은 페이로드를 JSON parse → `BridgeMessage` 로 변환 → 채널 매칭 → 등록된 handler 호출 순서로 처리한다. | When wry's IPC handler fires, the system **shall** parse, route, and dispatch to the registered handler. |
| REQ-WB-052 | Event-Driven | Rust 측 handler 가 `request` kind 메시지에 응답하려 할 때, 시스템은 `backend.evaluate_script(...)` 로 JS 측의 `window.studio._reply(id, result)` 를 호출하여 응답을 전달한다. | When a `request` requires a reply, the system **shall** dispatch via `evaluate_script` calling `window.studio._reply`. |
| REQ-WB-053 | Unwanted | 시스템은 `BridgeRouter` 에 등록되지 않은 채널의 메시지를 무시하고 tracing warn 1 건만 기록한다. JS 페이지에 에러 응답을 보내지 않는다 (정보 누출 방지). | The system **shall not** dispatch to unregistered channels; warn-log only, no JS-side error reply. |
| REQ-WB-054 | Unwanted | 시스템은 `BridgeMessage::payload` 의 직렬화된 크기가 1 MB 를 초과하면 reject 한다. tracing warn + 발신 페이지에 error reply ("payload too large"). | The system **shall not** accept payloads exceeding 1 MB; oversized messages are rejected with an error reply. |
| REQ-WB-055 | State-Driven | webview 의 페이지 origin 이 `config.web.trusted_domains` 매치 동안, 시스템은 페이지 로드 직후 `evaluate_script` 로 `window.studio = { send, on, _reply }` JS shim 을 주입한다. 비매치 origin 에서는 shim 미주입. | While the origin is trusted, the system **shall** inject the `window.studio` shim; otherwise no injection. |

### RG-WB-7 — Frozen Zone Carry (SPEC-V3-002/003/004 무변경)

| REQ ID | 패턴 | 요구사항 (한국어) |
|--------|------|-------------------|
| REQ-WB-060 | Ubiquitous | 시스템은 `crates/moai-studio-terminal/**` 의 어떤 파일도 수정하지 않는다. SPEC-V3-002 의 13 + 회귀 tests 가 본 SPEC 모든 milestone 에서 regression 0 으로 유지된다. |
| REQ-WB-061 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}` 의 공개 API 를 변경하지 않는다. |
| REQ-WB-062 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}` 의 공개 API 를 변경하지 않는다. `LeafKind::Web` enum variant 추가는 enum extension 으로만 처리 (signature 무변경). |
| REQ-WB-063 | Ubiquitous | 시스템은 `crates/moai-studio-workspace/src/persistence.rs` 의 `moai-studio/panes-v1` schema 의 기존 필드를 변경하지 않는다. `LeafState::Web { url }` variant 추가는 minor extension (forward-compatible). |
| REQ-WB-064 | Ubiquitous | 시스템은 `crates/moai-studio-ui/src/terminal/mod.rs` 의 공개 API 를 보존한다. PTY stdout observer hook 추가는 신규 메서드 추가만 (기존 메서드 시그니처 무변경). |

---

## 6. 비기능 요구사항

### 6.1 성능

- NFR-WB-1. 첫 webview leaf 마운트 후 첫 페이지 paint ≤ 500 ms (네트워크 지연 제외, OS-level webview 의 cold start 기준).
- NFR-WB-2. webview 영역 layout 변경 (윈도우 크기 변경 / 분할 비율 변경) → `set_bounds` 동기화 → OS webview 재배치까지 ≤ 100 ms.
- NFR-WB-3. PTY URL 자동 감지 → 토스트 표시까지 (debounce 5s 외) ≤ 200 ms.
- NFR-WB-4. JS bridge round-trip (`window.studio.send` → Rust handler → `_reply`) latency ≤ 50 ms.

### 6.2 메모리

- NFR-WB-5. webview tab 1 개 당 메모리 점유 측정 — 빈 about:blank 페이지 ≤ 80 MB. 일반 docs 페이지 ≤ 200 MB.
- NFR-WB-6. 동시 webview tab 개수 제한 — config 기본 10 개. 초과 시 가장 오래된 unfocused webview suspend (URL/history 보존, 페이지 unload).

### 6.3 안정성

- NFR-WB-7. wry crash (페이지 process kill) → Studio 의 `WebViewState::Crashed` 전이 + 토스트 표시. Studio 본체는 panic 하지 않음.
- NFR-WB-8. `cargo run -p moai-studio-app` 에서 webview tab 10 회 open/close cycle 후 메모리 leak ≤ 5 MB (zombie webview window 없음).

### 6.4 보안

- NFR-WB-9. webview cookie / localStorage 가 호스트 OS 시스템 브라우저와 격리됨. `~/.moai/webview-data/<workspace-id>/` 외 디렉터리에 storage 가 생성되지 않음 (USER-DECISION-A=(a) 채택 시).
- NFR-WB-10. JS bridge 가 `trusted_domains` 외 origin 에서 활성화되지 않음. 비신뢰 origin 의 `window.ipc.postMessage` 호출은 100% reject.
- NFR-WB-11. CVE 패치 — wry crate 의 보안 패치 release 시 dependabot 또는 manual review 후 1 주일 내 반영 (정기 보수 정책).

### 6.5 호환성

- NFR-WB-12. macOS 14+ + Ubuntu 22.04+ 양쪽에서 동일 사용자 가시 동작 (URL navigation, DevTools, bridge round-trip, dev-server detect).

### 6.6 접근성

- NFR-WB-13. URL bar 는 키보드 접근 (Tab focus + 직접 입력) 가능.
- NFR-WB-14. webview 내부의 a11y 는 OS webview 의 디폴트 정책 따름 (Studio 가 별도 가공하지 않음).

---

## 7. 아키텍처

### 7.1 모듈 토폴로지

```
crates/moai-studio-ui/src/
├── web/                        ← 신규 모듈 (본 SPEC 도입)
│   ├── mod.rs                  ← 진입점 + WebViewSurface re-export
│   ├── surface.rs              ← WebViewSurface struct + impl Render
│   ├── backend.rs              ← WebViewBackend trait
│   ├── wry_backend.rs          ← WryBackend impl
│   ├── history.rs              ← NavigationHistory
│   ├── url_detector.rs         ← detect_local_urls + Debouncer
│   ├── bridge.rs               ← BridgeMessage + BridgeRouter
│   └── config.rs               ← web.yaml 로드 helper
├── terminal/mod.rs             ← stdout observer hook 추가 (REQ-WB-031, 공개 API 보존)
├── tabs/container.rs           ← LeafKind::Web variant 추가 (만약 V3-006 가 먼저 머지 안 했다면)
└── lib.rs                      ← RootView 에 toast element + LeafKind::Web 분기 추가
```

### 7.2 데이터 흐름

```
┌──────────────────────────────────────────────────────────────────────┐
│  RootView (Entity<RootView>)                                         │
│  ├── tab_container: Option<Entity<TabContainer>>     ← SPEC-V3-004   │
│  ├── url_detector: UrlDetectionDebouncer             ← 신규 (RG-WB-4)│
│  └── pending_toasts: Vec<ToastEntry>                 ← 신규          │
│                                                                       │
│  Render::render:                                                      │
│    ├── ... (SPEC-V3-001/004 carry)                                   │
│    └── toast_overlay(self.pending_toasts) ← 우하단 stack             │
└──────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼ TabContainer 의 활성 탭의 PaneTree leaf
┌──────────────────────────────────────────────────────────────────────┐
│  LeafKind::Web(Entity<WebViewSurface>)                               │
│                                                                       │
│  WebViewSurface:                                                      │
│  ├── url: String                                                     │
│  ├── history: NavigationHistory  (RG-WB-2)                           │
│  ├── state: WebViewState         (Loading/Ready/Error/Crashed)       │
│  ├── backend: Box<dyn WebViewBackend>  (RG-WB-1)                     │
│  ├── bridge: Arc<BridgeRouter>   (RG-WB-6)                           │
│  ├── devtools_open: bool         (RG-WB-3)                           │
│  └── bounds: Option<Bounds<f32>> (Render 시 갱신)                    │
│                                                                       │
│  Render::render:                                                      │
│    ├── url_bar(self.url)                                             │
│    ├── nav_buttons(back/forward/reload)                              │
│    ├── webview_placeholder(self.bounds)  ← OS native 자식이 그릴 영역│
│    └── status_bar(self.state)                                        │
└──────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼ backend.set_bounds(rect) [deferred]
┌──────────────────────────────────────────────────────────────────────┐
│  wry::WebView (OS native subview)                                    │
│  ├── WKWebView (macOS) / webkit2gtk (Linux)                          │
│  ├── with_devtools(true)                                             │
│  ├── with_navigation_handler(...)  ← RG-WB-5 scheme 검증             │
│  ├── with_ipc_handler(...)         ← RG-WB-6 bridge 입구             │
│  └── with_data_directory(workspace_storage_path) ← RG-WB-5 sandbox   │
└──────────────────────────────────────────────────────────────────────┘
```

### 7.3 PTY URL 자동 감지 흐름

```
TerminalSurface (SPEC-V3-002 wrapper, ui crate 안)
     │
     │ stdout text 갱신 시 콜백
     ▼
url_detector::detect_local_urls(chunk: &str) -> Vec<DetectedUrl>
     │
     ▼
UrlDetectionDebouncer (5s 윈도우)
     │ ├── 같은 URL dedupe
     │ └── dismissed URL silence (30분)
     ▼
RootView::pending_toasts.push(ToastEntry::DevServerUrl { url })
     │
     ▼ Render::render → toast_overlay
사용자 클릭
     │
     ▼ TabContainer::new_tab + leaf = LeafKind::Web(WebViewSurface::new(url))
```

### 7.4 JS bridge 흐름

```
[JS]                                              [Rust]
window.studio.send({                  ─┐
  channel: 'log',                      │
  kind: 'event',                       │
  payload: { level: 'info', ... }      │
})                                     │
     │                                 │
     ▼ window.ipc.postMessage(JSON)    │ (wry IPC channel)
                                       │
                                       ▼ with_ipc_handler(string)
                                       parse → BridgeMessage
                                       origin 검증 (RG-WB-5)
                                       payload 크기 검증 (REQ-WB-054)
                                       channel 매칭
                                       handler.call(payload)
                                          │
                                          ▼ handler 가 응답을 반환하면
                                          backend.evaluate_script(
                                            "window.studio._reply(id, result)"
                                          )
[JS]                                       │
window.studio._reply(id, result) ◀─────────┘
  → resolves Promise from window.studio.send
```

---

## 8. Milestone

본 SPEC 은 3 milestone 으로 분할한다.

### MS-1: WebViewSurface Entity + wry backend skeleton (RG-WB-1, RG-WB-7 부분, USER-DECISION 4 게이트 + Spike 0/1)

- **범위**: USER-DECISION-A/B/C/D 호출 + Spike 0 (wry+GPUI handshake) + Spike 1 (Linux webkit2gtk-4.1 CI 빌드). `WebViewBackend` trait 정의 + `WryBackend` skeleton + `WebViewSurface` Entity 의 placeholder render. `LeafKind::Web` enum variant 추가 (SPEC-V3-006 미머지 시).
- **포함 요구사항**: REQ-WB-001 ~ REQ-WB-005, REQ-WB-060 ~ REQ-WB-064.
- **시연 가능 상태**: `cargo run -p moai-studio-app` 실행 시 빈 webview leaf 가 활성 탭에 마운트되어 URL bar + 빈 회색 영역 + status bar 가 가시. 실제 페이지 로딩은 MS-2.

### MS-2: URL navigation + History + DevTools + sandbox (RG-WB-2, RG-WB-3, RG-WB-5)

- **범위**: `NavigationHistory` 구현 + URL bar enter 핸들러 + back/forward 동작 + DevTools 토글 단축키 + per-workspace sandbox profile + scheme allowlist navigation_handler.
- **포함 요구사항**: REQ-WB-010 ~ REQ-WB-015, REQ-WB-020 ~ REQ-WB-023, REQ-WB-040 ~ REQ-WB-045.
- **시연 가능 상태**: 사용자가 URL 입력 + Enter → 페이지 가시. back/forward 버튼 동작. `Cmd+Opt+I` / `Ctrl+Shift+I` 로 DevTools 가시. `javascript:alert(1)` 입력 시 차단 + status "Blocked: unsafe scheme" 가시.

### MS-3: Bridge + Auto-detect + Persistence integration (RG-WB-4, RG-WB-6, RG-WB-7 잔여)

- **범위**: JS↔Rust bridge 양방향 + trusted_domains 검증 + dev-server URL auto-detect (PTY hook + debouncer + toast) + persistence schema 의 `LeafState::Web { url }` minor extension + crash recovery (state == Crashed 토스트 + reload).
- **포함 요구사항**: REQ-WB-006, REQ-WB-030 ~ REQ-WB-035, REQ-WB-050 ~ REQ-WB-055.
- **시연 가능 상태**: 별도 터미널에서 `python -m http.server 8080` 실행 → 토스트 → 클릭 → 새 탭 webview 가시. JS 콘솔에서 `window.studio.send(...)` → tracing log 가시. Studio 재시작 후 마지막 webview 의 URL 복원.

---

## 9. 파일 레이아웃 (canonical)

### 9.1 신규

- `crates/moai-studio-ui/src/web/mod.rs` — 모듈 진입점.
- `crates/moai-studio-ui/src/web/surface.rs` — `WebViewSurface` struct + impl Render.
- `crates/moai-studio-ui/src/web/backend.rs` — `WebViewBackend` trait.
- `crates/moai-studio-ui/src/web/wry_backend.rs` — `WryBackend` impl.
- `crates/moai-studio-ui/src/web/history.rs` — `NavigationHistory`.
- `crates/moai-studio-ui/src/web/url_detector.rs` — regex match + Debouncer.
- `crates/moai-studio-ui/src/web/bridge.rs` — `BridgeMessage` + `BridgeRouter`.
- `crates/moai-studio-ui/src/web/config.rs` — `WebConfig` (web.yaml 로드).
- `.moai/config/sections/web.yaml` — web 설정 디폴트 (trusted_domains, devtools_enabled, max_concurrent_webviews 등).
- `crates/moai-studio-ui/tests/integration_web.rs` — 통합 테스트 (Spike 0 결과 + USER-DECISION-gpui-test-support 결과에 따른 분기).

### 9.2 수정

- `crates/moai-studio-ui/Cargo.toml` — `wry = "0.50"` (Spike 0 시점 정확한 버전 lock), `regex = "1"` 추가 (URL detector).
- `crates/moai-studio-ui/src/lib.rs` — `pub mod web;` 추가, `RootView` 에 `url_detector` + `pending_toasts` 필드 추가, `Render for RootView` 에 toast_overlay element 추가.
- `crates/moai-studio-ui/src/terminal/mod.rs` — stdout observer hook 메서드 추가 (`add_stdout_observer(callback)`). 공개 API 보존, 신규 메서드만 추가.
- `crates/moai-studio-ui/src/tabs/container.rs` (또는 SPEC-V3-006 의 leaf.rs) — `LeafKind::Web(Entity<WebViewSurface>)` variant 추가 (먼저 머지된 SPEC 의 위치 따름).
- `crates/moai-studio-workspace/src/persistence.rs` — `LeafState` enum 에 `Web { url: String }` variant 추가 (forward-compat). schema version bump 없음 (minor extension).
- `.github/workflows/ci-rust.yml` (Linux job) — `apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev` 추가 (Spike 1 결과 반영).

### 9.3 변경 금지 (FROZEN — REQ-WB-060 ~ REQ-WB-064)

- `crates/moai-studio-terminal/**` 전체 (SPEC-V3-002 carry).
- `crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}` 의 공개 API.
- `crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}` 의 기존 메서드 시그니처. (`LeafKind::Web` enum variant 추가는 enum extension.)
- `crates/moai-studio-workspace/src/persistence.rs` 의 `moai-studio/panes-v1` schema 기존 필드.
- `crates/moai-studio-ui/src/terminal/mod.rs` 의 기존 공개 메서드 시그니처.

---

## 10. Acceptance Criteria

| AC ID | Requirement Group | Milestone | Given | When | Then | 검증 수단 |
|-------|-------------------|-----------|-------|------|------|-----------|
| AC-WB-1 | RG-WB-1 (REQ-WB-001~005) | MS-1 | Spike 0 PASS, USER-DECISION-A/B/C/D 결정 완료, `cargo run -p moai-studio-app` 실행 + 사용자가 새 webview leaf 를 활성 탭에 마운트 | RootView 렌더 시점 | URL bar + 빈 회색 webview 영역 + status bar 가 가시. backend 가 wry 인스턴스 보유. panic 없음. | 수동 smoke test + cargo test (WebViewSurface 단위 + Render 단위) |
| AC-WB-2 | RG-WB-2 (REQ-WB-010~014) | MS-2 | webview leaf 활성, 빈 about:blank 상태 | 사용자가 URL bar 에 `https://example.com` 입력 + Enter | (a) `NavigationHistory.entries.len() == 1`, (b) cursor == 0, (c) backend.navigate("https://example.com") 호출, (d) 페이지 load 후 history.entries[0].title == "Example Domain", (e) RootView re-render | integration test (TestAppContext + WryBackend mock) 또는 logic-level (NavigationHistory unit + backend mock) |
| AC-WB-3 | RG-WB-2 (REQ-WB-015) | MS-2 | webview leaf 활성 | 사용자가 URL bar 에 `javascript:alert(1)` 입력 + Enter | navigate 호출 reject, status bar 에 "Blocked: unsafe scheme" 표시, tracing warn 1 건 기록, history 미변경 | logic-level unit test (URL sanitize/validate 함수) + status bar render 검증 |
| AC-WB-4 | RG-WB-3 (REQ-WB-020~023) | MS-2 | webview leaf 활성, USER-DECISION-B=(a) 채택 (항상 활성) | 사용자가 `Cmd+Opt+I` (macOS) 또는 `Ctrl+Shift+I` (Linux) 입력 | `WebViewSurface.devtools_open == true`, `backend.open_devtools()` 호출, OS-level DevTools 창 가시 | 수동 smoke test (DevTools 창 가시) + logic-level unit (state 전이 + backend mock 호출 검증) |
| AC-WB-5 | RG-WB-4 (REQ-WB-030~033, REQ-WB-035) | MS-3 | TerminalSurface 가 PTY stdout 으로 `Serving HTTP on 0.0.0.0 port 8080` 텍스트 chunk 수신 | url_detector 가 매치 + debouncer 통과 | (a) RootView.pending_toasts 에 ToastEntry 1 개 추가, (b) toast element 우하단 가시, (c) 같은 URL 의 두 번째 chunk 가 5 초 내 도착 시 추가 토스트 없음 (dedupe) | unit test (regex + Debouncer state machine) + integration (mock TerminalSurface stdout + RootView 검증) |
| AC-WB-6 | RG-WB-4 (REQ-WB-033) | MS-3 | RootView 에 dev-server URL 토스트 가시 | 사용자가 토스트 클릭 | TabContainer 에 새 탭 생성, 활성 탭의 leaf 가 `LeafKind::Web(...)` 로 마운트, WebViewSurface.url == 감지된 URL, navigate 호출 | integration test (TabContainer mutation + leaf type 검증) |
| AC-WB-7 | RG-WB-5 (REQ-WB-040~045) | MS-2 | USER-DECISION-A=(a) 채택, workspace_id == "ws-test-1" | 새 WebViewSurface 생성 + navigate("https://example.com") | wry 의 with_data_directory 가 `~/.moai/webview-data/ws-test-1/` 로 설정됨. 시스템 브라우저의 ~/Library/Cookies (macOS) 또는 ~/.local/share/webkit2gtk (Linux) 가 변경되지 않음. | 수동 smoke (디렉터리 생성 검증) + integration (mock 환경에서 storage path 검증) |
| AC-WB-8 | RG-WB-6 (REQ-WB-050~055) | MS-3 | webview 가 `http://localhost:8080/test.html` 로드 (trusted_domain 매치). test.html 안에 `<script>window.studio.send({channel:'log', kind:'event', payload:{message:'hi'}})</script>`. BridgeRouter 에 'log' 채널 등록 | 페이지 load 완료 + JS 실행 | (a) Rust 측 'log' handler 가 1 회 호출, (b) payload.message == "hi", (c) tracing log "info: hi" 출력 | integration test (mock wry IPC handler + BridgeRouter unit + handler 호출 횟수 검증) |
| AC-WB-9 | RG-WB-6 (REQ-WB-053, REQ-WB-055) | MS-3 | webview 가 `https://untrusted.example.com/` 로드 (trusted_domain 비매치). 페이지가 `window.ipc.postMessage('{"channel":"log",...}')` 시도 | 메시지 도착 시점 | (a) BridgeRouter handler 미호출, (b) tracing warn 1 건 ("untrusted origin"), (c) JS 측에 error reply 미전송 (정보 누출 방지) | integration test (mock origin + BridgeRouter spy) |
| AC-WB-10 | RG-WB-7 (REQ-WB-060~064) | 전체 | 본 SPEC 모든 milestone 완료 후 | `cargo test -p moai-studio-terminal` + `cargo test -p moai-studio-workspace` + `cargo test -p moai-studio-ui --lib panes::` + `cargo test -p moai-studio-ui --lib tabs::` 실행 | SPEC-V3-002/003/004 기존 tests 전원 GREEN. terminal crate, panes/tabs 의 기존 unit tests, workspace crate persistence tests 모두 0 regression. `LeafState::Web` variant 추가 후에도 기존 v1 schema 파일 deserialize 호환. | CI gate / cargo test |

---

## 11. 의존성 및 제약

### 11.1 외부 의존성

| Crate | 버전 / 상태 | 비고 |
|-------|-------------|------|
| `gpui` | 0.2.2 (SPEC-V3-001~004 carry) | 변경 없음 |
| `wry` | 0.50.x (Spike 0 시점 lock) | 신규. macOS WKWebView / Linux webkit2gtk-4.1 백엔드 |
| `regex` | 1.x (workspace) | URL detector |
| `serde` / `serde_json` | workspace | bridge payload 직렬화 |
| `tokio` | workspace | async runtime (debouncer 타이머) |
| `tracing` | workspace | 보안 reject 로그 + bridge warn |

### 11.2 시스템 의존성 (Linux)

- `libwebkit2gtk-4.1-dev` + `libsoup-3.0-dev` (`apt install ...`).
- USER-DECISION-Linux 의 결과에 따라 추가 패키지 (4.0 도 지원 시 +`libwebkit2gtk-4.0-dev`).

### 11.3 USER-DECISION 게이트 (4 개)

- **[USER-DECISION-REQUIRED: webview-backend-choice-v3-007]** — MS-1 진입 직전.
  - (a) **(권장)**: wry. production-tested, OS-native. 빌드 단순.
  - (b): servo. Rust-native, 미성숙 (embed API 미공개).
  - (c): tao + 자체 구현. 작업량 거대, 비추.
  - Default: (a). 비채택 시 plan.md 의 T0 fail-fast.

- **[USER-DECISION-REQUIRED: linux-webkit2gtk-version-v3-007]** — MS-1 진입 직전.
  - (a) **(권장)**: 4.1 only. Ubuntu 22.04+ 기본.
  - (b): 4.0+4.1 cfg 분기. 구식 LTS 지원, +50 LOC + CI matrix +1.
  - (c): 4.1 only + 차단 release-note.
  - Default: (a).

- **[USER-DECISION-REQUIRED: devtools-activation-policy-v3-007]** — MS-1 진입 직전.
  - (a) **(권장)**: 항상 활성. 디버깅 편의 + IDE 정체성.
  - (b): debug build only. 보안 우선.
  - (c): config 토글, 디폴트 (a).
  - Default: (a).

- **[USER-DECISION-REQUIRED: webview-sandbox-profile-v3-007]** — MS-1 진입 직전.
  - (a) **(권장)**: per-workspace ephemeral profile. cookie 격리. +50~200MB disk.
  - (b): single global profile. disk 절약, cookie 공유 위험.
  - (c): 매 세션 incognito. OAuth 재로그인 매번 필요.
  - Default: (a).

### 11.4 내부 의존성

- `crates/moai-studio-terminal` (SPEC-V3-002 완료) — 무변경 carry (RG-WB-7).
- `crates/moai-studio-ui::{panes, tabs, terminal}` (SPEC-V3-003 완료) — 공개 API 무변경.
- `crates/moai-studio-ui::tab_container` 의 `LeafKind` (SPEC-V3-004) — `Web` variant 추가.
- `crates/moai-studio-workspace::persistence` (SPEC-V3-003 MS-3) — `LeafState::Web { url }` variant 추가 (forward-compat).
- SPEC-V3-005 (병행, File Explorer) — `OpenFileEvent { surface_hint: Some(Web) }` 핸들러 등록 가능. v1 에서는 mock event 로 검증.
- SPEC-V3-006 (병행, Markdown Viewer) — markdown 본문의 link click → `OpenLinkEvent { url, target: NewTab }` 핸들러. v1 에서는 mock 으로 검증.

### 11.5 시스템/도구 제약

- Rust stable 1.93+ (SPEC-V3-002 carry).
- macOS 14+ + Ubuntu 22.04+. Windows 는 본 SPEC 범위 밖 (SPEC-V3-002/003 carry).
- 기존 `mlugg/setup-zig@v2` CI 스텝 유지 (Terminal Core 링크 carry).
- Spike 0/1/2 모두 PASS 필요.

### 11.6 Git / Branch 제약

- 본 SPEC 구현은 향후 `feature/SPEC-V3-007-webview` 브랜치에서 진행 (현재 산출물은 `feature/SPEC-V3-004-render` 위 docs only commit).
- `main` 직접 커밋 금지 (CLAUDE.local.md §1).
- 각 MS 는 squash 머지를 위한 단일 또는 그룹 커밋으로 분리.

---

## 12. 위험 및 완화

상세 분석은 `.moai/specs/SPEC-V3-007/research.md` §10 참조.

| ID | 위험 | 영향 | 완화 전략 | research 참조 |
|----|------|------|-----------|---------------|
| R-WB1 | wry 와 GPUI 의 native handle handshake 미지원 가능성 | MS-1 차단 | Spike 0 (≤ 4h) — handshake 검증. 실패 시 GPUI 패치 PR 또는 SPEC 보류 + 사용자 통지. | research §10 R1 |
| R-WB2 | Linux webkit2gtk-4.1 CI 빌드 실패 (system 패키지) | CI 실패 | Spike 1 (≤ 1h). ci-rust.yml 에 apt install 한 줄 추가. | research §10 R2 |
| R-WB3 | set_bounds 와 GPUI layout 의 race condition | 화면 깜박임 | deferred update + 프레임당 1 회 호출 제한. | research §10 R3 |
| R-WB4 | OS native webview 의 z-order 충돌 | sidebar/title 가림 | parent-child native subview 관계 — Spike 0 검증. | research §10 R4 |
| R-WB5 | 악성 페이지의 bridge DoS / data 누출 | 사용자 보안 위협 | trusted_domains allowlist + 채널 명시 등록 + 1MB payload 제한. | research §10 R5 |
| R-WB6 | OAuth redirect 의 시스템 브라우저 default handler 충돌 | flow 중단 | navigation_handler 에서 redirect URL 매치 + bridge 로 token 전달 후 webview close. v1 에서는 별도 SPEC 으로 deferred. | research §10 R6 |
| R-WB7 | URL detector regex false positive | 토스트 폭주 | debounce + dedupe + 사용자 dismiss silence 30분. | research §10 R7 |
| R-WB8 | DevTools release 활성 → 보안 검토 결함 | 페이지 데이터 노출 | USER-DECISION-B 결과 + cfg 분기. | research §10 R8 |
| R-WB9 | wry crash 시 recovery 메커니즘 부재 | 사용자 경험 저하 | WebViewState::Crashed + 토스트 reload 옵션. | research §10 R9 |
| R-WB10 | Servo migration 미래 부담 | 기술 부채 | WebViewBackend trait 추상화 (RG-WB-1). | research §10 R11 |
| R-WB11 | wry CVE 패치 누락 | 보안 위협 | dependabot + 보안 review 정책 (1주 내 반영). | research §10 R10 |

---

## 13. 참조 문서

### 13.1 본 레포 내

- `.moai/specs/SPEC-V3-007/research.md` — 본 SPEC 의 코드베이스 분석.
- `.moai/specs/SPEC-V3-007/plan.md` — 본 SPEC 의 task 분해.
- `.moai/specs/SPEC-V3-004/spec.md` §7 — RootView ↔ TabContainer ↔ PaneTree leaf payload generic.
- `.moai/specs/SPEC-V3-006/research.md` §1.3 — `LeafKind` 도입 합의.
- `.moai/specs/SPEC-V3-005/research.md` — `OpenFileEvent` 정의 (file → surface 라우팅 협력).
- `.moai/specs/SPEC-V3-002/spec.md` — Terminal Core 무변경 원칙.
- `.moai/specs/SPEC-V3-003/spec.md` §3 — Pane/Tab logic 의 leaf payload 추상.
- `crates/moai-studio-ui/src/lib.rs:170-202` — RootView impl Render reference.
- `crates/moai-studio-ui/src/terminal/mod.rs` — stdout observer hook 추가 위치.

### 13.2 외부 참조

- [tauri-apps/wry](https://github.com/tauri-apps/wry) — 채택 라이브러리.
- [tauri-apps/wry examples](https://github.com/tauri-apps/wry/tree/dev/examples) — `with_ipc_handler`, `with_navigation_handler`, `evaluate_script` 사용 패턴.
- [WKWebView (Apple Developer)](https://developer.apple.com/documentation/webkit/wkwebview) — macOS 백엔드.
- [WebKitGTK 4.1](https://webkitgtk.org/) — Linux 백엔드.
- [raw-window-handle crate](https://crates.io/crates/raw-window-handle) — GPUI ↔ wry 핸들 추상.

---

## 14. Exclusions (What NOT to Build)

본 SPEC 이 명시적으로 **다루지 않는** 항목 (별도 SPEC 으로 분리):

- E1. **Windows 빌드** — SPEC-V3-002/003 carry. WebView2 backend 는 wry 가 지원하지만 GPUI 0.2.2 가 Windows 미지원이므로 본 SPEC 비대상.
- E2. **iOS / Android** — 모바일 platform 미지원.
- E3. **광고 차단 / 사용자 스크립트 / 확장 API** — 별도 SPEC (SPEC-V3-WEB-EXTENSIONS).
- E4. **인앱 print preview / PDF export** — 별도 SPEC (SPEC-V3-WEB-PRINT).
- E5. **Tab reorder / detach to new window** — SPEC-V3-003 N6/N7 carry.
- E6. **History persistence (탭 close 후 history 복원)** — 본 SPEC v1 은 마지막 URL 만. 전체 history 는 별도 SPEC.
- E7. **OAuth 자동 token 추출 / token store 통합** — bridge 는 도구만 제공. 실제 OAuth 흐름 + token 영속화는 별도 SPEC.
- E8. **WebRTC / 카메라 / 마이크 권한 prompt** — OS webview 디폴트 정책 그대로 사용.
- E9. **사용자 스크립트 주입 (UI 노출)** — `evaluate_script` 는 internal API only.
- E10. **Servo / 자체 webview 구현** — 별도 SPEC (SPEC-V3-WEB-MIGRATE, 장기).
- E11. **새 design token 추가** — 기존 토큰 재사용.
- E12. **Multi-process Studio (webview 별도 process)** — wry 가 OS-level 에서 처리, Studio 본체는 단일 process.
- E13. **Terminal Core 변경** — RG-WB-7 carry.
- E14. **Persistence schema major version bump** — minor extension (`LeafState::Web { url }` variant 추가) 만.

---

## 15. 용어 정의

- **WebView Surface**: moai-studio v3 의 4 대 surface 중 4 번째. 본 SPEC 의 책임 영역.
- **WebViewSurface Entity**: `cx.new(|cx| WebViewSurface::new(url))` 로 만든 `Entity<WebViewSurface>` 인스턴스. PaneTree leaf payload.
- **WebViewBackend trait**: webview 구현체의 추상화. `WryBackend` 가 v1 의 유일 구현체. servo 등 미래 backend 를 위한 swap 지점.
- **wry**: tauri-apps 의 cross-platform webview crate. 본 SPEC 의 채택 backend.
- **Native subview**: GPUI Window 의 OS-level child view 로 webview 를 attach 하는 통합 전략. 이외 off-screen render-to-texture 와 별도 OS window 는 비채택.
- **Bridge**: webview 안의 JS 와 Studio 의 Rust 코드 사이의 양방향 통신 채널. wry IPC handler + evaluate_script 로 구성.
- **Trusted domain**: bridge 가 활성화되는 origin allowlist. 디폴트 `["localhost", "127.0.0.1", "[::1]"]`.
- **Sandbox profile**: webview 의 cookie / localStorage / IndexedDB 가 영속되는 디렉터리. per-workspace 가 디폴트.
- **DevTools**: OS webview 의 개발자 도구 패널. Studio 단축키로 토글.
- **URL Detector**: PTY stdout 텍스트에서 `localhost:NNNN` 패턴 매치하는 regex 모듈.
- **Toast**: Studio 우하단에 표시되는 일시적 알림 element. 본 SPEC 에서는 dev-server URL 감지 통지에 사용.
- **escape hatch**: 본 SPEC 은 4-surface 비전의 4 번째 surface 완성 후 v3 IDE 정체성의 기본 형태 escape hatch.

---

## 16. 열린 결정 사항

| ID | 결정 사항 | Default / 권장 | 결정 시점 |
|----|----------|----------------|----------|
| OD-WB1 | webview backend 선택 | (a) wry (권장) | MS-1 진입 직전 ([USER-DECISION-REQUIRED: webview-backend-choice-v3-007]) |
| OD-WB2 | Linux webkit2gtk 버전 pin | (a) 4.1 only (권장) | MS-1 진입 직전 ([USER-DECISION-REQUIRED: linux-webkit2gtk-version-v3-007]) |
| OD-WB3 | DevTools 활성 정책 | (a) 항상 활성 (권장) | MS-1 진입 직전 ([USER-DECISION-REQUIRED: devtools-activation-policy-v3-007]) |
| OD-WB4 | Sandbox profile 정책 | (a) per-workspace (권장) | MS-1 진입 직전 ([USER-DECISION-REQUIRED: webview-sandbox-profile-v3-007]) |
| OD-WB5 | `LeafKind::Web` enum 도입 위치 (V3-006 vs 본 SPEC) | 먼저 머지되는 SPEC 이 도입 | MS-1 시작 시 (각 SPEC PR 머지 순서로 결정) |
| OD-WB6 | gpui test-support feature 채택 (e2e 테스트 깊이) | (a) 채택 (V3-004/005/006 carry consistency) | MS-1 진입 직전 (선행 SPEC 들의 결정 carry) |
| OD-WB7 | wry 정확한 버전 pin | Spike 0 시점 latest stable (예상 0.50.x) | T0 (Spike 0 결과 반영) |
| OD-WB8 | URL detector regex pattern | research §6.2 의 정규식 그대로 | MS-3 시작 시 |

---

작성: 2026-04-25
브랜치 (산출 위치): `feature/SPEC-V3-004-render`
브랜치 (구현 예정): `feature/SPEC-V3-007-webview`
다음 산출: plan.md (Milestone × Task table)
