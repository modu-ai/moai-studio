# SPEC-V3-007 Research — Embedded Web Browser Surface

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render` (orchestrator branch — 본 SPEC 산출물은 후속에 `feature/SPEC-V3-007-webview` 로 분리됨)
선행: SPEC-V3-001 (셸 레이아웃), SPEC-V3-002 (Terminal Core, 무변경 carry), SPEC-V3-004 (Render Layer Integration — Entity 패턴 차용 + leaf payload generic).
병행: SPEC-V3-005 (File Explorer — `.html` 파일 open-trigger 공급 가능), SPEC-V3-006 (Markdown/Code Viewer — leaf payload sibling).
범위: moai-studio v3 의 4 대 surface 중 하나인 **Embedded Web Browser** 의 도메인 모델, 라이브러리 채택, GPUI 0.2.2 통합 전략, 보안 제약, JS bridge 설계, 그리고 위험 분석.

---

## 1. 동기 — 왜 moai-studio 안에 웹 브라우저를 내장하는가

### 1.1 v3 비전에서의 위치

`.moai/project/product.md` §0.1 ~ §0.4 와 본 SPEC 사용자 프롬프트의 비전 문구는 moai-studio 가 **4 대 surface** 로 구성된 Agentic Coding IDE 임을 명시한다:

| Surface | 책임 | 선행 SPEC |
|---------|------|-----------|
| Terminal | PTY + ANSI render + 키 입력 | SPEC-V3-002 (완료) |
| File Explorer | 파일 트리 + open trigger | SPEC-V3-005 (병행) |
| Markdown / Code Viewer | EARS SPEC, 소스 코드 렌더 | SPEC-V3-006 (병행) |
| **Web Browser (Embedded WebView)** | 앱 내 documentation viewer / dev server preview / OAuth | **본 SPEC (SPEC-V3-007)** |

본 SPEC 이 PASS 하면 moai-studio 가 외부 브라우저로 컨텍스트 스위치 없이 다음 3 가지 핵심 시나리오를 처리한다:

1. **In-app docs viewer**: SPEC 문서의 `https://...` 링크 클릭 시 외부 브라우저 대신 본 SPEC 의 WebView leaf 가 활성 탭에 마운트되어 본문을 렌더.
2. **Dev server preview**: `vite dev` / `next dev` / `cargo run --example` 등 `localhost:NNNN` 을 turminal pane 옆에서 라이브 미리보기 (SPEC-V3-002 의 PtyWorker 출력에서 URL 자동 감지).
3. **OAuth flow**: GitHub / Google OAuth callback URL 을 외부 브라우저 우회 없이 인앱에서 처리, redirect 결과를 Studio 의 token store 로 안전하게 전달 (USER-DECISION-D 게이트).

### 1.2 사용자 가시 정의 (escape hatch)

본 SPEC v1.0.0 이 PASS 한 시점에 `cargo run -p moai-studio-app` 실행 후 사용자가 직접 관찰할 수 있어야 하는 것:

1. SPEC 문서 링크 (예: `https://github.com/zed-industries/zed`) 더블클릭 → 활성 탭의 leaf 가 WebView 로 교체되어 GitHub 페이지가 가시.
2. 주소 표시줄에서 URL 직접 입력 + Enter → navigate.
3. `Cmd+Opt+I` (macOS) / `Ctrl+Shift+I` (Linux) → DevTools 패널 토글 (개발 모드 빌드 한정).
4. 별도 터미널에서 `python -m http.server 8080` 실행 → moai-studio 가 `http://localhost:8080` URL 을 자동 감지하여 사용자에게 "Open in Studio?" 토스트 표시.
5. JS 코드에서 `window.studio.send('hello')` 실행 → Rust 측 `WebViewBridge::on_message` 핸들러가 메시지 수신.
6. JS 가 `localStorage` / cookie 를 사용하더라도 위치는 Studio sandbox profile 로 격리 (시스템 브라우저 cookie 와 분리).

### 1.3 이미 존재하는 자산과의 격차

| 자산 | 위치 | 상태 |
|------|------|------|
| GPUI Render trait + Entity 패턴 | SPEC-V3-001/002/003/004 | ✅ 활용 가능 |
| `RootView::tab_container` leaf payload generic | SPEC-V3-004 RG-R-2 | ✅ `LeafKind::Web(...)` 추가 지점 명확 |
| WebView crate (wry / servo / 자체 구현) | — | ❌ 미존재 — **본 SPEC 이 도입** |
| URL parsing / navigation history | — | ❌ 미존재 |
| Dev server URL auto-detect (PTY 출력 watch) | — | ❌ 미존재 |
| JS ↔ Rust bridge 설계 | — | ❌ 미존재 |
| WebView sandbox / CSP / cookie 격리 | — | ❌ 미존재 |

