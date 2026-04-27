---
id: SPEC-V3-DIST-001
version: 1.1.0
status: ready
created_at: 2026-04-27
updated_at: 2026-04-27
author: MoAI (main session, manager-spec fallback per memory pattern)
priority: Medium
issue_number: 0
depends_on: [SPEC-V3-011]
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, distribution, packaging, homebrew, scoop, aur, appimage]
revision: v1.1.0 (USER-DECISION-DIST-A/B/C 세 게이트 모두 RESOLVED, status draft → ready)
---

# SPEC-V3-DIST-001: Distribution Channel Registration — Homebrew Cask + Scoop + AppImage + AUR (Cert 미보유 0 마찰 채널)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.1.0 | 2026-04-27 | USER-DECISION-DIST-A/B/C 세 게이트 모두 RESOLVED (sess 6). DIST-A=(a) modu-ai/homebrew-tap custom tap, DIST-B=(a) modu-ai/scoop-bucket custom bucket, DIST-C=(a) release.yml 자동화. status draft → ready. v0.1.0 release Path B 의 후속 배포 SPEC 으로 진입 가능. |
| 1.0.0-draft | 2026-04-27 | 초안 작성. 2026-04-27 sess 5 의 USER-DECISION-PK-B (b) 미보유 결정의 후속 작업으로 무료 배포 채널 (Homebrew Cask, Scoop bucket, AUR PKGBUILD, AppImage README 안내) 등록 SPEC. 본 SPEC 은 SPEC-V3-011 MS-1 산출 (cross-platform unsigned 빌드 인프라) 의 artifact 를 입력으로 사용한다. RG-DIST-1 (macOS Homebrew Cask) / RG-DIST-2 (Windows Scoop) / RG-DIST-3 (Linux AUR + AppImage) / RG-DIST-4 (README quarantine 우회 안내) / RG-DIST-5 (release-time 자동화). REQ 14 건, AC 9 건, MS-1/MS-2/MS-3, USER-DECISION-DIST-A/B/C 세 게이트. |

---

## 1. 개요

### 1.1 목적

USER-DECISION-PK-B (서명 인증서 보유) 가 (b) 미보유로 결정 (2026-04-27 sess 5) 됨에 따라, macOS Gatekeeper / Windows SmartScreen 의 **사용자 마찰을 OS 레벨 인증서 없이도 최소화** 하기 위한 무료 배포 채널을 등록한다. Homebrew Cask, Scoop bucket, AUR PKGBUILD, AppImage README 안내가 그것이다.

핵심 원리: 패키지 매니저 (brew, scoop) 는 다운로드한 binary 의 quarantine 속성을 자동으로 제거 또는 우회 처리하므로, unsigned binary 도 사용자 추가 작업 없이 정상 실행된다. AUR / AppImage 도 Linux 의 무서명 표준 채널.

### 1.2 Scope

`crates/` 의 코드는 변경하지 않는다. 본 SPEC 은 다음 위치에 새 artifacts 를 추가하거나 외부 저장소에 PR 을 생성한다:

- `dist/homebrew/Casks/moai-studio.rb` — Homebrew Cask formula
- `dist/scoop/moai-studio.json` — Scoop manifest
- `dist/aur/PKGBUILD` — Arch Linux PKGBUILD
- `dist/appimage/README.md` — AppImage 사용 안내
- `README.md` — quarantine 우회 + 패키지 매니저 설치 안내 섹션 추가
- `.github/workflows/release.yml` — (옵션, USER-DECISION-DIST-C 결정 시) 새 release publish 시 cask / scoop manifest 자동 업데이트 step 추가

### 1.3 비목적

- N1. 인증서 결제 — USER-DECISION-PK-B 결정 (b) 미보유 유지. 본 SPEC 은 그 결정의 *대안* 이지 *대체* 가 아니다.
- N2. Mac App Store / Microsoft Store 등록 — 인증서 + 심사 필요. 본 SPEC 무관.
- N3. Linux 패키지 저장소 (Debian APT, Fedora RPM, openSUSE OBS) 등록 — 별 SPEC. 본 SPEC 은 AUR + AppImage 까지.
- N4. 설치 후 자동 업데이트 — `update.json` 기반 self-updater 는 SPEC-V3-011 MS-3 의 T20. 본 SPEC 무관.
- N5. Code signing 의 대체 (예: Sigstore / cosign) — macOS Gatekeeper / Windows SmartScreen 은 Sigstore 를 받지 않으므로 무용. 본 SPEC 무관.

