# Edge Cases & Error States

---
title: Comprehensive Error & Edge Case Handling
version: 1.0.0
source: All surfaces
last_updated: 2026-04-25
---

## 전역 상태

### Empty Workspace

모든 surfaces에서:

```
┌──────────────────────────────────┐
│ 📂 No workspace open             │
│                                   │
│ [Select a folder to open]        │
│ (File explorer → Select)         │
│                                   │
│ [+ Create new project]           │
│ [Browse examples]                │
└──────────────────────────────────┘
```

---

## Terminal Surface Edge Cases

### Case: PTY Dead

```
~/project $ _  ✗ (red badge)

Status: PTY disconnected
Last exit: signal 11 (segmentation fault)

[Restart shell] [Close tab]
```

- 커서: gray (neutral.600)
- Prompt: disabled (no input accepted)
- Badge: red ✗

### Case: Large Output (> 100MB)

```
~/project $ cargo test (... 10000+ lines)

⚠️ Terminal buffer full

Displaying last 1000 lines.
Older output not available in memory.

[Export scrollback to file]
[Clear scrollback]
```

- Auto-clear: scrollback 버퍼 주기적 trim (1000 line max)

---

## File Explorer Surface Edge Cases

### Case: Cannot Read Directory

```
⚠️ Error loading directory

src/private/ — Permission denied

[Retry] [Skip] [Show in terminal]
```

- 아이콘: ⚠️ (yellow triangle)
- Interactive: retry 또는 skip

### Case: Too Many Files (> 10000)

```
📁 src/  ▷ (collapsed, 1000+ files detected)

⚠️ This folder has many files (2341)

Showing first 100 for performance.

[Show all (slow)] [Search instead]
```

- Lazy load: 처음 100개만 render
- Filter: 검색으로 narrow down

### Case: Broken Symlink

```
📁 src/
 ├─ link.rs  ⚠️ (broken symlink icon)
```

- 아이콘: ⚠️
- 클릭: error toast "Target not found"

---

## Code Viewer Surface Edge Cases

### Case: LSP Server Unavailable

```
fn main() {
    let x = 5;
}

ℹ️ LSP server unavailable

Syntax highlighting only.
Type hints and diagnostics disabled.

[Configure LSP] [Try again]
```

- Color: info blue
- Fallback: syntax highlight still works (tree-sitter)

### Case: File Encoding Error

```
⚠️ Encoding error

Detected: binary-like (UTF-8 decode failed)

Show as:
[Hex] [Latin-1] [UTF-8 with errors]

Fallback: displaying as UTF-8 (lossy)
```

- 사용자 선택: hex / alternative encoding

### Case: 100KB+ Code File

```
fn main() { ... (line 50000)

⚠️ Large file (2.5 MB)

Enabling optimization:
- Virtual scrolling
- Syntax highlight async
- LSP diagnostics limited (first 100 lines)

[Load all anyway]
```

---

## Markdown Viewer Surface Edge Cases

### Case: KaTeX/Mermaid Not Loaded

```
# Heading

## Math (MS-2 deferred)

$$
\begin{align}
E = mc^2
\end{align}
$$

[Math rendering pending — MS-3]
(shows LaTeX source text)
```

- Fallback: LaTeX source 텍스트로 표시
- Banner: "Math/diagrams in MS-2"

### Case: Image Failed to Load

```
![alt text](missing-image.png)

⚠️ Image failed to load

File not found or broken link.

[Reload] [Browse for file]
```

---

## Agent Dashboard Surface Edge Cases

### Case: Agent Crash

```
❌ Agent error

Claude subprocess crashed:
Signal 11 (segmentation fault)

Last event: tool_use_start (Bash)

[View log] [Restart] [Report issue]
```

- 색: error red
- Action: restart, view log, report

### Case: Network Timeout

```
⏱️ Request timeout

Agent unresponsive for 30 seconds.
Last event: message_start (30s ago)

[Pause] [Kill] [Wait more]
```

---

## Pane/Tab Surface Edge Cases

### Case: Pane Below Minimum Size

```
User tries to resize below 240×120:

┌───────┬─────────────────────┐
│ A     │ B                   │
├───────┼─────────────────────┤
│       │ (cannot shrink      │
│       │  smaller)           │
│       │                      │
└───────┴─────────────────────┘

Divider clamps at min ratio (0.3/0.7)
Visual feedback: divider stops, shake animation
```

### Case: Dirty Tab + Close App

```
User closes app with unsaved changes:

┌──────────────────────────────┐
│ Save changes before closing? │
│                               │
│ [main.rs] ● (1 unsaved)     │
│ [lib.rs]  ● (1 unsaved)     │
│                               │
│ [Save all] [Discard] [Cancel]│
└──────────────────────────────┘
```

---

## Git Surface Edge Cases

### Case: Merge Conflict

```
⚠️ Merge conflict

1 file has conflict markers

src/main.rs — 2 conflicts
[Resolve] [View]

Conflict resolver:
┌──────────┬──────────┬──────────┐
│ Base     │ Ours     │ Theirs   │
├──────────┼──────────┼──────────┤
│ x = 5    │ x = 10   │ x = 15   │
│          │[Accept] │[Accept]   │
└──────────┴──────────┴──────────┘
```

### Case: Detached HEAD

```
⚠️ Detached HEAD

You are on: abc1234 (commit)
Not on any branch.

To fix:
[git checkout -b new-branch]
[git checkout main]
[View history]
```

---

## Network/Connection Edge Cases

### Case: Offline (Web Browser)

```
🌐 Web Browser (offline)

⚠️ No internet connection

Current URL: https://example.com
Last successful load: 2h ago

[Retry] [Browse local]
```

### Case: Dev Server Connection Failed

```
🚀 localhost:3000 detected
But cannot connect.

Is the server running?

[Retry] [View logs] [Kill]
```

---

## 공용 에러 상태 스타일

### Error Banner (상단)

```
┌──────────────────────────────────┐
│ ❌ Operation failed               │
│ Please try again or contact      │
│ support.                         │
│                                   │
│ Error code: E-001234             │
│ [Report] [Dismiss]               │
└──────────────────────────────────┘
```

- 배경: error.red @ 10% (subtle)
- 테두리: 1px error.red (좌측 4px thick)
- 아이콘: ❌ (red)
- 폰트: 12px (sm), regular

### Toast Error (우상단)

```
┌──────────────────────────────────┐
│ ✗ Error occurred                 │
│ Check connection                 │
│ [Retry]                          │
└──────────────────────────────────┘
```

- Duration: 10 seconds (긴 시간, 더 눈에 띄도록)
- Action button: retry 또는 dismiss

---

## 복구 전략

| 상황 | 자동 복구 | 수동 옵션 |
|------|----------|---------|
| LSP unavailable | No | Retry / Reconfigure |
| File deleted mid-edit | No (warn) | Close tab / Reload |
| Network timeout | 3x retry (5s apart) | Kill / Wait |
| Permission denied | No | Run as admin / Skip |
| Corrupted JSON layout | Yes (load default) | — |
| Pane tree invalid | Yes (rebuild) | — |

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — 모든 edge case