격차는 5 갈래 — (a) WebView crate 채택, (b) Entity + Render 통합, (c) navigation/history 도메인, (d) PTY URL 자동 감지, (e) JS bridge + 보안.

---

## 2. WebView 라이브러리 분석

### 2.1 후보 비교

| 후보 | 라이센스 | 플랫폼 | 백엔드 | 활성도 | 평가 |
|------|----------|--------|--------|--------|------|
| **`wry`** (tauri-apps) | Apache-2.0 / MIT | macOS / Linux / Windows / iOS / Android | WKWebView (macOS) / webkit2gtk (Linux) / WebView2 (Windows) | crates.io 100k+ DL/월, Tauri 의 핵심 의존 | **권장 — production-tested** |
| `servo` | MPL-2.0 | Linux + macOS (실험) | Rust-native (Stylo + WebRender) | Mozilla 부활 프로젝트, 미성숙 | embed API 미공개, 단일 페이지 렌더 가능 수준 |
| `tao` + 자체 구현 | Apache-2.0 / MIT | tao 는 윈도우 라이브러리만 | — | tao 는 wry 의 deps | 자체 webview 구현은 OS API 직접 호출 — 작업량 거대 |
| `webview-rs` (zserge) | MIT | macOS / Linux / Windows | OS native | 유지보수 정체 | 활성도 낮음, wry 가 더 광범위 |
| `cef` (Chromium Embedded) | BSD | 전 플랫폼 | Chromium 풀 임베드 | crate `cef-rs` 미성숙 | 바이너리 크기 +200MB, 라이센스 복잡, 본 SPEC 비추천 |

### 2.2 wry 채택 근거

- **Tauri 가 production 에서 사용**: 100k+ 앱이 wry 위에 떠 있음, 즉 macOS/Linux/Windows 의 OS-level webview 통합은 충분히 검증됨.
- **OS native 백엔드**: macOS 는 WKWebView (Safari 엔진), Linux 는 webkit2gtk-4.1, Windows 는 Edge WebView2. 각 OS 의 시스템 업데이트와 함께 보안 패치가 자동 반영.
- **바이너리 크기**: CEF 와 달리 OS 가 제공하는 webview 를 사용하므로 모아이 스튜디오 binary 크기 증가는 ~5 MB (wry crate + tao 종속).
- **GPUI 호환성**: wry 의 `WebView::new_as_child` 가 native window handle 을 받는다. GPUI Window 가 `raw-window-handle` trait 을 구현하므로 통합 가능 (Spike 0 으로 검증).
- **단점**: Windows GA 의 GPUI 0.2.2 미준비 → 본 SPEC v1 은 macOS/Linux 만 (SPEC-V3-002/003 carry).

### 2.3 wry 버전 결정

- 2026-04 시점 wry 최신 안정: `0.50.x` (workspace 검토 필요, Spike 0 시 정확한 버전 lock).
- `tao` 의 GPUI 의존성 충돌 가능성: GPUI 0.2.2 가 winit 가 아닌 자체 windowing 을 사용하므로 tao 와의 충돌은 없음. 단, **wry::WebViewBuilder::new_as_child(parent_handle)** 사용 시 GPUI Window 가 native handle 을 노출해야 한다 (Spike 0 검증 항목).

### 2.4 servo 미채택 사유

- 2026-04 현재 servo 는 embed API 가 공개되지 않았다 (`servoshell` 단독 데모만 가능).
- Rust-native 의 매력은 크나, "production-tested" 기준 미달.
- 장기 후보로 두되 본 SPEC v1 은 wry. 미래 SPEC (예: SPEC-V4-WEB-MIGRATE) 에서 swap 가능하도록 `WebViewBackend` trait 을 도입한다 (RG-WB-1).

### 2.5 Linux webkit2gtk 버전 pin

[USER-DECISION-REQUIRED: linux-webkit2gtk-version-v3-007]

webkit2gtk 의 ABI 가 GTK 메이저 버전과 묶여 있으며, 다음 두 변형이 있다:

- `webkit2gtk-4.1` (GTK 4 / libsoup 3) — Ubuntu 22.04+, Fedora 38+ 기본
- `webkit2gtk-4.0` (GTK 3 / libsoup 2) — 구식 LTS (Ubuntu 20.04)

