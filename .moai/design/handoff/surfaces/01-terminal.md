# Terminal Surface

---
title: GPU-Accelerated Terminal
version: 1.0.0
source: SPEC-V3-002
last_updated: 2026-04-25
---

## 개요

**Terminal** 은 moai-studio 의 가장 핵심 surface 이다. libghostty-vt (Zig FFI) 를 기반으로 **60fps@4K GPU-accelerated shell** 을 제공한다.

상태: **SPEC-V3-002 완료 + 변경금지 zone** (locked)

---

## 기능 & 상태

### 상태 분류

| 상태 | 시각 표현 | 설명 |
|------|----------|------|
| Idle | 커서만 깜빡 | PTY ready, 명령 기다리는 중 |
| Active | 텍스트 출력 animated | 명령 실행 중 |
| Disconnected | 회색 커서, "×" badge | PTY dead, 재시작 불가 |
| Scrollback | 스크롤 위치 표시 | 과거 출력 보기 중 |

### 주요 컴포넌트

```
┌─────────────────────────────────────────────────────┐
│ [↑ scrollbar] ← 우측 edge, 16px wide, hover=blue   │
├─────────────────────────────────────────────────────┤
│ ~/project $ echo "Hello"                            │
│ Hello                                                │
│ ~/project $                                          │
│ │  ← 커서 (blinking, 2px wide, primary.500)        │
│                                                      │
├─────────────────────────────────────────────────────┤
│ Status: idle | git main | exit 0 ✓                  │
└─────────────────────────────────────────────────────┘
```

### 시각적 신호

- **Prompt area**: 우측 상단에 CWD, git branch, exit code
  - CWD: secondary text (neutral.300), mono 12px
  - git branch: secondary color, Cmd+G 로 checkout 가능
  - Exit code: success (green ✓) / error (red ×)
  - PTY status: 좌측 corner dot (green=alive, gray=dead)

- **Cursor**: blinking (200ms visible, 200ms hidden)
  - Color: primary.500
  - Width: 2px (thin)
  - Shape: block (full-width character)

- **Selection**: primary.500 + alpha 0.2 (subtle highlight)
- **Scrollbar**: 16px wide, neutral.600, hover=primary.500, drag=primary.500

---

## 디자인 토큰 적용

| 항목 | 값 |
|------|-----|
| Font family | JetBrains Mono |
| Font size | **14px (base)** |
| Line height | 1.4 |
| Background | neutral.950 (dark), neutral.0 (light) |
| Text color | neutral.50 (dark), neutral.950 (light) |
| Cursor color | primary.500 |
| Selection bg | primary.500 @ 20% alpha |
| Scrollbar | neutral.600 (neutral.700 hover) |
| Border | neutral.800 (dark), neutral.100 (light) — divider 4px only |
| Padding (inner) | 8px (spacing.2) |

---

## Syntax Highlighting (터미널 특화)

터미널은 ANSI 256-color 또는 true color (24-bit) 를 지원한다. Shell prompt 색상:

- **Prompt ($)**: primary.500 (blue)
- **Path**: secondary text (neutral.300)
- **Command output**: neutral.50 (default)
- **Error output**: error red
- **Success (exit 0)**: success green
- **Git status**: brand colors (branch name = cyan)

---

## 인터랙션

### 키 입력
- 모든 ASCII/Unicode 글자 입력 가능
- Backspace, Delete, Arrow keys (자동 shell 에 전송)
- Cmd+C / Cmd+D (interrupt / EOF)
- Cmd+L (clear screen = `clear` 명령)
- Cmd+K (clear scrollback 커스텀 구현)

### 마우스
- 클릭: 커서 위치 변경 (PTY 지원 시)
- 우클릭: context menu
  - Copy (선택 텍스트)
  - Paste (clipboard)
  - Clear scrollback
  - 선택 해제
- 휠 스크롤: scrollback 위로/아래로 (Shift+Page Up/Down 과 동등)