---

## 2. 입력 / 출력

### 2.1 입력

- SPEC-V3-011 MS-1 산출 artifacts:
  - `moai-studio-mac-{aarch64,x86_64}.dmg` (또는 `.app.tar.gz`)
  - `moai-studio-linux-amd64.deb`
  - `moai-studio-linux-amd64.AppImage`
  - `moai-studio-windows-x64.msi`
  - 각 artifact 의 SHA256 checksum (release 페이지)
- GitHub Releases publish 이벤트 (자동화 결정 시)
- USER-DECISION-DIST-A/B/C 결정값

### 2.2 출력

- `dist/homebrew/Casks/moai-studio.rb` (cask formula)
- `dist/scoop/moai-studio.json` (scoop manifest)
- `dist/aur/PKGBUILD` + `dist/aur/.SRCINFO`
- `dist/appimage/README.md`
- `README.md` 에 "Installation" 섹션 추가
- (옵션) `.github/workflows/release.yml` 에 cask/scoop bump step 추가
- 외부: `modu-ai/homebrew-tap` 또는 `homebrew/homebrew-cask` PR (USER-DECISION-DIST-A 따라)
- 외부: `modu-ai/scoop-bucket` 또는 `ScoopInstaller/Extras` PR (USER-DECISION-DIST-B 따라)
- 외부: AUR `moai-studio-bin` package upload (인증된 AUR 계정 필요)

---

## 3. 배경

### 3.1 USER-DECISION-PK-B 결정과 사용자 마찰

2026-04-27 sess 5 에서 USER-DECISION-PK-B 가 (b) 미보유로 결정됨에 따라:

- **macOS**: Gatekeeper 가 unsigned binary 실행 차단. 사용자가 우클릭→열기 또는 System Settings → Privacy & Security → "Open Anyway" 1회 클릭 필요. 또는 `xattr -dr com.apple.quarantine /Applications/moai-studio.app` 명령으로 quarantine 제거.
- **Windows**: SmartScreen "Windows protected your PC" 경고. "More info → Run anyway" 클릭 필요.
- **Linux**: 무서명 표준. `.deb` / AppImage 모두 별 인증서 없이 실행 가능.

### 3.2 패키지 매니저의 자동 우회

- **Homebrew Cask**: `brew install --cask` 명령은 다운로드한 binary 의 `com.apple.quarantine` 속성을 자동 제거 (cask formula 의 `postflight` 또는 `app` stanza 가 처리). 사용자는 추가 클릭 없이 즉시 실행 가능.
- **Scoop**: 다운로드 + 압축 해제 + PATH 등록. SmartScreen 의 "MOTW (Mark of the Web)" 속성을 자동 제거.
- **AUR**: makepkg 가 PKGBUILD 의 `pkgver()` / `prepare()` / `package()` 함수로 binary 또는 source 를 처리. 무서명 표준.
- **AppImage**: 단일 파일 portable. 실행 권한 부여 후 즉시 동작.

결과: **인증서 결제 없이 brew install / scoop install / yay -S 명령으로 거의 0 마찰 UX 달성**.

### 3.3 비교 — Anthropic Claude Desktop 사례

Anthropic Claude Desktop 도 brew cask `claude` 로 등록되어 macOS 사용자 다수가 그 경로로 설치한다 (인증서 결제 + brew cask 등록을 *둘 다* 사용 — brew 가 더 친숙한 UX 때문). moai-studio 는 인증서 미보유 상태에서 brew cask 만으로도 동등한 사용자 경험을 제공할 수 있다.

---

## 4. Goals

