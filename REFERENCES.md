# Reference Repositories Setup

**MoAI Studio** 개발 중 외부 저장소 소스 코드를 자주 참조합니다. 이 문서는 **로컬 개발 환경에서 참조 저장소를 설정하는 방법**을 설명합니다.

참조 저장소는 `.references/` 디렉토리에 **심볼릭 링크**로 배치되며, **gitignored** 입니다. 즉, 저장소 외부의 코드를 복사하거나 fork 하지 않고 읽기 전용으로 참조합니다.

> **경로 안내**: 아래 setup 명령은 저장소의 현재 디스크 위치인 `~/moai/moai-cli` 를 기준으로 작성되었습니다. 저장소가 장래 `moai-studio` 로 리네임되면 명령의 `cd` 대상도 함께 업데이트됩니다.

---

## 왜 참조 저장소가 필요한가

MoAI Studio 는 두 가지 외부 시스템 위에 구축됩니다:

1. **moai-adk** (Go CLI) — MoAI Studio 가 통합하는 본체. Hook 핸들러, plugin 설정, 27 이벤트 wiring, SPEC workflow, TRUST 5 게이지, @MX 태그 스캐너가 여기에 구현되어 있습니다.
2. **Claude Code CLI** (TypeScript + React+Ink) — MoAI Studio 가 subprocess 로 호스트하는 대상. stream-json 프로토콜, SDKMessage 타입, hook 이벤트 스키마, MCP 통합, plugin 시스템이 여기에 구현되어 있습니다.

MoAI Studio 코드를 작성할 때 이 두 소스를 **반복적으로 검증** 해야 합니다 — "moai-adk 가 Hook 을 어떻게 등록하는가?", "Claude Code 가 `PreToolUse` payload 에 어떤 필드를 채워 보내는가?" 같은 질문이 계속 발생합니다.

v2, v3 에서 경험한 **추측 기반 가정의 함정** (예: `.moai/hooks.yaml` 이 존재한다고 오인) 을 피하려면 소스를 직접 grep 하는 것이 유일한 안전한 방법입니다. `.references/` 는 이를 위한 장치입니다.

---

## 초기 설정

복제 직후 또는 재설치 시:

```bash
cd ~/moai/moai-cli    # 현 디스크 경로. 리네임 후에는 ~/moai/moai-studio
mkdir -p .references

# moai-adk Go CLI 소스
ln -sf /Users/goos/MoAI/moai-adk-go .references/moai-adk-go

# Claude Code CLI 소스 (mapped 버전)
ln -sf /Users/goos/moai/claude-code-map .references/claude-code-map

# 검증
ls -la .references/
# 결과:
#   lrwxr-xr-x  claude-code-map -> /Users/goos/moai/claude-code-map
#   lrwxr-xr-x  moai-adk-go -> /Users/goos/MoAI/moai-adk-go
```

**경로는 로컬 환경에 따라 다를 수 있습니다.** 자신의 머신에 맞게 수정.

---

## 사용 예

### moai-adk 소스 참조

**Hook 핸들러 확인:**
```bash
ls .references/moai-adk-go/.claude/hooks/moai/
# handle-post-tool.sh, handle-pre-tool.sh, handle-session-start.sh, ...
```

**Hook 이벤트 dispatch 로직:**
```bash
grep -rn "post-tool\|pre-tool\|session-start" .references/moai-adk-go/internal/hook/
```

**설정 파일 구조:**
```bash
ls .references/moai-adk-go/.moai/config/sections/
# quality.yaml, workflow.yaml, language.yaml, llm.yaml, mx.yaml, ...
```

**Template 시스템 (`moai init` 이 배포하는 것):**
```bash
ls .references/moai-adk-go/internal/template/templates/
```

### Claude Code 소스 참조

**Hook 이벤트 스키마:**
```bash
# HOOK_EVENTS 상수 위치
grep -n "HOOK_EVENTS" .references/claude-code-map/src/entrypoints/sdk/coreSchemas.ts

# PreToolUse 스키마
sed -n '414,423p' .references/claude-code-map/src/entrypoints/sdk/coreSchemas.ts
```

**Plugin 시스템:**
```bash
cat .references/claude-code-map/src/utils/plugins/schemas.ts | head -100
```

**stream-json codec 과 Query 엔진:**
```bash
ls .references/claude-code-map/src/cli/
ls .references/claude-code-map/src/bridge/
```

**MCP 통합:**
```bash
ls .references/claude-code-map/src/services/mcp/
```

**IDE MCP server 패턴:**
```bash
grep -A 30 "ide" .references/claude-code-map/src/utils/ide.ts
```

---

## 제약 사항

### .references/ 는 읽기 전용

`.references/` 디렉토리 안에서 파일을 **편집하거나 생성하지 마십시오**. 이는 외부 저장소를 오염시킵니다.

MoAI Studio 코드는 항상 저장소 루트 (`~/moai/moai-cli/` — 리네임 전 기준) 안에만 작성합니다.

### .references/ 는 gitignored

`.gitignore` 에 `.references/` 가 포함되어 있어 git 이 심볼릭 링크를 추적하지 않습니다. **복제한 개발자는 반드시 위의 "초기 설정" 을 실행해야 합니다.**

CI/CD 환경에서는 `.references/` 없이도 빌드가 성공해야 합니다 — 참조는 개발자의 이해를 돕는 수단일 뿐, 런타임 의존성이 아닙니다.

### Claude Code 버전 pinning

`claude-code-map` 은 특정 시점의 mapped 소스 스냅샷입니다. Claude Code 는 빠르게 진화하므로, `.references/claude-code-map` 이 최신 릴리스와 일치하지 않을 수 있습니다.

**권장:** 주요 설계 결정을 내릴 때마다 `.references/claude-code-map` 을 최신 버전으로 갱신 (혹은 새 snapshot 디렉토리 생성). 날짜 기반 디렉토리 전략도 가능:

```
.references/
├── claude-code-map            → /Users/.../claude-code-map (현 기준)
├── claude-code-map-2026-01    → 특정 시점 snapshot
└── claude-code-map-2026-04    → 또 다른 snapshot
```

---

## Claude Code 접근 팁

`claude` 세션이 MoAI Studio 디렉토리에서 시작되면 `.references/` 를 통해 외부 소스를 읽을 수 있습니다:

```
Read .references/claude-code-map/src/entrypoints/sdk/coreSchemas.ts
Grep pattern:"PreToolUse" path:.references/claude-code-map/src
```

이것이 "moai-adk 폴더 소스 코드 항상 참고해서 작업을 할 수 있도록" 한다는 설계 목표입니다.

---

## 언제 참조 저장소를 업데이트할까

- **moai-adk-go**: 상위 moai-adk 에서 Hook 관련 변경 또는 template 수정이 있을 때
- **claude-code-map**: Claude Code 릴리스에서 새로운 hook event, SDK message type, plugin 필드가 추가되었을 때 (Anthropic release notes 추적)

업데이트 방법:
```bash
# 외부 저장소를 git pull 하면 심볼릭 링크도 자동으로 최신 내용을 가리킴
git pull
cd /Users/goos/moai/claude-code-map && git pull  # 또는 재생성

# MoAI Studio 쪽에서는 아무 것도 할 필요 없음
```

---

**Version**: 1.1.0
**Last Updated**: 2026-04-11
**Brand**: MoAI Studio (confirmed)
