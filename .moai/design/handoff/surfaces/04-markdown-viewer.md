# Markdown Viewer Surface

---
title: CommonMark + GFM Renderer
version: 1.0.0
source: SPEC-V3-006 MS-1
last_updated: 2026-04-25
---

## 개요

**Markdown Viewer** 는 CommonMark + GitHub Flavored Markdown (GFM) 을 렌더한다. SPEC 문서 읽기에 최적화. KaTeX/Mermaid 는 MS-2 (텍스트 fallback 지원).

상태: 구현 진행 중 (SPEC-V3-006 MS-1)

---

## 지원 요소

### 텍스트 요소
- Heading (h1~h6)
- Paragraph
- Emphasis (italic) & strong (bold)
- Inline code
- Strikethrough (`~~text~~`)
- Links & images
- Hard/soft line breaks

### 블록 요소
- Unordered list (bullet points)
- Ordered list (numbered)
- List nesting
- Blockquote
- Code block (with syntax highlight)
- Horizontal rule (`---`)
- Table (GFM)

### 고급 (MS-2, TBD)
- KaTeX: `$$math$$` (block), `$math$` (inline)
- Mermaid: ` ```mermaid ... ``` ` (diagram)
- Footnotes
- Task list (`- [x] done`)

---

## 타이포그래피

### Heading 스케일

| Tag | Font Size | Weight | Line Height |
|-----|-----------|--------|-------------|
| h1 | 30px (3xl) | 700 bold | 1.2 tight |
| h2 | 24px (2xl) | 700 bold | 1.2 tight |
| h3 | 20px (xl) | 600 semibold | 1.2 tight |
| h4 | 18px (lg) | 600 semibold | 1.3 |
| h5 | 16px (md) | 600 semibold | 1.4 |
| h6 | 14px (base) | 600 semibold | 1.5 |

### 본문 텍스트

| 요소 | 크기 | Weight | 색상 |
|------|------|--------|------|
| Paragraph | 16px (md) | 400 regular | neutral.50 (dark) |
| List | 16px (md) | 400 regular | neutral.50 |
| Blockquote | 16px (md) | 400 italic | neutral.300 |
| Code (inline) | 12px (sm) | 400 | syntax.constant |
| Code (block) | 12px (sm) | 400 mono | — |

### Line Height & Spacing

- **Body line-height**: 1.75 (relaxed, 가독성)
- **Paragraph margin**: 16px (spacing.4) bottom
- **Heading margin-top**: 20px (spacing.5) (first h2+)
- **Heading margin-bottom**: 8px (spacing.2)
- **List item margin**: 4px (spacing.1)
- **List nested indent**: 16px (spacing.4)

---

## 레이아웃

### 최대 너비 & 컨테이너

```
┌──────────────────────────────────────────────────────┐
│                                                        │
│  ← 16px margin                                       │
│  ┌──────────────────────────────────────────────┐   │
│  │                                              │   │
│  │  # Heading                                   │   │
│  │  max-width 780px (readability)              │   │
│  │                                              │   │
│  │  Paragraph text flows at 16px size with    │   │
│  │  relaxed line-height for comfortable       │   │
│  │  reading...                                 │   │
│  │                                              │   │
│  └──────────────────────────────────────────────┘   │
│                                       16px margin → │
└──────────────────────────────────────────────────────┘
```

- **Max width**: 780px (narrow column, newspaper-like)
- **Left/right margin**: 16px (spacing.4)
- **Inner padding**: 8px (spacing.2)
- **Vertical gutter space**: scrollable, 여백 충분

---

## @MX 거터

### Gutter 칼럼 (좌측)

```
@MX:ANCHOR  │ ## Architecture
@MX:WARN    │ > WARNING: DO NOT modify this section
@MX:NOTE    │ This implementation uses...
@MX:TODO    │ - [ ] Implement X
            │ - [x] Implement Y
```

- 너비: 60px (dedicated column)
- 배경: neutral.800 (dark), neutral.100 (light)
- Border: 1px neutral.700 (dark), neutral.200 (light)
- 아이콘 색:
  - ANCHOR: 🔗 primary.500
  - WARN: ⚠️ warning
  - NOTE: 📝 info
  - TODO: ☐ neutral.400