- G1. macOS 사용자가 `brew install --cask moai-studio` 명령으로 추가 클릭 없이 실행 가능한 상태로 설치 가능.
- G2. Windows 사용자가 `scoop install moai-studio` 명령으로 SmartScreen 경고 없이 설치 가능.
- G3. Arch Linux 사용자가 `yay -S moai-studio-bin` 또는 `paru -S moai-studio-bin` 명령으로 설치 가능.
- G4. 그 외 Linux 사용자가 `.AppImage` 파일을 다운로드 후 chmod +x → 실행 만으로 동작.
- G5. README 에 "Installation" 섹션이 명확히 위 4 채널을 안내.
- G6. (옵션, USER-DECISION-DIST-C 결정 시) GitHub Releases 에 새 tag publish 시 brew cask / scoop manifest 가 자동으로 PR 생성 또는 직접 commit.
- G7. 본 SPEC 의 어떤 산출도 인증서 결제를 요구하지 않는다 (USER-DECISION-PK-B (b) 의 prerequisite 위반 금지).
- G8. 본 SPEC 의 채널 등록은 SPEC-V3-011 MS-1 의 artifact 를 입력으로만 사용하며, 빌드 로직을 변경하지 않는다.

---

## 5. Non-Goals

- N1. 인증서 결제 / 갱신 / 키 관리. (별 SPEC 또는 USER-DECISION-PK-B 재논의)
- N2. Mac App Store / Microsoft Store / Snap Store 등록. (별 SPEC)
- N3. Debian APT / Fedora RPM / openSUSE OBS 저장소 등록. (별 SPEC, v0.2.0+ 후보)
- N4. self-updater 구현. (SPEC-V3-011 MS-3 의 T20 영역)
- N5. Sigstore / cosign 도입. (OS Gatekeeper / SmartScreen 미인정)
- N6. Linux Flatpak 등록. (Flathub 의 자체 빌드 프로세스 + manifest 검토 부담. 별 SPEC v0.2.0+ 후보)

---

## 6. User Stories

- **US-DIST-1**: macOS 사용자가 `brew install --cask moai-studio` 1회로 설치 후 Spotlight / Launchpad 에서 즉시 실행 가능. Gatekeeper 경고 없음.
- **US-DIST-2**: Windows 사용자가 `scoop install moai-studio` 1회로 설치 후 시작 메뉴에서 즉시 실행. SmartScreen 경고 없음.
- **US-DIST-3**: Arch Linux 사용자가 `yay -S moai-studio-bin` 으로 설치 후 desktop entry (`moai-studio.desktop`) 가 자동 등록되어 GUI 메뉴에서 실행 가능.
- **US-DIST-4**: 그 외 Linux 배포판 사용자가 GitHub Releases 에서 `.AppImage` 다운로드 → `chmod +x moai-studio.AppImage` → 더블클릭으로 실행. 별 의존성 없음.
- **US-DIST-5**: 신규 contributor 가 README "Installation" 섹션을 읽고 자기 OS 의 설치 명령을 즉시 발견.
- **US-DIST-6**: maintainer 가 새 release tag (v0.1.x) 를 publish 하면 cask / scoop manifest 가 자동 또는 1 PR 클릭 으로 갱신.

---

## 7. Requirement Groups

본 섹션의 요구사항은 5 개 그룹 (RG-DIST-1 ~ RG-DIST-5) 으로 조직되며, 각 그룹은 MS-1 / MS-2 / MS-3 중 하나 이상에 매핑된다.

### RG-DIST-1: macOS Homebrew Cask (MS-1)

| ID | Type | EARS (한국어) | EARS (English) |
|----|------|---------------|----------------|
| REQ-DIST-001 | Ubiquitous | 시스템은 `dist/homebrew/Casks/moai-studio.rb` 에 Homebrew Cask formula 를 제공한다. formula 는 `version`, `sha256`, `url`, `name`, `desc`, `homepage`, `app` stanza 를 포함한다. | The system **shall** provide a Homebrew Cask formula at `dist/homebrew/Casks/moai-studio.rb` containing version, sha256, url, name, desc, homepage, and app stanzas. |
| REQ-DIST-002 | Ubiquitous | 시스템은 cask formula 의 `url` 이 GitHub Releases 의 `moai-studio-mac-{universal,aarch64,x86_64}.dmg` 또는 `.app.tar.gz` 를 가리키며, `version` 변수로 tag 와 동기화 가능하도록 한다. | The system **shall** ensure the cask `url` points to a GitHub Releases macOS artifact and is parameterized by `version`. |
| REQ-DIST-003 | Event-Driven | 사용자가 `brew install --cask moai-studio` 를 실행하면, 시스템은 quarantine 속성이 자동 제거된 상태로 `/Applications/moai-studio.app` 를 설치한다 (Cask 의 기본 동작). | When the user runs `brew install --cask moai-studio`, the system **shall** install the application with quarantine removed. |
| REQ-DIST-004 | Optional | USER-DECISION-DIST-A 가 (b) homebrew-cask 본가 PR 로 결정될 경우, 시스템은 `homebrew/homebrew-cask` 저장소에 PR 을 제출한다. (a) custom tap 결정 시 `modu-ai/homebrew-tap` 저장소를 신설한다. | Where USER-DECISION-DIST-A is resolved (b), the system **shall** open a PR to homebrew/homebrew-cask; (a) it **shall** create modu-ai/homebrew-tap. |