옵션 (a) **권장**: `webkit2gtk-4.1` 만 지원. Ubuntu 22.04+ (CI 매트릭스 기본). 빌드 의존성: `libwebkit2gtk-4.1-dev`.
옵션 (b): `webkit2gtk-4.0` + `webkit2gtk-4.1` 둘 다 cfg 분기. 구식 배포판 사용자 지원 +1, 빌드 의존성 분기 LOC +50, CI 매트릭스 추가 1 job.
옵션 (c): `webkit2gtk-4.1` 만, 구식 배포판은 release-note 에서 명시 차단.

Default: (a). Spike 0 에서 빌드 검증.

### 2.6 macOS 코드 서명 / Entitlements

- WKWebView 는 추가 entitlement 불필요 (sandbox 옵션은 별도). 단 dev-server preview 시 `localhost` 접근에 `com.apple.security.network.client` 가 필요할 수 있음 — Spike 0 에서 확인.
- CSP 위반 / mixed content 처리는 wry 의 `WebViewBuilder::with_navigation_handler` 콜백에서 차단 (RG-WB-5).

---

## 3. GPUI 0.2.2 와 wry 의 통합 전략

### 3.1 통합 토폴로지

GPUI 는 자체 GPU-accelerated rendering 을 사용하고, wry 는 OS native webview window/view 를 child 로 임베드한다. 두 시스템이 한 화면에 공존하려면 **레이어 합성** 전략을 결정해야 한다.

| 전략 | 동작 | 호환성 | 권장 |
|------|------|--------|------|
| (A) Native subview | GPUI Window 의 native handle 을 wry::WebView 의 부모로 사용. wry 가 OS-level 합성을 책임. | macOS WKWebView ✅ / GTK 부모 ChildView ✅ | **권장** |
| (B) Off-screen render → texture | wry 가 off-screen 으로 렌더 → bitmap 을 GPUI texture 로 매 frame upload | 60fps 어려움, wry off-screen 모드 부재 | 비추 |
| (C) 별도 OS window | wry::WebView 가 별도 child window 로 floating | UX 저하 (탭 기반 layout 부적합) | 비추 |

채택: (A) Native subview. 단 Spike 0 으로 macOS 에서 GPUI 의 native handle 노출 + WKWebView attach 가능한지 확인.

### 3.2 GPUI Render trait 연계

SPEC-V3-004 의 `LeafKind` 는 `Render + 'static` 만 요구한다. 본 SPEC 은 다음과 같이 `WebViewSurface` Entity 를 정의:

```text
pub struct WebViewSurface {
    pub url: String,                      // 현재 URL
    pub history: NavigationHistory,       // back/forward stack
    pub state: WebViewState,              // Loading / Ready / Error / Crashed
    pub webview_handle: Option<wry::WebView>,
    pub bridge: Option<Arc<WebViewBridge>>,
    pub bounds: Option<gpui::Bounds<f32>>,    // Render::render 에서 갱신
    pub devtools_open: bool,
}

impl Render for WebViewSurface {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 1. 빈 placeholder div 를 GPUI 에서 그리고 (URL bar + status bar)
        // 2. div 의 layout bounds 가 결정되면 wry::WebView::set_bounds 로 동기화
        // 3. wry 가 OS-level 에서 그 영역에 webview 를 그린다
    }
}
```

핵심 관건은 `set_bounds` 호출 시점 — GPUI 의 layout 페이즈가 끝난 후 `Window::on_layout_complete` 등의 hook 을 활용. Spike 0 검증.

### 3.3 leaf payload 통합 (SPEC-V3-004 RG-R-2 carry)

SPEC-V3-006 가 도입할 `LeafKind` enum 에 본 SPEC 이 새 variant 추가:

```text
pub enum LeafKind {
    Terminal(Entity<TerminalSurface>),
    Markdown(Entity<MarkdownViewer>),     // SPEC-V3-006
    Code(Entity<CodeViewer>),             // SPEC-V3-006
    Web(Entity<WebViewSurface>),          // ← 본 SPEC
    Empty,
}
```

SPEC-V3-004 의 `render_pane_tree<L: Render + 'static>` 는 L 만 보면 되므로 본 추가는 비파괴적이다. 단 SPEC-V3-006 가 `LeafKind` 를 먼저 도입할 가능성이 있으므로, **본 SPEC 의 `Web(...)` variant 추가 시점은 SPEC-V3-006 의 PR 머지 후** — 충돌 시 본 SPEC 이 양보한다 (research §11.3).

### 3.4 키보드 포커스 모델

