# MoAI Studio

> **moai-adk 의 공식 macOS 네이티브 Agent IDE.**
> Claude Code 를 subprocess 로 호스트하여 27개 hook 이벤트 + 26 전문 에이전트 + TRUST 5 품질 게이트 + @MX 태그 시스템 + Kanban/SPEC 워크플로우를 한 화면에서 시각화 · 조작한다.

**Status**: Design phase (v4 draft). Implementation not started.
**Platform**: macOS only (macOS 14+, Apple Silicon + Intel).
**License**: MIT
**Language**: Swift (UI) + Rust (Core)
**Brand**: MoAI Studio (확정 — 2026-04-11, DESIGN.v4 §14 O6 RESOLVED)
**Package**: `moai-studio` (바이너리/패키지 식별자)

> **저장소 리네임 안내**: 본 저장소의 디스크 경로 (`~/moai/moai-cli`) 와 GitHub URL (`modu-ai/moai-cli`) 은 브랜드 확정에 따라 **장래 `moai-studio` 로 리네임 예정**. 설계 문서와 setup 명령의 현 경로 표기는 실제 디스크 상태와의 정합성을 위해 리네임 시점까지 유지한다.

---

## 현재 상태

이 저장소는 **설계 단계**입니다. 아직 코드 없음. 4개의 설계 문서가 순차 진화해왔습니다:

| 파일 | 버전 | 상태 | 요약 |
|---|---|---|---|
| [DESIGN.md](./DESIGN.md) | v2 (2026-04-11) | 참고용 | 초기 아키텍처 — 일부 가정 오류 (Bridge, hooks.yaml) |
| [DESIGN.v3.md](./DESIGN.v3.md) | v3 (2026-04-11) | 참고용 | "SDK 임베드" 가정, Pure Swift 제안 |
| **[DESIGN.v4.md](./DESIGN.v4.md)** | **v4 (2026-04-11)** | **★ 현 기준** | 공식 문서 검증 완료, Rust core + Swift UI, IDE MCP Server Pattern |
| [NEXT-STEPS.md](./NEXT-STEPS.md) | v1 | ★ 작업 계획 | Pre-M0 spike + M0 킥오프 + 열린 결정 + 커뮤니티 |

**v4 를 보십시오.** v2, v3 는 evolution 기록으로만 보존.

---

## 핵심 아키텍처 (v4)

```
┌─────────────────────────────────────────────────────────┐
│           MoAI Studio.app (macOS, SwiftUI + Rust)       │
│                                                         │
│  Swift UI (SwiftUI + AppKit)                            │
│     │ swift-bridge FFI                                  │
│     ▼                                                   │
│  Rust Core (moai-core workspace)                        │
│     - ClaudeSubprocessManager                           │
│     - StreamJsonCodec                                   │
│     - IdeMcpServer (127.0.0.1 + ~/.claude/ide/*.lock)   │
│     - HookHttpEndpoint                                  │
│     - Store (rusqlite WAL)                              │
│     - Git (git2)                                        │
└──────────────┬─────────────────────────┬────────────────┘
               │ stdin/stdout            │ HTTP loopback
               ▼                         ▼
       claude subprocess         Plugin http hooks
       (per workspace)           POST /hooks/<event>
```

### 3가지 핵심 피벗 (v4)

1. **Subprocess 호스트** (SDK 임베드 아님) — 공식 Agent SDK 조차 `claude` 를 subprocess spawn
2. **IDE MCP Server Pattern PRIMARY** — VS Code 확장과 동일한 공식 아키텍처
3. **Rust core + Swift UI** — macOS 단독이지만 actor supervision / stream-json 성능 / 메모리 안전성 이득

### 7가지 Moat (경쟁사 0)

1. 공식 IDE MCP Server Pattern 채택
2. Hook 18-25 이벤트 양방향 + tool input rewriting
3. LSP as plugin feature (`.lsp.json`)
4. Kanban + Memory + InstructionsGraph 3종
5. @MX 태그 거터 + TRUST 5 게이지
6. In-app Claude UI 조작 (`mcp__moai__*`)
7. Native permission dialog + updatedPermissions

---

## 저장소 구조