### RG-DIST-2: Windows Scoop Bucket (MS-2)

| ID | Type | EARS (한국어) | EARS (English) |
|----|------|---------------|----------------|
| REQ-DIST-010 | Ubiquitous | 시스템은 `dist/scoop/moai-studio.json` 에 Scoop manifest 를 제공한다. manifest 는 `version`, `description`, `homepage`, `license`, `url`, `hash`, `bin`, `shortcuts` 필드를 포함한다. | The system **shall** provide a Scoop manifest at `dist/scoop/moai-studio.json` containing version, description, homepage, license, url, hash, bin, shortcuts fields. |
| REQ-DIST-011 | Ubiquitous | 시스템은 manifest 의 `url` 이 GitHub Releases 의 `moai-studio-windows-x64.msi` 또는 `.zip` 을 가리키도록 한다. | The system **shall** ensure the manifest `url` points to the Windows artifact on GitHub Releases. |
| REQ-DIST-012 | Event-Driven | 사용자가 `scoop install moai-studio` 를 실행하면, 시스템은 SmartScreen MOTW 우회 + PATH 등록 + 시작 메뉴 shortcut 까지 자동 처리한다 (Scoop 기본 동작). | When the user runs `scoop install moai-studio`, the system **shall** complete MOTW bypass + PATH registration + Start Menu shortcut. |
| REQ-DIST-013 | Optional | USER-DECISION-DIST-B 가 (b) scoop-extras 본가 PR 로 결정될 경우, 시스템은 `ScoopInstaller/Extras` 저장소에 PR 을 제출한다. (a) custom bucket 결정 시 `modu-ai/scoop-bucket` 저장소를 신설한다. | Where USER-DECISION-DIST-B is resolved (b), the system **shall** open a PR to ScoopInstaller/Extras; (a) it **shall** create modu-ai/scoop-bucket. |

### RG-DIST-3: Linux AUR + AppImage (MS-3)

| ID | Type | EARS (한국어) | EARS (English) |
|----|------|---------------|----------------|
| REQ-DIST-020 | Ubiquitous | 시스템은 `dist/aur/PKGBUILD` 에 Arch Linux 용 PKGBUILD 를 제공한다. PKGBUILD 는 `pkgname=moai-studio-bin`, `pkgver`, `source` (GitHub Releases `.tar.gz` 또는 `.deb` 추출), `package()` 함수에서 `/usr/bin/moai-studio` + `.desktop` 파일 + 256x256 icon 설치를 수행한다. | The system **shall** provide an Arch Linux PKGBUILD at `dist/aur/PKGBUILD` with `pkgname=moai-studio-bin`, `pkgver`, `source`, and `package()` installing the binary, .desktop, and icon. |
| REQ-DIST-021 | Ubiquitous | 시스템은 `dist/aur/.SRCINFO` 를 PKGBUILD 와 동기화하여 제공한다 (AUR 등록 필수 파일). | The system **shall** provide `.SRCINFO` synchronized with PKGBUILD. |
| REQ-DIST-022 | Event-Driven | 사용자가 `yay -S moai-studio-bin` 또는 `paru -S moai-studio-bin` 을 실행하면, 시스템은 binary 다운로드 + checksum 검증 + 시스템 통합 (PATH + .desktop + icon) 을 완료한다. | When the user runs `yay -S moai-studio-bin`, the system **shall** complete download + checksum + integration. |
| REQ-DIST-023 | Ubiquitous | 시스템은 `dist/appimage/README.md` 에 `.AppImage` 다운로드 + chmod +x + 실행 + (옵션) AppImageLauncher 등록 안내를 제공한다. | The system **shall** provide an `.AppImage` usage guide at `dist/appimage/README.md`. |
| REQ-DIST-024 | Optional | 시스템은 `.AppImage` 에 zsync 정보 (`moai-studio.AppImage.zsync`) 를 함께 publish 하여 AppImageLauncher 의 자동 업데이트 기능을 활용 가능하도록 한다. | The system **shall** publish a zsync file alongside `.AppImage`. |