WebView 가 키 입력을 받을 때:
- 사용자가 webview 영역 클릭 → focus enter → 키 이벤트가 wry → 내부 webcontent 로 전달 (TextInput / form 등).
- `Cmd+Opt+I` (DevTools) / `Cmd+L` (URL bar) / `Cmd+R` (reload) 같은 Studio 단축키는 webview 가 아닌 RootView level 에서 가로채야 한다 (REQ-WB-031).
- 단축키 우선순위 정책: SPEC-V3-003 의 `dispatch_tab_key` 와 동일한 패턴 — RootView 가 `Window::on_key_down` 에서 먼저 가로채고 Studio 단축키만 소비, 나머지는 활성 leaf (webview 포함) 로 forward.

---

## 4. Navigation + History 도메인 설계

### 4.1 NavigationHistory 자료구조

```text
pub struct NavigationHistory {
    entries: Vec<HistoryEntry>,   // 최대 100 항목 (RG-WB-2 NFR)
    cursor: usize,                // 현재 위치 (back/forward 기준점)
}

pub struct HistoryEntry {
    pub url: String,
    pub title: Option<String>,    // <title> 태그 결과
    pub visited_at: SystemTime,
}
```

- `navigate(url)` → cursor 이후 entries 절단 + 새 entry 추가 + cursor = entries.len() - 1.
- `back()` → cursor > 0 면 cursor -= 1 + url 복원.
- `forward()` → cursor < entries.len() - 1 면 cursor += 1.
- `clear_history()` → entries.clear(), cursor = 0.

### 4.2 wry 의 자체 history vs 본 SPEC 의 history

wry::WebView 는 내부적으로 OS webview 의 history 를 사용 (`WebView::evaluate_script("history.back()")`). 그러나 Studio 가 자체 NavigationHistory 를 보유해야 하는 이유:

- 사용자가 새 URL 을 직접 입력해도 history 가 일관되게 추적됨.
- 탭 영속화 (SPEC-V3-003 persistence) 시 마지막 URL 만이 아니라 history snippet 도 복원 가능 (본 SPEC 비목표 N4 — 마지막 URL 만 저장).
- wry 의 history API 는 OS 별 차이가 있을 수 있어, 자체 추상화가 안전.

### 4.3 URL 검증 / sanitize

- 사용자가 `localhost:8080` 입력 → `http://localhost:8080` 로 자동 prefix.
- `file://` scheme 허용 여부: USER-DECISION-A 와 묶음 — 기본 허용, but 워크스페이스 root 외부 file 접근은 차단 (보안).
- `javascript:` scheme 차단 (XSS 방지, wry navigation handler 에서 reject).
- `chrome:`, `about:` 등 비표준 scheme 차단 (RG-WB-5).

---

## 5. DevTools 토글 정책

### 5.1 wry 의 DevTools API

- `WebViewBuilder::with_devtools(true)` 로 빌드 시 활성화.
- `WebView::open_devtools()` / `close_devtools()` / `is_devtools_open()` 런타임 토글.
- macOS: WKWebView 의 Web Inspector 별도 창 또는 inline 패널.
- Linux webkit2gtk: GTK Web Inspector 별도 창.
- Windows WebView2: Edge DevTools — 본 SPEC v1 에선 Windows 비대상.

### 5.2 활성 정책

[USER-DECISION-REQUIRED: devtools-activation-policy-v3-007]

옵션 (a) **권장**: `Cmd+Opt+I` (macOS) / `Ctrl+Shift+I` (Linux) 단축키로 항상 활성. 디버깅 편의 우선.
옵션 (b): debug build 에서만 단축키 동작, release build 에서는 의도적 차단 (보안 강화).
옵션 (c): 사용자 설정 (`config.web.devtools_enabled: bool`) 으로 토글, 디폴트 true (debug) / false (release).

Default: (a). 디버깅 편의 + Studio 자체가 개발자용 IDE 라는 정체성.

### 5.3 보안 영향

- DevTools 가 열린 상태에서는 JS 가 임의 코드를 실행할 수 있다 (페이지 자체의 권한 안에서). Studio 의 token store 등 sensitive data 가 webview 에 노출되지 않도록 RG-WB-5 의 sandboxing 으로 격리.

---

## 6. Dev Server Auto-Detect 설계

### 6.1 아이디어

사용자가 터미널 pane 에서 `vite dev` 실행 → stdout 에 `Local: http://localhost:5173/` 출력. Studio 가 이 출력을 watch 하여 URL 추출 → 사용자에게 토스트 알림 → 클릭 시 새 탭 / 새 leaf 로 webview 마운트.

### 6.2 PTY 출력 watch 메커니즘

