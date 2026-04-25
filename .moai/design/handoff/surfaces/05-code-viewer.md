# Code Viewer Surface

---
title: Code Editor with LSP Integration
version: 1.0.0
source: SPEC-V3-006 MS-2
last_updated: 2026-04-25
---

## 개요

**Code Viewer** 는 syntax-highlighted code editor with LSP diagnostics, tree-sitter support, @MX gutter 을 제공한다. 다중 언어 지원 (Rust, Go, Python, TypeScript, etc).

상태: 구현 미완료 (SPEC-V3-006 MS-2 계획)

---

## 레이아웃

```
┌─────────────────────────────────────────────────────┐
│ [Close] main.rs  ●  [Fold/Unfold]                   │ ← Tab bar
├─────┬───────────────────────────────────────────────┤
│ MX  │ Line │ Gutter │ Code                          │
├─────┼─────┬────────┼─────────────────────────────────┤
│     │ 1   │ ⚙      │ fn main() {                    │
│ ⚠   │ 2   │        │   let x = 5;    // error: X   │
│ 🔗  │ 3   │ error  │   ^^^^^^^^^                    │
│     │ 4   │        │ }                              │
└─────┴─────┴────────┴─────────────────────────────────┘
```

### 칼럼별 너비

| 칼럼 | 너비 | 역할 |
|------|------|------|
| @MX | 60px | @MX:ANCHOR/WARN/NOTE/TODO badges |
| Line number | 40px | Line number (우정렬) |
| Gutter | 24px | Diagnostic icon, breakpoint |
| Code | flex | 실제 코드 렌더 |

---

## 신택스 하이라이트

### 색상 매핑 (tree-sitter)

| Element | HEX | 예 |
|---------|-----|-----|
| keyword | #C792EA | fn, let, if, impl |
| string | #C3E88D | "hello", 'x' |
| number | #F78C6C | 123, 0xFF |
| comment | #546E7A | // comment, /* */ |
| function | #82AAFF | func_name() |
| type | #FFCB6B | String, i32, &mut |
| variable | #EEFFFF | variable_name |
| operator | #89DDFF | +, =, &&, -> |
| constant | #F07178 | const VAR |

### 멀티 언어

- Rust ✓ (moai primary)
- Go ✓
- Python ✓
- TypeScript / JavaScript ✓
- JSON ✓
- Markdown ✓
- YAML ✓
- TOML ✓
- Bash ✓
- HTML / CSS ✓
- (tree-sitter 지원 모든 언어)

---

## LSP 진단

### 진단 아이콘 (Gutter)

```
⚠️ Warning (yellow)
❌ Error (red)
ℹ️ Hint (blue)
⚡ Deprecated (gray)
```

### Squiggly Underline

```
   let x = 5;
       ^^^^^^ ← error (red underline, wavy)
```

- 색: error red / warning yellow / hint blue
- 스타일: wavy underline (2px 두께)
- Hover: tooltip with message

### Hover Tooltip

```
┌───────────────────────┐
│ Type: i32             │
│ Expected: String      │
│                       │
│ Error: mismatched ... │
└───────────────────────┘
```

- 배경: neutral.900 (dark), neutral.50 (light)
- 테두리: 1px neutral.700
- Radius: 4px
- Shadow: 2
- 최대 너비: 400px (wrapping)

---

## @MX 거터

### 좌측 칼럼

```
@MX:ANCHOR  │ fn main() {
@MX:WARN    │ > DO NOT modify
@MX:NOTE    │ Uses unsafe code
@MX:TODO    │ - [ ] Implement X
            │ - [x] Implement Y
```

- 아이콘 색: primary/warning/info/neutral
- 호버: 배경 highlight, tooltip
- 클릭: line number 복사

---

## 인터랙션

### 키보드

| 단축키 | 동작 |
|--------|------|
| Cmd+F | Find in file |
| Cmd+H | Replace |
| Cmd+/ | Comment toggle |
| Cmd+[ | Indent decrease |
| Cmd+] | Indent increase |
| Cmd+D | Select word |
| Cmd+Shift+L | Select all occurrences |
| Enter | New line |
| Tab | Indent |
| Shift+Tab | Outdent |

### 마우스

- **클릭**: 커서 위치 설정
- **더블클릭**: 단어 선택
- **트리플클릭**: 라인 선택
- **드래그**: 텍스트 선택
- **우클릭**: context menu (copy/paste/comment/format)
- **스크롤**: 코드 스크롤 (가상 스크롤 1000+ lines)

---

## Find & Replace

### Find Bar

```
┌──────────────────────────────────┐
│ 🔍 [search query] (1/23) [Next] [Prev] [×]
└──────────────────────────────────┘
```

- 위치: 코드 상단
- Highlight: 모든 매칭 (primary.500 배경)
- Current match: 더 밝은 하이라이트
- Case sensitive toggle: 선택가능

### Replace Bar (Cmd+H)

```
┌──────────────────────────────────┐
│ 🔍 [search] [Replace] [Replace All] [×]
└──────────────────────────────────┘
```

---

## Minimap (선택사항)

우측 에지:

```
█████ ← 현재 viewport
█████
░░░░░
░░░░░
```

- 너비: 60px
- 마우스 drag: jump to line
- 색: syntax highlight 축소판

---

## Line Numbers & Folding

### Line Number 스타일

- 색: neutral.400 (secondary text)
- Align: right (padding-right 8px)
- Hover: neutral.300 (highlight)
- Selection: primary.500

### Code Folding

- 아이콘: ▼ (expanded) / ▶ (collapsed)
- 위치: line number 좌측
- 클릭: 해당 블록 fold/unfold
- 색: neutral.500 (hover=primary.500)

---

## 문법 범위 (Scope)

### 지원 수준

- Language: Go, Rust, Python, TypeScript, etc. (tree-sitter 기반)
- Scope: statement-level (function, class, if-block)
- Accuracy: ~95% (complex nested structure 제외)

---

## 상태

### Empty
```
"코드를 선택하세요"
```

### Loading
```
⟳ 신택스 강조 중...
```

### Populated
```
fn main() {
    let x = 5;
    println!("{}", x);
}
```

### LSP Unavailable
```
ℹ️ LSP server 연결 불가
  신택스 강조만 사용 가능
```

### Large File
```
⚠️ 파일이 매우 큼 (> 100KB)
  최적화 모드로 렌더
```

---

## 접근성

- **Color contrast**: syntax ≥ 3:1 (UI vs code)
- **Line number**: screen reader 인식 가능
- **Keyboard**: 모든 editing 100% keyboard-only
- **Focus ring**: 5px primary.500

---

## 성능

### Rendering
- Virtual scroll: 1000+ lines
- Syntax highlight: tree-sitter async (~100ms)
- LSP diagnostics: async 500ms debounce
- Selection rendering: O(1) GPU

### Motion
- 부드러운 scroll: 200ms easeOut

---

**마지막 수정**: 2026-04-25  
**상태**: 설계 완료 — SPEC-V3-006 MS-2 plan