### RG-DIST-4: README Documentation (MS-1)

| ID | Type | EARS (한국어) | EARS (English) |
|----|------|---------------|----------------|
| REQ-DIST-030 | Ubiquitous | 시스템은 `README.md` 의 첫 H2 직후 또는 "Installation" 섹션에 4 채널 (Homebrew, Scoop, AUR, AppImage) 별 1-line 설치 명령을 명시한다. | The system **shall** add an "Installation" section to README.md with 1-line commands per channel. |
| REQ-DIST-031 | Ubiquitous | 시스템은 README 에 "Manual Download (GitHub Releases)" 하위 섹션을 두고 macOS 사용자에게 `xattr -dr com.apple.quarantine /Applications/moai-studio.app` 우회 명령 + Windows 사용자에게 SmartScreen "More info → Run anyway" 안내를 제공한다. | The system **shall** include a "Manual Download" subsection with macOS quarantine bypass and Windows SmartScreen instructions. |

### RG-DIST-5: Release-time Automation (MS-1, MS-2, MS-3)

| ID | Type | EARS (한국어) | EARS (English) |
|----|------|---------------|----------------|
| REQ-DIST-040 | Optional | USER-DECISION-DIST-C 가 (a) 자동화로 결정될 경우, 시스템은 `.github/workflows/release.yml` 에 `homebrew-bump-cask-pr` action 호출 step 을 추가하여 새 tag publish 시 cask formula 의 `version` / `sha256` 을 자동 PR 로 갱신한다. | Where USER-DECISION-DIST-C is (a), the system **shall** add a `homebrew-bump-cask-pr` step to release.yml. |
| REQ-DIST-041 | Optional | USER-DECISION-DIST-C 가 (a) 자동화로 결정될 경우, 시스템은 `.github/workflows/release.yml` 에 Scoop manifest 갱신 step 을 추가한다 (예: `scoop-extras` PR 자동 생성 또는 `modu-ai/scoop-bucket` 직접 commit). | Where USER-DECISION-DIST-C is (a), the system **shall** add a Scoop manifest bump step to release.yml. |

---

## 8. Milestones

### MS-1: macOS Homebrew Cask + README (REQ-DIST-001 ~ 004, REQ-DIST-030 ~ 031, RG-DIST-1, RG-DIST-4)

- 시연 가능 상태: `brew install --cask` (custom tap 또는 본가 PR) 으로 macOS 설치 후 즉시 실행. README "Installation" 섹션에 brew 명령 명시.
- 관련 AC: AC-DIST-1, AC-DIST-2, AC-DIST-7

### MS-2: Windows Scoop Bucket (REQ-DIST-010 ~ 013, RG-DIST-2)

- 시연 가능 상태: `scoop install moai-studio` 으로 Windows 설치 후 시작 메뉴에서 실행. README 에 scoop 명령 명시.
- 관련 AC: AC-DIST-3, AC-DIST-4

### MS-3: Linux AUR + AppImage README (REQ-DIST-020 ~ 024, RG-DIST-3)

- 시연 가능 상태: `yay -S moai-studio-bin` 으로 Arch 설치 + AppImage README 안내 동작.
- 관련 AC: AC-DIST-5, AC-DIST-6, AC-DIST-8

추가: USER-DECISION-DIST-C 에 따라 RG-DIST-5 (release 자동화) 가 MS-1/2/3 각각의 일부로 포함될 수 있음.

---

## 9. Acceptance Criteria