### Context Menu (우클릭)
```
┌─────────────────────┐
│ Copy      Cmd+C    │
│ Paste     Cmd+V    │
│ ─────────────────  │
│ Clear scrollback   │
│ ─────────────────  │
│ Select all  Cmd+A  │
│ Deselect    Esc    │
└─────────────────────┘
```

---

## 상태 시각화

### State: Idle
```
~/project $ _
```
- 커서 visible, blinking
- Status bar: "idle"
- 모든 상호작용 활성

### State: Active (명령 실행)
```
~/project $ cargo test
   Compiling moai-studio v0.0.1
    Finished `test` [unoptimized + debuginfo]
        running 103 tests
...
```
- 텍스트 animated (character-by-character 또는 burst)
- Scrollback 자동으로 아래로
- Status bar: "running..." (진행률 % 선택적)

### State: Disconnected
```
~/project $ _  ✗ (PTY error badge)
```
- 우측 상단: red "×" badge
- 커서: gray (neutral.600)
- 입력 disabled
- Status bar: "PTY disconnected"
- 해결책: 탭 재시작 (Cmd+W → 새 탭) 또는 shell restart button

### State: Scrollback
```
[↑ scrolled back to line 42 of 200]
~/project $ echo "5 minutes ago"
```
- Scrollbar: primary.500 (사용자 위치 표시)
- Bottom 에 "Return to latest" button (클릭 또는 Shift+End)

---

## 다크/라이트 테마 변환

### Dark Theme (기본)
- Background: neutral.950 (`#09090B`)
- Text: neutral.50 (`#FAFAFA`)
- Cursor: primary.500 (`#2563EB`)

### Light Theme
- Background: neutral.0 (`#FFFFFF`)
- Text: neutral.950 (`#09090B`)
- Cursor: primary.500 (`#2563EB`) — 양 테마 동일
- Contrast check: text/bg ≥ 4.5:1 ✓

---

## 성능 특성

### Rendering
- **Target**: 60fps @ 4K resolution
- **Method**: libghostty GPU-accelerated (Metal on macOS, Vulkan on Linux, DirectX on Windows)
- **Scroll**: 시간 O(1) (virtual scrolling, don't render off-screen)
- **Large output**: 1000+ lines 버퍼 (circular buffer 로 메모리 제한)

### Animation
- **Cursor blink**: 200ms fast (motion.duration.fast)
- **Text appear**: instant (character streaming)
- **Scrolling**: smooth 60fps

### Accessibility
- **Blink toggle**: prefers-reduced-motion 시 커서 blink=off (solid cursor)
- **Color fallback**: ANSI color 미지원 시 neutral.50 text
- **Contrast**: dark/light 모두 ≥ 4.5:1 (WCAG AA)

---

## 콘텍스트 메뉴 (우클릭)

```
┌──────────────────────┐
│ Copy        Cmd+C   │  ← 선택 텍스트 copy
│ Paste       Cmd+V   │  ← clipboard paste
│ ──────────────────  │
│ Clear scrollback    │  ← Cmd+K (alias)
│ ──────────────────  │
│ Select all  Cmd+A   │
│ Deselect    Esc     │
└──────────────────────┘
```

---

## Keyboard Shortcuts (Terminal Focused)

| 단축키 | 동작 |
|--------|------|
| Cmd+C / Ctrl+C | Interrupt (SIGINT) |
| Cmd+D / Ctrl+D | EOF (close if empty) |
| Cmd+L | Clear screen |
| Cmd+K | Clear scrollback |
| Cmd+A | Select all |
| Cmd+C | Copy selection |
| Cmd+V | Paste clipboard |
| Shift+Page Up | Scroll up |
| Shift+Page Down | Scroll down |
| Shift+End | Jump to latest (if scrolled back) |

---

## Future Enhancements (MS-2+)

- Image 렌더링 (sixel, iTerm2 image protocol)
- 하이퍼링크 support (OSC 8)
- URL auto-detect + click-to-open
- Session recording/playback
- Search in scrollback (Cmd+F)

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — SPEC-V3-002 구현됨, 변경금지 zone