- SPEC-V3-002 의 `PtyWorker` 는 stdout 을 GPUI 에 ANSI-stripped text 로 전달한다.
- 본 SPEC 은 **read-only observer** 로 PtyWorker 의 출력 stream 을 구독 (WatchHandle 패턴).
- regex `r"https?://(localhost|127\.0\.0\.1|\[::1\]):(\d+)"` 로 URL 매치.
- 매치 발생 후 5 초 debounce 윈도우 — 같은 URL 이 반복 출력되어도 토스트 1 회만.

### 6.3 SPEC-V3-002 의 변경 금지 zone 회피

- SPEC-V3-002 RG-P-7 carry: `crates/moai-studio-terminal/**` 무변경.
- 본 SPEC 은 **observer 등록 hook** 만 PtyWorker 측에 추가하지 않고, GPUI 측 `TerminalSurface` (SPEC-V3-002 외부) 에서 stdout text 가 갱신될 때 본 SPEC 이 등록한 콜백을 호출.
- 즉, 변경 대상은 `crates/moai-studio-ui/src/terminal/` (SPEC-V3-002 의 wrapper 모듈, SPEC-V3-002 의 변경 금지 zone 외부) 의 적은 메서드 추가 — 또는 `crates/moai-studio-ui/src/web/url_detector.rs` 신규 + 외부에서 stdout 을 받아 처리.

USER-DECISION 또는 design choice: 본 SPEC plan 단계에서 SPEC-V3-002 의 RG-P-7 정책이 `crates/moai-studio-terminal/**` 에만 적용된다는 점을 검증 후 (`grep -r "RG-P-7" .moai/specs/SPEC-V3-002/`), `crates/moai-studio-ui/src/terminal/` 는 자유롭게 확장.

### 6.4 토스트 vs 자동 오픈

옵션 (a) 토스트 알림 + 사용자 클릭으로 오픈 (권장 — 사용자 의도 존중).
옵션 (b) 자동으로 새 탭 오픈 (DX 우선, but 의도하지 않은 페이지 오픈 우려).

본 SPEC v1 은 (a). config 로 (b) 를 활성화 가능 (향후 SPEC).

---

## 7. JS ↔ Rust Bridge 설계

### 7.1 bridge 모델

wry::WebView 는 다음 두 방식의 bridge 를 제공:

1. `WebViewBuilder::with_ipc_handler(callback)` — JS 가 `window.ipc.postMessage(string)` 호출 시 callback 발화. 단방향 (JS → Rust).
2. `WebView::evaluate_script(script_string)` — Rust 에서 임의 JS 실행. 단방향 (Rust → JS).

본 SPEC 은 두 방식을 조합하여 양방향 bridge 를 구성:

```text
[JS]                          [Rust]
window.studio.send(payload)
  → window.ipc.postMessage(JSON.stringify(payload))
                              ipc_handler(string) parse → BridgeMessage
                              dispatch to subscriber
                              response = { reply_to, payload }
                              evaluate_script(`window.studio._reply(...)`)
window.studio.on('event', cb)
  ← _reply 콜백 호출
```

### 7.2 BridgeMessage 스키마

```text
{
  "id": "uuid-v4",            // 요청-응답 매칭용
  "kind": "request" | "event",
  "channel": "string",        // 라우팅 키 (e.g., "log", "open-link", "auth-callback")
  "payload": <serde_json::Value>
}
```

- `request` 는 Rust 가 응답을 evaluate_script 로 보냄 (`_reply(id, result)` 형태).
- `event` 는 응답 없음 (fire-and-forget).

### 7.3 보안 — bridge 권한

- 모든 채널은 **명시 허용 리스트** 기반: `BridgeRouter::register("channel-name", handler)`.
- 미등록 채널 → 메시지 무시 + tracing warn (DoS 방어).
- payload 크기 제한: 1 MB / 메시지 (config 가능).
- channel 명 정규화: 알파벳 + 하이픈 + 숫자만 허용.

### 7.4 신뢰 도메인 분리

- bridge 는 **항상 활성**이 아니라 사용자가 명시적으로 활성화한 webview 에서만 동작.
- 활성화 트리거:
  - 옵션 (a) 모든 webview 에서 자동 활성 (편의 우선).
  - 옵션 (b) 워크스페이스 트러스트 도메인 리스트 (`config.web.trusted_domains`) 에 매치되는 URL 만 (보안 우선, **권장**).
  - 옵션 (c) URL 별 prompt — 사용자가 "Allow bridge?" 선택.