| ID | 기준 | 검증 방법 | 매핑 REQ |
|----|------|----------|---------|
| AC-DIST-1 | `brew install --cask moai-studio` 명령이 macOS 14+ 에서 추가 클릭 없이 정상 설치되고 `open -a moai-studio` 로 실행 가능 | macOS 14 / 15 VM 또는 호스트에서 직접 검증 | REQ-DIST-001/002/003 |
| AC-DIST-2 | cask formula 가 `brew style --cask dist/homebrew/Casks/moai-studio.rb` 통과 | brew style 자동 검증 | REQ-DIST-001 |
| AC-DIST-3 | `scoop install moai-studio` 명령이 Windows 11 에서 SmartScreen 경고 없이 정상 설치되고 시작 메뉴에서 실행 가능 | Windows 11 VM 검증 | REQ-DIST-010/011/012 |
| AC-DIST-4 | scoop manifest 가 `Test-Json -Path dist/scoop/moai-studio.json` 통과 + JSON schema 검증 | PowerShell 자동 검증 | REQ-DIST-010 |
| AC-DIST-5 | `yay -S moai-studio-bin` 또는 `paru -S moai-studio-bin` 명령이 Arch Linux 에서 정상 설치되고 메뉴에서 실행 가능 | Arch VM 검증 + makepkg 로컬 검증 | REQ-DIST-020/021/022 |
| AC-DIST-6 | `chmod +x moai-studio-*.AppImage && ./moai-studio-*.AppImage` 명령이 Ubuntu 22.04 / Fedora 40 에서 정상 실행 | Ubuntu / Fedora VM 검증 | REQ-DIST-023 |
| AC-DIST-7 | `README.md` 의 "Installation" 섹션에 4 채널 + Manual Download 안내가 명시됨 | grep / md preview 검토 | REQ-DIST-030/031 |
| AC-DIST-8 | `dist/aur/.SRCINFO` 가 PKGBUILD 와 동기화 (`makepkg --printsrcinfo` 출력 일치) | makepkg 자동 검증 | REQ-DIST-021 |
| AC-DIST-9 | (옵션, USER-DECISION-DIST-C (a) 결정 시) 새 tag publish 후 cask + scoop PR 또는 commit 이 7 분 내 자동 생성됨 | release.yml run log 확인 | REQ-DIST-040/041 |

---

## 10. USER-DECISION

### 10.1 USER-DECISION-DIST-A — macOS Homebrew Cask 등록 위치 (MS-1 진입)

**상태**: **RESOLVED** (2026-04-27 sess 6). 결정값: **(a) `modu-ai/homebrew-tap` custom tap 신설**.

옵션:

- (a) **선택 — `modu-ai/homebrew-tap` custom tap 신설**: 사용자는 `brew tap modu-ai/tap && brew install --cask moai-studio` 2 명령. 즉시 publish 가능. 통제 100%. 본가 audit 거절 위험 0. v0.1.0 빠른 출시 목적에 부합.
- (b) `homebrew/homebrew-cask` 본가 PR: 사용자는 `brew install --cask moai-studio` 1 명령. 마찰 0. 본가 review 시간 (수일~수주) + cask audit 기준 (versioned URL + license + 업스트림 active 등) 만족 필요.

**근거**: v0.1.0 critical path 에서 즉시 publish + 통제 우선. 본가 PR 은 v0.1.x 안정화 + 본가 audit 정책 검증 후 v0.2.x 시점 재논의 (별 SPEC).

**기록 위치**: 본 spec.md (이 섹션). plan.md 의 RESOLVED 표기 동기화 필요.

### 10.2 USER-DECISION-DIST-B — Windows Scoop Bucket 등록 위치 (MS-2 진입)

**상태**: **RESOLVED** (2026-04-27 sess 6). 결정값: **(a) `modu-ai/scoop-bucket` custom bucket 신설**.

옵션:

- (a) **선택 — `modu-ai/scoop-bucket` custom bucket 신설**: 사용자는 `scoop bucket add moai https://github.com/modu-ai/scoop-bucket && scoop install moai-studio` 2 명령. 통제 100%. 즉시 publish.
- (b) `ScoopInstaller/Extras` 본가 PR: 사용자는 `scoop install moai-studio` 1 명령. 본가 review 시간 필요.

**근거**: DIST-A 와 동일 — 즉시 publish + 통제 우선. v0.2.x 시점 본가 PR 재논의.

### 10.3 USER-DECISION-DIST-C — Release-time 자동화 수준 (MS-1/2/3 공통)