```
moai-cli/                  ← 현 저장소 이름 (MoAI Studio 로 리네임 예정)
├── README.md              ← 이 파일
├── DESIGN.md              ← v2 (참고)
├── DESIGN.v3.md           ← v3 (참고)
├── DESIGN.v4.md           ← v4 (★ 현 기준)
├── NEXT-STEPS.md          ← 4 단계 다음 작업
├── REFERENCES.md          ← 참조 저장소 설정 가이드
├── .gitignore
├── .references/           ← 로컬 전용 (gitignored)
│   ├── moai-adk-go  →  /Users/goos/MoAI/moai-adk-go
│   └── claude-code-map  →  /Users/goos/moai/claude-code-map
├── design-exports/        ← 12 PNG UI 목업 + v1 PDF
└── research/              ← 리서치 결과
    ├── R1-native-ai-shells.md           (50KB, 경쟁사)
    ├── B1-bridge-direct-connect.md      (10KB, 소스 분석)
    ├── B2-hook-events-tool-system.md    (20KB, 소스 분석)
    ├── B3-extension-points.md           (24KB, 소스 분석)
    ├── B4-official-docs-verification.md (19KB, 공식 문서)
    └── B5-wsl-wslg-windows-coverage.md  (13KB, Linux 포기 근거)
```

## 개발 환경

### 참조 저장소 (읽기 전용 심볼릭 링크)

`.references/` 디렉토리는 외부 저장소의 소스 코드를 참조합니다. **gitignored**, 로컬 개발 전용.

| 심볼릭 링크 | 타깃 | 용도 |
|---|---|---|
| `.references/moai-adk-go` | `/Users/goos/MoAI/moai-adk-go` | moai-adk Go CLI 소스 — Hook 통합, plugin 자동 설치, 27 이벤트 wiring 참조 |
| `.references/claude-code-map` | `/Users/goos/moai/claude-code-map` | Claude Code 소스 (mapped) — stream-json 프로토콜, SDKMessage, hook 이벤트, MCP 통합 참조 |

**사용 예:**
```bash
# moai-adk 소스 참조
cat .references/moai-adk-go/internal/hook/post-tool.go

# Claude Code 소스 참조
grep -r "HookEvent" .references/claude-code-map/src/
```

**복제 시 재설정:**
```bash
mkdir -p .references
ln -sf /path/to/moai-adk-go .references/moai-adk-go
ln -sf /path/to/claude-code-map .references/claude-code-map
```

상세는 [REFERENCES.md](./REFERENCES.md) 참조.

---

## 다음 단계

→ **[NEXT-STEPS.md](./NEXT-STEPS.md)** 를 보십시오.

4 단계 작업 계획:
1. **Pre-M0 검증 스파이크** (3-4일) — Claude CLI 공식 경로, IDE MCP, plugin http hook, GhosttyKit xcframework 검증
2. **M0 킥오프** (2주) — Rust core skeleton + Swift UI shell + 첫 hook 왕복
3. **열린 결정 (O1-O5)** — swift-bridge, rmcp, 미문서화 field 정책 등 (O6 브랜딩은 RESOLVED)
4. **커뮤니티 시작 신호** — README 로드맵, HN 예고, cmux 팀 outreach

---

## 브랜딩 제약 (Anthropic 공식)

출처: [Claude Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview)

- ✅ 허용: "MoAI Studio", "MoAI Agent IDE", "moai + Claude", "Powered by Claude"
- ❌ **금지**: "Claude Code" 명칭 사용, "Claude Code Agent", Claude Code ASCII art 차용
- ❌ **금지**: claude.ai OAuth 로그인 구현
- ✅ 인증: `ANTHROPIC_API_KEY`, Bedrock, Vertex, Foundry

---

## 라이선스

MIT License © 2026 modu-ai

---

## 관련 저장소

- [modu-ai/moai-adk](https://github.com/modu-ai/moai-adk) — Go CLI, MoAI Studio 가 통합하는 본체
- [ghostty-org/ghostty](https://github.com/ghostty-org/ghostty) — 터미널 엔진 (libghostty)
- [anthropics/claude-code](https://github.com/anthropics/claude-code) — Claude Code CLI