### Hover & Interaction

- 마우스 hover: 배경 +1 shade (밝아짐)
- 클릭: line number 복사
- Tooltip: "@MX:ANCHOR at line 42"

---

## 신택스 하이라이트 (Code Block)

### Tree-Sitter 색상

| Element | HEX | Usage |
|---------|-----|-------|
| keyword | #C792EA | if, for, fn, impl |
| string | #C3E88D | "text" |
| number | #F78C6C | 123, 0xFF |
| comment | #546E7A | // comment |
| function | #82AAFF | func_name() |
| type | #FFCB6B | String, i32 |
| variable | #EEFFFF | variable |
| operator | #89DDFF | +, =, && |

### 코드 블록 스타일

```
┌────────────────────────────────┐
│ rust                 [Copy]    │  ← 언어 라벨 + copy button
├────────────────────────────────┤
│ fn main() {                    │
│   println!("Hello");           │
│ }                              │
└────────────────────────────────┘
```

- 배경: neutral.900 (dark), neutral.50 (light)
- 테두리: 1px neutral.700 (dark), neutral.200 (light)
- Radius: 8px (lg)
- Padding: 12px (spacing.3)
- Overflow: horizontal scroll (large code)
- Copy button: 우측 상단, 호버 시 나타남

---

## 이미지 & 임베드

### 이미지

```
![alt text](image.png)

┌────────────────────────┐
│                        │
│   [렌더된 이미지]       │
│                        │
└────────────────────────┘
```

- Max width: 100% (container width, max 780px)
- Border: 1px neutral.700 (dark), neutral.200 (light)
- Radius: 6px (md)
- Aspect ratio: preserve
- Alt text: hover tooltip

### 링크

- 색: brand.accent.500 (cyan, `#06B6D4`)
- Underline: hover 시만 표시
- Visited: brand.primary.700 (darker blue)

---

## 테이블 (GFM)

### 렌더

```
| Column 1 | Column 2 |
|----------|----------|
| Cell     | Cell     |
| Left     | Right    |
```

- Header: 600 weight, neutral.800 bg (dark)
- Rows: alternating neutral.900/850 (dark)
- Border: 1px neutral.700
- Padding: 8px (spacing.2)
- Align: left/center/right (GFM syntax 지원)

---

## Blockquote

```
> Important note
> spanning multiple lines
```

- 좌측 border: 4px primary.500
- 배경: neutral.850 (dark), neutral.50 (light)
- Padding: 12px (spacing.3)
- Italic text

---

## 리스트

### Unordered

```
- Item 1
  - Nested item
    - Deep nested
- Item 2
```

- Bullet: primary.500 dot (·)
- Indent: 16px per level
- Item margin: 4px

### Ordered

```
1. First
2. Second
   1. Nested
3. Third
```

- Number: primary.500, bold weight

---

## Table of Contents (선택사항)

우측 사이드 패널 (h2/h3 only):

```
[TOC]
  Architecture
    Components
    Data Flow
  Implementation
    Setup
```

- 클릭 시: page scroll to section
- Indent: 12px per level
- 색: secondary text

---

## 상태

### Empty
```
"파일을 선택하세요"
```

### Loading
```
⟳ 로드 중...
```

### Populated
```
# Heading

Rendered markdown content...
```

### Error
```
⚠ 마크다운 파싱 실패

[파일 다시 로드]
```

---

## 성능

### 렌더링
- Virtual scroll: 1000+ lines
- Syntax highlight: 500ms max (tree-sitter async)
- Image lazy load: scroll-into-view

### Motion
- Scroll smooth: 200ms easeOut
- prefers-reduced-motion: 0ms

---

## 접근성

- **Heading hierarchy**: h1 → h2 → h3... (skip 없음)
- **Contrast**: 모든 텍스트 ≥ 4.5:1
- **Link underline**: 필수 (색만으로 구분 X)
- **Alt text**: 이미지 필수 (screen reader)
- **List structure**: semantic `<ul>` / `<ol>`

---

**마지막 수정**: 2026-04-25  
**상태**: 구현 진행 중 — SPEC-V3-006 MS-1