본 SPEC v1 은 (b) — 디폴트 trusted = `["localhost", "127.0.0.1", "[::1]"]` (dev server). 외부 도메인은 사용자가 추가.

---

## 8. 보안 — Sandboxing + CSP + Same-Origin

### 8.1 OS-level sandbox 활용

- WKWebView (macOS): App Sandbox 환경에서 동작. wry 의 빌드 옵션으로 `incognito mode` 활성화 시 cookie/localStorage 가 시스템 기본 storage 와 분리.
- webkit2gtk (Linux): `WebsiteDataManager` 로 ephemeral profile 생성, 디스크 storage 분리.
- 본 SPEC 은 **per-Studio profile 디렉터리** 사용: `~/.moai/webview-data/<workspace-id>/` (USER-DECISION-A 의 sandbox 정책 결정에 따라).

### 8.2 CSP 정책

- 신규 페이지 navigate 전: `with_navigation_handler` 가 호출되어 URL 검증.
- `javascript:`, `data:` (text/html), `chrome:` scheme reject.
- `file://` scheme: 워크스페이스 루트 prefix 만 허용 (USER-DECISION-A).
- mixed content (https 페이지의 http 리소스): reject + tracing warn.

### 8.3 same-origin 정책

- 기본은 OS webview 의 same-origin 정책 그대로 사용.
- bridge 는 page 의 origin 과 무관 (Studio 가 controlled). 단 RG-WB-5 의 trusted_domains 정책으로 활성화 범위 제한.

### 8.4 USER-DECISION-A — sandbox profile 정책

[USER-DECISION-REQUIRED: webview-sandbox-profile-v3-007]

옵션 (a) **권장**: per-workspace ephemeral profile (`~/.moai/webview-data/<workspace-id>/`). 워크스페이스 별 cookie 격리. 디스크 사용량 +50~200MB.
옵션 (b): single global profile (`~/.moai/webview-data/global/`). 디스크 절약, 단 워크스페이스 간 cookie 공유 위험.
옵션 (c): 매 세션 incognito (no persistent storage). cookie 영속화 없음 → OAuth 재로그인 필수.

Default: (a). 설계 단순성 + 보안 + workspace persistence 정합.

---

## 9. WebView 의 라이프사이클

### 9.1 생성 / 파괴

- WebViewSurface Entity 생성 시 wry::WebView 도 동시에 생성.
- Entity drop 시 wry::WebView 도 drop → OS-level 자원 해제.
- 탭 close → leaf drop → WebViewSurface drop chain → wry drop.

### 9.2 메모리 압력 관리

- 한 webview 가 ~50-200 MB RAM 점유 (페이지 콘텐츠에 따라).
- 다중 webview tab 시 전체 메모리 스파이크 가능.
- 본 SPEC v1: tab 당 1 webview, 최대 동시 10 개 webview tab (config). 초과 시 가장 오래된 unfocused webview suspend (페이지 unload + URL/history 보존 → focus 복귀 시 reload).

### 9.3 crash recovery

- WKWebView 가 crash 하면 별도 시스템 프로세스가 SIGCHLD → wry 가 detect → `WebViewState::Crashed` 로 전이.
- Studio 는 "Page crashed. Reload?" 토스트 표시 → 사용자 클릭 시 reload.
- bridge 등록은 wry::WebView 새 인스턴스 생성 시 재등록.

---

