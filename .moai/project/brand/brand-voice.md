# Brand Voice — MoAI Studio

> v1.0.0 baseline (2026-04-25). 모두의AI 모브랜드 voice + agentic coding IDE 컨텍스트.
> Notion 모두의AI 디자인 문서 접근 가능 시 후속 갱신.

---

## Tone

**confident, technical, calmly direct** — 개발자 동료처럼 말한다. 과장 없이, 지시 없이, 결과로 증명한다.

- 정확한 기술 용어 사용 (PaneTree, GPUI Entity, libghostty, hook event)
- 영업/마케팅 어조 회피
- 한국어 + 영문 자연스럽게 혼용 (코드 키워드 영문 보존)

## Register Spectrum

| 축 | 위치 |
|----|------|
| formal ↔ informal | informal-leaning (개발자 친근함) |
| serious ↔ playful | serious-leaning (정밀성 우선, 농담 절제) |
| technical ↔ accessible | technical (개발자 페르소나, jargon 허용) |

## Vocabulary

### preferred (선호)

- "build", "ship", "wire", "implement", "delegate"
- "구현", "통합", "위임", "분기"
- "agent", "orchestrator", "specialist"
- "verified locally" (CI 의존 표현 회피)
- "TRUST 5", "SPEC", "@MX", "Path A/B" (도메인 용어)

### avoided (금지)

- "innovative", "cutting-edge", "game-changing", "revolutionary"
- "혁신적인", "차세대", "꿈의", "마법 같은"
- "leverage" (use "use" 또는 "활용")
- "seamless" (use "smooth" 또는 "매끄러운")
- "AI-powered" (구체 능력 명시: "Claude Code 호스트" 등)

## Audience Familiarity

- **jargon_level**: high (expert audience — Rust 개발자, Claude Code 사용자, moai-adk 채택자)
- **assumed_knowledge**: cargo workspace + GPUI rendering + git workflow + agentic AI 기본 이해

## Example Phrases

- "탭 전환 시 last_focused_pane 을 복원한다 — REQ-P-023 invariant."
- "PR #9 admin override 머지 — local 5 gates GREEN 검증."
- "GitHub Actions billing 의존 제거. 로컬 cargo test --workspace 가 quality gate."
- "Render layer escape hatch 완성. 다음 surface 는 V3-005 file explorer."

## Anti-Examples

- "혁신적인 AI 기반 개발 환경" → 과장
- "Unlock developer productivity with our cutting-edge IDE" → 영업 어조
- "Welcome to the future of coding" → 추상적
- "AI-powered seamless workflow integration" → 무의미 buzzword

---

Version: 1.0.0
Last Updated: 2026-04-25
Pending: 모두의AI Notion 정합