**상태**: **RESOLVED** (2026-04-27 sess 6). 결정값: **(a) 자동화 (release.yml step 추가)**.

옵션:

- (a) **선택 — 자동화 (release.yml step 추가)**: 새 tag publish 시 GitHub Actions 가 cask / scoop / AUR 모두 자동 PR 생성 또는 commit. maintainer 부담 0. 초기 setup 1회. `homebrew-bump-cask-pr@v3` 등 외부 action 사용.
- (b) 수동: 매 release 마다 maintainer 가 cask / scoop / AUR PKGBUILD 의 `version` / `sha256` 직접 수정. setup 부담 0. release 마다 작업 부담.

**근거**: v0.1.x patch 빈번 발생 시 manifest 동기화 누락 방지. RG-DIST-5 (REQ-DIST-040, REQ-DIST-041) 가 본 SPEC 범위로 확정 진입. AC-DIST-9 (자동화 검증 AC) 활성.

---

## 11. Risks

- **R-DIST-1**: Homebrew Cask 본가 audit 거절 — `version` 이 versioned URL 이 아니거나 license 가 audit 정책에 부합하지 않으면 본가 PR 거절. 완화: 본가 도전 전 custom tap 으로 운영하면서 cask 정책 충족 검증.
- **R-DIST-2**: Scoop manifest 의 hash 불일치 — `.msi` 또는 `.zip` 의 SHA256 이 manifest 와 다르면 install 실패. 완화: release.yml 자동화 step 으로 빌드 직후 hash 추출.
- **R-DIST-3**: AUR 계정 부재 — AUR upload 는 SSH key 등록된 AUR 계정 필요. maintainer 1 명이 계정 등록 prerequisite. 완화: AUR 계정 생성은 무료 + 1회.
- **R-DIST-4**: AppImage Linux 의존성 누락 — `libxkbcommon`, `libfontconfig` 등 GPUI runtime 의존성이 일부 distribution 에서 부재 가능. 완화: AppImage 가 portable 의존성 번들. linuxdeploy 의 plugin 으로 자동 처리.
- **R-DIST-5**: brew cask `postflight` 의 quarantine 자동 제거가 macOS 보안 정책 변경으로 미래 동작 변경 가능 — Apple 의 정책 추적 + Cask community 의 patch 추종.
- **R-DIST-6**: scoop manifest 의 SmartScreen 우회가 Microsoft Defender 정책 변경 시 영향 — Scoop community 의 issue 추적.
- **R-DIST-7**: 본 SPEC 의 자동화 (USER-DECISION-DIST-C (a)) 가 실패하면 release tag publish 후 사용자가 stale version 의 cask/scoop 으로 설치할 위험. 완화: release.yml 의 cask/scoop step 실패 시 maintainer 알림 + 수동 fix.

---

## 12. References

- 관련 SPEC: SPEC-V3-011 MS-1 (cross-platform unsigned 빌드 인프라 — 본 SPEC 의 입력)
- 관련 결정: USER-DECISION-PK-B (b) 미보유 (2026-04-27 sess 5)
- 관련 메모리: `project_v0_1_0_release_path.md` (Path B + cert (b) 결정 + 본 SPEC 후보 명시)
- Homebrew Cask 작성 가이드: <https://docs.brew.sh/Cask-Cookbook>
- Scoop manifest reference: <https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests>
- AUR submission guide: <https://wiki.archlinux.org/title/AUR_submission_guidelines>
- AppImage 가이드: <https://docs.appimage.org/packaging-guide/index.html>
- Anthropic Claude Desktop brew cask 사례: `brew search claude` 로 확인 가능 (cask `claude`)

---

Version: 1.1.0 (ready)
Last Updated: 2026-04-27
Status: ready (USER-DECISION-DIST-A/B/C 모두 RESOLVED — 2026-04-27 sess 6)
REQ coverage: REQ-DIST-001 ~ 004, REQ-DIST-010 ~ 013, REQ-DIST-020 ~ 024, REQ-DIST-030 ~ 031, REQ-DIST-040 ~ 041 (총 14 건)
AC coverage: AC-DIST-1 ~ 9 (총 9 건)
Resolved decisions: DIST-A=(a) modu-ai/homebrew-tap, DIST-B=(a) modu-ai/scoop-bucket, DIST-C=(a) release.yml automation
