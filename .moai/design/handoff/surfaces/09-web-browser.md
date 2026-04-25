# Web Browser Surface

---
title: Embedded Web Browser
version: 1.0.0
source: SPEC-V3-007
last_updated: 2026-04-25
---

## 개요

**Web Browser** 는 내장 웹 브라우저다. URL bar, navigation, DevTools toggle, dev server auto-detect 를 지원.

상태: 설계 완료 (구현 미완료, SPEC-V3-007)

---

## 레이아웃

```
┌──────────────────────────────────────────────────────┐
│ [◀] [▶] [↻] [🏠] [URL bar] [DevTools ⌘D]          │
├──────────────────────────────────────────────────────┤
│                                                        │
│ [Web content rendered here]                           │
│                                                        │
├──────────────────────────────────────────────────────┤
│ [DevTools panel — 선택사항]                         │
│ Console | Elements | Network | Application           │
└──────────────────────────────────────────────────────┘
```

---

## Toolbar

### 네비게이션 버튼

| 버튼 | 동작 | 단축키 |
|------|------|--------|
| ◀ (Back) | 이전 페이지 | Cmd+[ |
| ▶ (Forward) | 다음 페이지 | Cmd+] |
| ↻ (Reload) | 새로고침 | Cmd+R |
| 🏠 (Home) | 기본 페이지 | Cmd+Home |

- 비활성: 회색, disabled cursor
- 호버: 배경 highlight (neutral.750)

### URL Bar

```
┌──────────────────────────────────────────────────┐
│ https://example.com/path                         │
│ ↓ [History dropdown, fuzzy search]               │
│                                                   │
│ · Recent:                                        │
│   - localhost:3000                               │
│   - https://docs.moai.dev                        │
│   - (10 more)                                    │
│                                                   │
│ [Go] [Copy] [Share]                             │
└──────────────────────────────────────────────────┘
```

- Focus: 전체 URL highlight (Cmd+L)
- History: 클릭 or arrow up 로 접근
- Fuzzy search: 입력 중 matching URLs
- Go button: Enter 와 동등

### 스타일

- 높이: 40px
- Font: 14px (base)
- 배경: neutral.800 (dark)
- Border: 1px primary.500 (focused), neutral.700 (blurred)
- Radius: 6px (md)

---

## Security Indicator (URL bar 우측)

### HTTPS Locked

```
🔒 Secure  https://example.com
```

- 색: success.green
- 클릭: certificate info

### HTTP Warning

```
⚠️ Not Secure  http://example.com
```

- 색: warning.yellow
- 클릭: security info

### Blocked Resource

```
🚫 CSP violation  example.com
```

- 색: error.red

---

## Web View Canvas

### 기술

- macOS: WKWebView
- Linux: WebKit2GTK
- Windows: WebView2

### 랜더링

- 전체 screen 크기 (resize 가능)
- 스크롤: 부드러운 (200ms easeOut)
- Zoom: Cmd++ / Cmd+- (25~200%)
- Focus: GPUI surface 와 동일 focus ring

---

## DevTools (선택사항)

### Toggle Button

```
[⌘D] DevTools
```

- 위치: URL bar 우측
- 클릭: DevTools 패널 toggle

### DevTools Panel (하단 split)

```
┌──────────────────────────────────────┐
│ Console | Elements | Network | App   │
├──────────────────────────────────────┤
│ > console.log('hello')                │
│ hello                                 │
│ undefined                             │
└──────────────────────────────────────┘
```

- Height: 기본 200px, drag-resize (min 100px)
- 탭: console, elements, network, application
- 색: neutral.800 bg, neutral.50 text

---

## Dev Server Auto-Detect

### Banner (자동)

```
┌──────────────────────────────────────┐
│ 🚀 localhost:3000 감지됨             │
│ Vite / Next.js / webpack 개발 서버   │
│                                       │
│ [Open] [Dismiss] [Never ask again]   │
└──────────────────────────────────────┘
```

- 감지: localhost 포트 자동 스캔 (3000, 5173, 8080, etc.)
- 배경: secondary.500 @ 20% (subtle)
- 위치: toolbar 하단, 자동으로 닫힘 (3초, 또는 click)

---

## 상태

### Loading

```
[◀ disabled] [▶ disabled] [↻] [🏠] [URL bar]

⟳ Loading...

(blank canvas, spinner)
```

### Loaded

```
[active navigation controls]

[Web content]
```

### Error

```
⚠️ Failed to load page

Error: Connection refused
(localhost:3000)

[Retry] [Change URL]
```

---

## 인터랙션

### Keyboard

| 단축키 | 동작 |
|--------|------|
| Cmd+L | URL bar focus & select all |
| Cmd+R | Reload |
| Cmd+[ | Back |
| Cmd+] | Forward |
| Cmd++ | Zoom in |
| Cmd+- | Zoom out |
| Cmd+0 | Reset zoom |
| Cmd+D | DevTools toggle |
| Escape | Blur URL, focus web view |

### 마우스

- 우클릭: context menu (inspect, copy link, etc.)
- 링크 클릭: navigate
- 드래그: text select

---

## 제약사항

### Blocked Features
- Geolocation API (permission 필요)
- Camera/Microphone (permission 필요)
- Payment Request API (미지원)
- Full-screen (IDE window boundary 내에서만)

### Allowed Features
- Local storage (per domain)
- Cookies (session only)
- WebSocket
- Service Worker (선택사항)

---

## 접근성

- Tab order: URL bar → Web view → DevTools
- Keyboard: 모든 조작 keyboard-only 가능
- Color contrast: UI ≥ 4.5:1 (web content 는 외부 책임)
- Focus ring: 5px primary.500

---

**마지막 수정**: 2026-04-25  
**상태**: 설계 완료 — SPEC-V3-007 plan

