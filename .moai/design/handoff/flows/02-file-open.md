# User Flow — File Open

---
title: Open File Flow
version: 1.0.0
source: SPEC-V3-005
last_updated: 2026-04-25
---

## 흐름도

```
User Action (File Explorer / Command)
    ↓
[Detect file type]
    ↓
    ├─ .md → [Markdown Viewer]
    ├─ .rs / .go / .py / .ts → [Code Viewer]
    ├─ .png / .jpg / .gif → [Image Viewer]
    ├─ .pdf → [PDF Viewer or warning]
    └─ binary → [Binary leaf + "Cannot display"]
```

---

## File Explorer Click

### 단계

1. **File tree 에서 파일 클릭**

```
📁 src/
 ├─ main.rs     ← 클릭
 ├─ lib.rs
 └─ tests.rs
```

2. **파일 타입 감지**

확장자 based:
- `.rs` → Code Viewer (Rust syntax)
- `.md` → Markdown Viewer (with @MX gutter)
- `.png` → Image Viewer (with metadata)

3. **새 탭 생성 및 content 렌더**

```
┌───────────────────────────────────┐
│ [main.rs] [lib.rs] [+]            │
├───────────────────────────────────┤
│ fn main() {                       │
│   println!("Hello");              │
│ }                                 │
```

4. **Active 탭 표시 (bold)**

---

## Command Palette (Cmd+P)

### 1단계: 검색창 열기

```
┌───────────────────────────────────┐
│ 🔍 [search query]                 │
├───────────────────────────────────┤
│ Results:                          │
│ · src/main.rs                     │
│ · src/lib.rs                      │
│ · tests/integration.rs            │
│ (10+ more, fuzzy matching)        │
└───────────────────────────────────┘
```

### 2단계: Fuzzy Match

입력: "main"
```
Results (matching "main"):
 1. src/main.rs         (exact)
 2. main.test.ts        (substring)
 3. bin/main_cli.rs     (substring)
```

- 검색: 파일명 + 전체 경로
- Order: exact match → substring → distance
- Real-time: 100ms debounce

### 3단계: 선택 & 열기

Arrow Down → Enter:

```
[main.rs selected]

↓ [renders main.rs in Code Viewer]
```

---

## Drag & Drop (File Explorer → Pane)

### 1단계: Drag

```
📁 src/
 ├─ main.rs  (drag start)
 ├─ lib.rs
 └─ tests.rs
```

- Source: opacity 50% (fade)
- Cursor: "dragging..." indicator

### 2단계: Drop Target

```
Pane 1 (Code Viewer)
  ↓
[Drop zone highlight]
  ↑
Pane 2 (Terminal)
```

- Target: primary.500 2px dashed border
- Drop-able: pane areas

### 3단계: Drop & Open

File 이 Pane 에 drop:

```
┌──────────────────────┐
│ [main.rs]            │ ← 새 탭
├──────────────────────┤
│ fn main() {          │
│   ...                │
└──────────────────────┘
```

---

## 파일 타입별 렌더

### Code File (.rs, .go, .py, .ts, etc.)

```
Code Viewer 로드:
1. 신택스 강조 (tree-sitter async)
2. LSP 진단 (async, 500ms debounce)
3. Line numbers + gutter
4. @MX 거터 (if present)

성능:
- 1000 lines: < 200ms
- 10000 lines: virtual scroll
```

### Markdown File (.md)

```
Markdown Viewer 로드:
1. CommonMark 파싱 (instant)
2. Syntax highlight code blocks
3. @MX 거터 렌더
4. Table of contents (optional)

특수:
- .moai/specs/ .md → AC state tracker
- README.md → [+] TOC 사이드 패널
```

### Image File (.png, .jpg, .gif)

```
Image Viewer 로드:
1. 이미지 렌더
2. Metadata: dimension, size, format
3. Zoom controls (Cmd++/- or mousewheel)
4. Download button

Diff mode (if comparing 2 images):
- Side-by-side
- Overlay with opacity slider
- SSIM score
```

### Binary File

```
┌──────────────────────────────────┐
│ [binary_file.exe]                │
├──────────────────────────────────┤
│                                   │
│ 🚫 Cannot display binary file    │
│                                   │
│ Type: Executable (ELF)            │
│ Size: 2.4 MB                      │
│                                   │
│ [Open in external app] [Download] │
│                                   │
└──────────────────────────────────┘
```

---

## 에러 처리

### 파일 없음 (Deleted)

```
⚠️ File not found

main.rs was deleted or moved

[Reload file tree] [Close tab]
```

### 파일 너무 큼 (> 100MB)

```
⚠️ File too large

main_dump.bin (250 MB)

Virtual scroll 활성화
Only first 10,000 lines rendered

[Load more] [Load all]
```

### 권한 없음 (Permission denied)

```
❌ Cannot read file

Permission denied: .env

Check file permissions or open with elevated privileges

[Open as admin] [Skip] [Close tab]
```

### 인코딩 에러

```
⚠️ Encoding error

Unable to decode: main.rs

Detected: binary-like content
Fallback: show as hex or text anyway?

[Show as hex] [Show as text] [Close]
```

---

## Dirty State (수정됨)

파일을 열고 수정하면:

```
┌──────────────────────────────────┐
│ [main.rs] ●  [lib.rs] [+]       │ ← white dot on main.rs (dirty)
├──────────────────────────────────┤
│ fn main() {  ← modified content  │
│   let x = 5; // changed          │
│ }                                │
```

탭 닫기 시:

```
⚠️ Unsaved changes

main.rs has unsaved changes.

[Save] [Discard] [Cancel]
```

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — 파일 열기 흐름