## 10. 위험 분석 및 완화

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R1 | wry 의 GPUI 통합 — native handle 노출 미지원 가능성 | MS-1 차단 | Spike 0 (≤ 4h) — GPUI Window::native_handle() 노출 검증, 미지원 시 GPUI 패치 검토 또는 SPEC 보류 |
| R2 | webkit2gtk-4.1 빌드 의존성 (Linux CI runner 에 system 패키지 필요) | CI 빌드 실패 | Spike 0 의 Linux job — `apt install libwebkit2gtk-4.1-dev` 로 해결, ci-rust.yml 에 한 줄 추가 |
| R3 | wry::WebView::set_bounds 와 GPUI layout 의 동기화 race | 화면 깜박임 / 잘못된 위치에 webview | 프레임당 한 번만 set_bounds, GPUI layout 후 deferred update |
| R4 | OS native webview window 가 GPUI Window 위로 z-order 충돌 | webview 가 Studio sidebar/title bar 가림 | 부모-자식 native subview 관계로 OS 가 자동 처리 — Spike 0 검증 |
| R5 | bridge 의 인증되지 않은 메시지 (악성 페이지의 `window.ipc.postMessage` 호출) | DoS / sensitive data 누출 | trusted_domains allowlist + 채널 명시 등록 + payload 크기 제한 |
| R6 | OAuth flow 의 redirect URL 캡처 — 시스템 브라우저 default handler 충돌 | flow 완성 못 함 | navigation_handler 에서 redirect URL 패턴 매치 → bridge 로 token 전달 후 webview close |
| R7 | dev-server URL detect 정규식의 false positive (예: docstring 안의 URL) | 의도하지 않은 토스트 폭주 | debounce + recently-detected dedupe + 사용자 차단 옵션 |
| R8 | DevTools 가 release build 에서도 활성 → 보안 검토 결함 | 사용자 페이지 데이터 노출 | USER-DECISION-B 결과에 따라 release 차단 옵션, 명시적 settings UI |
| R9 | DevTools / webview window 가 Studio process 와 별도 OS process 일 시 IPC 비용 | 60fps 렌더 영향 | 구조적으로 wry 가 native — 추가 비용 없음 |
| R10 | wry crate 의 LICENSE (MIT/Apache-2.0) — moai-studio MIT 와 호환 | 라이센스 충돌 | wry 는 dual-license, MIT 채택으로 호환. cargo deny check 로 자동 검증 |
| R11 | Servo / wry / native 중 어떤 것을 future-proof 로 둘지 미결 | 장기 stratagy 부재 | `WebViewBackend` trait 추상화 도입 (RG-WB-1) — 추후 backend swap 가능 |

---

## 11. 의존 SPEC 및 경계

### 11.1 SPEC-V3-004 (Render Layer) 의존

- `LeafKind` 가 SPEC-V3-006 시점에 도입되거나 본 SPEC 시점에 도입되거나 — 둘 중 먼저 머지되는 SPEC 이 enum 정의 책임을 진다.
- 본 SPEC 은 SPEC-V3-006 보다 늦게 머지된다고 가정 (depends_on: [SPEC-V3-004], parallel_with: [SPEC-V3-005, SPEC-V3-006]).
- SPEC-V3-006 의 머지 시점에 `LeafKind::Web` variant 가 미존재하면 본 SPEC 이 추가.

### 11.2 SPEC-V3-005 (File Explorer) 와의 협력

- File Explorer 에서 `.html` 파일 더블클릭 → `OpenFileEvent { surface_hint: Some(Web) }` 발행.
- 본 SPEC 의 `WebViewSurface::open_file(path)` 가 핸들러로 등록.
- file:// URL 변환은 `path::canonicalize` + scheme prefix.
- 본 SPEC v1 에서는 SPEC-V3-005 의 머지 후 hooked. 미머지 상태에서는 logic-level mock event 로 통합 검증.

### 11.3 SPEC-V3-006 (Markdown Viewer) 와의 협력

- Markdown 본문에서 `https://...` 링크 클릭 → `OpenLinkEvent { url, target: NewTab }` 발행.
- 본 SPEC 의 dispatch handler 가 새 tab leaf 로 WebViewSurface 마운트.
- v1 에서 markdown 의 link click handler 가 동일 RootView 의 활성 탭으로 dispatch — SPEC-V3-006 와 합의 필요 (research §11.5).

### 11.4 SPEC-V3-002 (Terminal Core) 무변경 carry

- 본 SPEC 은 `crates/moai-studio-terminal/**` 를 변경하지 않는다 (RG-WB-7).
- PTY URL 자동 감지는 `crates/moai-studio-ui/src/terminal/` (terminal wrapper) 의 stdout 텍스트 hook 으로 구현. SPEC-V3-002 의 본체는 무관.

### 11.5 SPEC-V3-003 (Pane/Tab) 무변경 carry

- 본 SPEC 은 tabs/panes 의 logic API 를 변경하지 않는다 (RG-WB-7).
- `LeafKind::Web` 추가만 (enum extension).

### 11.6 향후 SPEC 후보

- **SPEC-V3-WEB-MIGRATE**: wry → servo migration (장기, servo embed API 안정화 후).
- **SPEC-V3-WEB-EXTENSIONS**: WebView 확장 API (광고 차단, 사용자 스크립트 등). 본 SPEC v1 비목표 (E10).
- **SPEC-V3-WEB-PRINT**: 인앱 print preview / PDF export. 본 SPEC v1 비목표 (E11).

---

## 12. 변경 금지 zone

- **`crates/moai-studio-terminal/**`**: 무변경 (SPEC-V3-002 carry, RG-WB-7).
- **`crates/moai-studio-ui/src/panes/{tree.rs, constraints.rs, focus.rs, splitter.rs, divider.rs}`**: 공개 API 무변경 (SPEC-V3-003 carry).
- **`crates/moai-studio-ui/src/tabs/{container.rs, bar.rs, keys.rs}`**: 공개 API 무변경. `LeafKind::Web` variant 추가는 enum 확장만 (signature 무변경).
- **`crates/moai-studio-workspace/src/persistence.rs`**: SPEC-V3-003 MS-3 산출 schema 무변경. 본 SPEC 은 last URL 만 read/write 추가 (lifecycle: persistence schema 의 minor extension — `LeafState::Web { url }` variant 추가).
- **`crates/moai-studio-ui/src/terminal/mod.rs`**: 공개 API 보존, observer hook 추가만 허용.

---

## 13. USER-DECISION 게이트 요약

본 SPEC 진입 시 사용자 결정이 필요한 게이트 4 개:

| 게이트 ID | 결정 사항 | Default | 결정 시점 |
|-----------|----------|---------|----------|
| webview-backend-choice-v3-007 | wry / servo / 자체 구현 | wry (권장) | T0 (MS-1 진입 직전) |
| linux-webkit2gtk-version-v3-007 | 4.1 only / 4.0+4.1 / 4.1 + 차단 | 4.1 only (a) | T0 |
| devtools-activation-policy-v3-007 | 항상 / debug-only / config | 항상 (a) | T0 |
| webview-sandbox-profile-v3-007 | per-workspace / global / incognito | per-workspace (a) | T0 |

추가 옵션:
- gpui test-support adoption (SPEC-V3-004/005/006 carry, 본 SPEC 의 e2e 테스트에 영향).

---

## 14. Spike 계획

### 14.1 Spike 0 — wry + GPUI 통합 가능성 (≤ 4h)

- 목표: GPUI Window 의 native handle 을 wry::WebView 의 부모로 attach 하는 minimal example 빌드.
- macOS: `cocoa::base::id` 또는 `raw-window-handle` trait 으로 view ptr 추출. Linux: `gdk::Window` raw handle.
- 검증: 빈 GPUI 윈도우에 wry::WebView 가 자식으로 navigate("https://example.com") + 가시.
- 실패 시: GPUI 측 PR 제안 또는 본 SPEC 보류 + plan.md 의 T0 fail-fast 보고.

### 14.2 Spike 1 — webkit2gtk-4.1 CI 빌드 (≤ 1h)

- ci-rust.yml 의 ubuntu-22.04 job 에 `apt install libwebkit2gtk-4.1-dev` 추가 후 `cargo build -p moai-studio-ui --features web` 통과 검증.
- 실패 시: USER-DECISION-Linux-pin 의 옵션 (b) (cfg 분기) 로 fallback.

### 14.3 Spike 2 — bridge round-trip (≤ 2h)

- wry::WebViewBuilder::with_ipc_handler 등록 + JS 에서 `window.ipc.postMessage("ping")` → Rust 수신 + `evaluate_script("console.log('pong')")` 응답 round-trip 검증.
- 실패 시: bridge 설계 재검토.

---

## 15. 영문 보조 요약

This research analyzes the gap between moai-studio v3's existing assets and the Embedded Web Browser surface. wry (tauri-apps) is selected as the primary backend due to production maturity (Tauri uses it), OS-native rendering (WKWebView/webkit2gtk-4.1), and cross-platform parity. Servo is rejected for v1 due to absent embed API. The integration with GPUI 0.2.2 uses a native subview strategy: GPUI manages chrome (URL bar, status), and wry's OS webview is positioned as a child via `set_bounds`. Four USER-DECISION gates are surfaced: backend choice, Linux webkit version pin, DevTools policy, and sandbox profile. Three Spikes (GPUI+wry handshake, Linux CI build, bridge round-trip) gate the Run phase. The bidirectional JS-Rust bridge uses wry's `with_ipc_handler` (JS→Rust) plus `evaluate_script` (Rust→JS) with channel allowlist + payload size limits + trusted-domain enforcement. Dev-server URL auto-detect taps PTY stdout from `crates/moai-studio-ui/src/terminal/` (not SPEC-V3-002's frozen core), with regex matching + 5s debounce + user toast (no auto-open). SPEC-V3-002/003 frozen zones are honored: `LeafKind::Web` is the sole new enum variant; persistence schema gains a minor `LeafState::Web { url }` variant for last-URL restore. 11 risks are catalogued with mitigations, principally focused on the wry-GPUI handshake (Spike 0).

---

작성 완료: 2026-04-25
다음 산출: spec.md (EARS 요구사항 + AC), plan.md (milestone × task).
