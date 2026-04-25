---
id: SPEC-V3-011
version: 1.0.0
status: draft
created_at: 2026-04-25
updated_at: 2026-04-25
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-003, SPEC-V3-004, SPEC-V3-005, SPEC-V3-006, SPEC-V3-008, SPEC-V3-009, SPEC-V3-010]
parallel_with: []
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [phase-3, release, packaging, distribution, auto-update, ci, infrastructure]
revision: v1.0.0 (initial draft, v0.1.0 release infrastructure)
---

# SPEC-V3-011: Cross-platform Packaging & Auto-update — macOS .dmg / Linux .deb+.AppImage / Windows .msi + GitHub Releases JSON manifest 자동 업데이트

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-25 | 초안 작성. moai-studio v0.1.0+ 정식 릴리스의 외부 빌드/배포 인프라 정의. RG-PK-1 ~ RG-PK-7, AC-PK-1 ~ AC-PK-12, MS-1/MS-2/MS-3, USER-DECISION 4 게이트. v3 functional SPEC (V3-001 ~ V3-010) 모두 선행. CI billing 해소 + 서명 인증서 보유는 외부 차단으로 명시. 코드베이스 무변경 (외부 빌드 인프라 한정). Enhanced GitHub Flow (CLAUDE.local.md §1, §5) 와 정합. |

---

## 1. 개요

### 1.1 목적

moai-studio 가 사용자에게 도달하는 마지막 km — macOS / Linux / Windows 3-platform 의 패키지 빌드 / 코드 서명 / Apple notarization / 배포 채널 / 자동 업데이트 인프라를 정의한다.

본 SPEC 의 산출물은:

- macOS 사용자가 Apple notarized `.dmg` 를 다운로드하여 drag-to-Applications 만으로 설치한다 (Gatekeeper 통과).
- Linux 사용자가 `.deb` (Ubuntu/Debian) 또는 `.AppImage` (배포 무관) 중 선호하는 형식을 선택한다.
- Windows 사용자가 EV-signed `.msi` 를 SmartScreen 경고 없이 설치한다.
- 앱 실행 시 자동으로 update manifest (GitHub Releases JSON) 을 polling 하고, 새 버전 발견 시 사용자 동의 후 in-place upgrade.
- 빌드 인프라가 GitHub Actions 만으로 재현 가능 (private CI 의존성 없음).

본 SPEC 은 v0.1.0 정식 릴리스의 hard prerequisite 이다.

### 1.2 Enhanced GitHub Flow 와의 정합

CLAUDE.local.md §1 의 branch model (`main` / `release/v{x.y.z}` / `develop` / `feature/*` / `hotfix/*`), §5 의 Release Drafter (CHANGELOG draft 누적), §6.2 의 Release 준비 워크플로 (release branch → main + tag + Release Drafter publish) 와 직접 정합한다.

본 SPEC 의 release.yml 은 `git tag v{x.y.z}` 푸시 시 자동 트리거되며, hotfix (`hotfix/v0.1.1-{slug}`) 흐름의 tag 도 동일하게 인식한다.

### 1.3 근거 문서

- `.moai/specs/SPEC-V3-011/research.md` — 코드베이스 분석, 플랫폼별 packaging 옵션, USER-DECISION 게이트, AC 후보, 위험.
- `CLAUDE.local.md` §1 (branch model), §5 (Release Drafter), §6 (워크플로 체크리스트), §8 (v0.1.0 임시 규칙).
- 외부 표준 — Apple notarytool, cargo-bundle, cargo-deb, cargo-wix, linuxdeploy, signtool, ed25519-dalek.

---

## 2. 배경 및 동기

본 섹션의 상세는 `.moai/specs/SPEC-V3-011/research.md` §1 ~ §3 참조. 최소 맥락만 요약한다.

- **v0.1.0 release 마지막 km** (research §1.1): moai-studio 가 사용자에게 도달하기 위한 외부 인프라. functional SPEC (V3-001 ~ V3-010) 의 가치는 이 SPEC 이 PASS 해야 사용자에게 전달된다.
- **Apple Gatekeeper / Windows SmartScreen 진입장벽** (research §3 / §5): 서명 + notarization 없이는 사용자가 "확인되지 않은 개발자" 경고 우회 필요. 진입장벽 = 사용 포기.
- **자동 업데이트 부재 시 위험** (research §6): 사용자가 신버전 다운로드를 누락하면 보안 패치 미적용. 자동 업데이트는 v0.1.0 단계 필수.
- **외부 차단 의식** (research §12): CI billing 해소 + 서명 인증서 보유 = release-ready 의 사전 전제.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. macOS universal `.app` (arm64 + x86_64) 가 GitHub Actions `macos-14` runner 에서 빌드된다.
- G2. macOS `.app` 이 Apple Developer ID 로 서명되고 Apple notarytool 로 공증되며 stapler 로 staple 된다.
- G3. macOS `.dmg` 가 `create-dmg` 로 생성되고 사용자 drag-to-Applications UX 를 지원한다.
- G4. Linux `.deb` (Ubuntu/Debian) 가 `cargo-deb` 로 생성되며 `dpkg-deb --info` 검증을 통과한다.
- G5. Linux `.AppImage` 가 `linuxdeploy` 로 생성되며 dynamic library bundling 을 포함한다.
- G6. Windows `.msi` 가 `cargo-wix` 로 생성되고 EV 인증서로 `signtool` 서명된다.
- G7. update manifest (`update.json`) 가 release tag 시점에 자동 생성되며 sha256 + Ed25519 서명을 포함한다.
- G8. 앱 실행 시 24h 주기 (또는 시작 시) update.json 을 polling 하며 새 버전 발견 시 사용자 동의 UI 를 표시한다.
- G9. 사용자 동의 시 in-place update — macOS `.app` 교체 / Windows `msiexec quiet` / Linux AppImage self-replace (또는 deb 사용자 안내).
- G10. `.github/workflows/release.yml` 이 `v{x.y.z}` tag push 시 모든 platform artifact 를 빌드 → 서명 → notarize → GitHub Releases attach 까지 자동화한다.
- G11. Release Drafter (CLAUDE.local.md §5) draft 가 release tag 시점에 자동 publish 된다.
- G12. moai-studio 코드베이스 (crates/) 가 변경되지 않는다 — 본 SPEC 은 외부 빌드 인프라 + 신규 update module 만 추가.

### 3.2 비목표 (Non-Goals)

- N1. **Linux ARM64 (aarch64) 빌드** — v0.1.0 단계 비목표. v0.2.0+ 후보.
- N2. **macOS App Store 배포** — Mac App Store 는 별도 Sandbox 요건 + 추가 entitlements. 본 SPEC 비목표.
- N3. **Microsoft Store 배포 (.msix)** — Microsoft Store + MSIX 패키징 별 SPEC.
- N4. **Linux apt repository 운영 (PPA / OBS)** — GitHub Releases 직접 다운로드만. PPA 별 SPEC.
- N5. **delta update (binary diff)** — full download 만. v0.1.0 단계 5~20MB 적정.
- N6. **crash reporting (Sentry)** — USER-DECISION-PK-D default opt-out. 별 SPEC.
- N7. **자동 silent update** — 항상 사용자 동의 prompt. 무동의 update 금지 (보안/UX).
- N8. **moai-studio 코드베이스 변경** — RG-P-7 carry from V3-002 ~ V3-009. 본 SPEC 은 `.github/workflows/release.yml` + `Cargo.toml` 메타데이터 + 신규 update 모듈만 변경.
- N9. **i18n 패키지 분리** — 단일 universal artifact 만. 다국어는 런타임에서.
- N10. **portable mode (config 외부 위치)** — 표준 설치 location (Applications / Program Files / /opt). portable 별 SPEC.
- N11. **Linux ARM (Raspberry Pi)** — N1 carry.
- N12. **release notes 자동 번역** — Release Drafter 영문 그대로. 다국어 별 SPEC.

---

## 4. 사용자 스토리

- **US-PK-1**: 사용자가 GitHub Releases 페이지에서 macOS 사용자라면 `.dmg` 를, Linux 사용자라면 `.deb` 또는 `.AppImage` 를, Windows 사용자라면 `.msi` 를 받는다 → release.yml 가 모든 artifact 를 attach.
- **US-PK-2**: 사용자가 macOS `.dmg` 를 더블클릭하면 mount 후 drag-to-Applications UX 가 보인다 → create-dmg 의 background image + drop link.
- **US-PK-3**: 사용자가 macOS `.app` 을 처음 실행해도 "확인되지 않은 개발자" 경고가 보이지 않는다 → Apple notarized + stapled.
- **US-PK-4**: 사용자가 Windows `.msi` 를 더블클릭해도 SmartScreen 경고가 보이지 않는다 → EV signed.
- **US-PK-5**: 사용자가 Linux 에서 `dpkg -i moai-studio_0.1.0_amd64.deb` 로 설치하면 `/usr/bin/moai-studio` 와 `.desktop` 파일이 등록된다 → cargo-deb assets.
- **US-PK-6**: 사용자가 Linux `.AppImage` 를 chmod +x 후 더블클릭하면 별도 의존 설치 없이 실행된다 → linuxdeploy bundling.
- **US-PK-7**: 사용자가 앱을 실행하면 백그라운드에서 update.json 을 polling 하고, 새 버전 발견 시 in-app notification 을 받는다 → update polling + 24h cache.
- **US-PK-8**: 사용자가 update prompt 의 "지금 업데이트" 를 클릭하면 신버전이 다운로드 → 서명 검증 → in-place 교체된다 → ed25519 verify + plat-specific update path.
- **US-PK-9**: 사용자가 `.deb` 사용자라면 update prompt 가 "수동 설치 필요" 안내를 보여준다 → admin 권한 부재 graceful 안내.
- **US-PK-10**: 개발자가 `git tag v0.1.0 && git push origin v0.1.0` 만 실행하면 release.yml 이 30~60분 후 모든 platform artifact 를 GitHub Release 에 attach 한다 → tag push trigger workflow.
- **US-PK-11**: 개발자가 release publish 직후 GitHub Release notes 에 Release Drafter CHANGELOG (Added/Fixed/Security 분류) 가 자동 채워진 것을 본다 → release-drafter publish trigger.

---

## 5. 기능 요구사항 (EARS)

### RG-PK-1 — Cargo workspace → 3-platform binary 빌드

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-001 | Ubiquitous | 시스템은 GitHub Actions matrix 로 다음 4 빌드를 병렬 실행한다: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc. | The system **shall** run a 4-target build matrix in GitHub Actions: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc. |
| REQ-PK-002 | Ubiquitous | 시스템은 워크스페이스 `rust-toolchain` 의 Rust 버전을 모든 빌드 job 에서 동일 사용한다. | The system **shall** use the workspace `rust-toolchain` Rust version uniformly across all build jobs. |
| REQ-PK-003 | Event-Driven | macOS 빌드가 두 target (arm64 + x86_64) 모두 성공하면, 시스템은 `lipo -create` 로 universal binary 를 생성한다. | When both macOS targets succeed, the system **shall** create a universal binary via `lipo -create`. |
| REQ-PK-004 | Ubiquitous | 시스템은 release tag (`v{x.y.z}`) 의 semver 와 `Cargo.toml` `[workspace.package].version` 이 일치하는지 검증한다. 불일치 시 빌드 fail. | The system **shall** verify the release tag's semver matches `Cargo.toml`'s `[workspace.package].version`. Mismatch fails the build. |
| REQ-PK-005 | Unwanted | 시스템은 Linux ARM64 또는 Windows ARM64 빌드를 시도하지 않는다 (N1 / N2 carry). | The system **shall not** attempt Linux ARM64 or Windows ARM64 builds (N1 / N2 carry). |

### RG-PK-2 — macOS .dmg + notarization + stapling

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-010 | Ubiquitous | 시스템은 universal binary 를 `.app` bundle 구조 (Contents/MacOS/, Contents/Info.plist, Contents/Resources/icon.icns) 로 패키징한다. | The system **shall** package the universal binary into a `.app` bundle (Contents/MacOS/, Info.plist, Resources/icon.icns). |
| REQ-PK-011 | State-Driven | 서명 인증서 (USER-DECISION-PK-B 의 (a) 결정) 가 secret 으로 가용한 동안, 시스템은 `codesign --force --deep --sign "Developer ID Application: <Team>" --options runtime` 으로 `.app` 을 서명한다. | While the signing certificate (USER-DECISION-PK-B (a)) is available as secret, the system **shall** codesign the `.app` with `--options runtime`. |
| REQ-PK-012 | Event-Driven | `.app` 서명이 완료되면, 시스템은 `xcrun notarytool submit ... --wait` 로 Apple 공증을 요청하고 응답을 대기한다 (timeout 60분). | When `.app` signing completes, the system **shall** request Apple notarization via `xcrun notarytool submit ... --wait` (60min timeout). |
| REQ-PK-013 | Event-Driven | notarization 이 통과하면, 시스템은 `xcrun stapler staple` 로 ticket 을 `.app` 에 embed 한다. | When notarization passes, the system **shall** staple the ticket via `xcrun stapler staple`. |
| REQ-PK-014 | Ubiquitous | 시스템은 `create-dmg` 로 `.dmg` 를 생성하며 background image + drag-to-Applications symlink 를 포함한다. | The system **shall** generate `.dmg` via `create-dmg` with background image and drag-to-Applications symlink. |
| REQ-PK-015 | Unwanted | 서명 인증서가 없는 동안, 시스템은 production release artifact 를 publish 하지 않는다 (MS-1 dry-run 한정). | While no signing certificate is available, the system **shall not** publish production release artifacts (MS-1 dry-run only). |

### RG-PK-3 — Linux .deb + .AppImage + GPG signing (옵션)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-020 | Ubiquitous | 시스템은 `cargo-deb` 로 `.deb` 패키지를 생성하며 `Cargo.toml` `[package.metadata.deb]` 의 maintainer / depends / assets 설정을 준수한다. | The system **shall** generate `.deb` via `cargo-deb` honoring `[package.metadata.deb]` settings. |
| REQ-PK-021 | Ubiquitous | `.deb` 패키지는 `/usr/bin/moai-studio` (binary), `/usr/share/applications/moai-studio.desktop`, `/usr/share/icons/hicolor/256x256/apps/moai-studio.png` 를 포함한다. | The `.deb` **shall** include `/usr/bin/moai-studio`, `.desktop`, and icon assets. |
| REQ-PK-022 | Ubiquitous | 시스템은 `linuxdeploy` 로 `.AppImage` 를 생성하며 dynamic library (libfontconfig 등) 를 AppDir 에 자동 bundling 한다. | The system **shall** generate `.AppImage` via `linuxdeploy` with automatic dynamic library bundling. |
| REQ-PK-023 | State-Driven | GPG key 가 secret 으로 가용한 동안, 시스템은 `.deb` 와 `.AppImage` 에 detached GPG signature (`.asc`) 를 생성한다. | While GPG key is available, the system **shall** generate detached GPG signatures (`.asc`). |
| REQ-PK-024 | Unwanted | 시스템은 `.deb` 의 dependency (libgtk 등) 누락으로 설치 후 실행 실패하는 산출을 publish 하지 않는다. CI 에서 dpkg -i + 실행 smoke 를 수행한다. | The system **shall not** publish a `.deb` that fails post-install execution; CI performs dpkg -i + execution smoke. |

### RG-PK-4 — Windows .msi + signtool + WinUI manifest

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-030 | Ubiquitous | 시스템은 `cargo-wix` 로 `.msi` 인스톨러를 생성하며 `Cargo.toml` `[package.metadata.wix]` 의 upgrade-guid / path-guid 설정을 준수한다. | The system **shall** generate `.msi` via `cargo-wix` honoring `[package.metadata.wix]` settings. |
| REQ-PK-031 | Ubiquitous | `.msi` 는 application manifest 에 `requestedExecutionLevel=asInvoker`, `dpiAware=true/PM`, `supportedOS=Windows 10/11 GUID` 를 포함한다. | The `.msi` **shall** include application manifest with `asInvoker`, `dpiAware=true/PM`, and Windows 10/11 supportedOS GUID. |
| REQ-PK-032 | State-Driven | EV 인증서 (USER-DECISION-PK-B 의 (a) 결정) 가 cloud HSM (DigiCert KeyLocker 등) 으로 가용한 동안, 시스템은 `signtool sign /tr /td sha256 /fd sha256` 로 `.msi` 를 서명한다. | While EV certificate via cloud HSM is available, the system **shall** sign `.msi` via `signtool sign /tr /td sha256 /fd sha256`. |
| REQ-PK-033 | Ubiquitous | `.msi` 는 Start Menu shortcut 과 (옵션) Desktop shortcut 을 등록하며 silent install (`msiexec /i ... /qn`) 을 지원한다. | The `.msi` **shall** register Start Menu shortcut, support silent install (`msiexec /qn`). |
| REQ-PK-034 | Unwanted | 시스템은 self-signed 또는 인증서 없이 production `.msi` 를 publish 하지 않는다 (SmartScreen 경고 우회 = 사용 포기). | The system **shall not** publish production `.msi` without proper EV signing. |

### RG-PK-5 — Auto-updater (GitHub Releases JSON manifest)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-040 | Ubiquitous | 시스템은 매 release tag 시 update manifest `update.json` 을 생성하여 GitHub Release 의 latest 에 attach 한다. schema = `{version, released_at, notes_url, platforms: {macos-universal, linux-x86_64-deb, linux-x86_64-appimage, windows-x86_64-msi}: {url, sha256, signature}}`. | The system **shall** generate `update.json` at each release with the schema above and attach to GitHub Releases latest. |
| REQ-PK-041 | Ubiquitous | 시스템은 update.json 의 각 platform entry 에 sha256 hash 와 Ed25519 detached signature 를 포함한다. | The system **shall** include sha256 and Ed25519 detached signature for each platform entry in update.json. |
| REQ-PK-042 | Event-Driven | 앱이 시작되거나 마지막 polling 후 24h 가 경과하면, 시스템은 `https://github.com/GoosLab/moai-studio/releases/latest/download/update.json` 을 GET 으로 polling 한다. | When the app starts or 24h since last polling elapses, the system **shall** GET the update.json URL. |
| REQ-PK-043 | State-Driven | update.json.version 이 현재 버전보다 semver-newer 인 동안, 앱은 사용자에게 in-app notification (변경사항 + "지금 업데이트" 버튼) 을 표시한다. | While update.json.version is semver-newer, the app **shall** display in-app notification with changes and update button. |
| REQ-PK-044 | Event-Driven | 사용자가 "지금 업데이트" 를 클릭하면, 시스템은 platform 맞는 url 을 다운로드하여 sha256 + Ed25519 서명을 검증한다. 검증 실패 시 abort + 사용자에게 에러 메시지. | When user clicks "Update Now", the system **shall** download platform-matched url, verify sha256 + Ed25519 signature; abort with error on failure. |
| REQ-PK-045 | Event-Driven | 검증 통과 후 시스템은 platform 별 update path 를 실행한다: macOS `.app` 교체 + 재시작 prompt, Windows `msiexec /i ... /qb`, Linux `.AppImage` self-replace, Linux `.deb` 사용자 안내. | When verification passes, the system **shall** execute platform-specific update paths: macOS .app replace, Windows msiexec /qb, Linux AppImage self-replace, Linux deb manual guidance. |
| REQ-PK-046 | Unwanted | 시스템은 사용자 동의 없이 자동 업데이트를 silent 적용하지 않는다 (보안 + UX). | The system **shall not** apply silent automatic updates without user consent. |
| REQ-PK-047 | Unwanted | 시스템은 sha256 또는 Ed25519 서명 검증 실패 시 다운로드된 파일을 실행하지 않는다. 임시 파일은 즉시 삭제. | The system **shall not** execute the downloaded file on sha256 or Ed25519 verification failure; temp files deleted immediately. |
| REQ-PK-048 | State-Driven | Linux `.deb` 사용자가 admin 권한 부재 상태인 동안, 시스템은 자동 update 대신 "수동 설치 필요" 안내 + GitHub Releases 링크를 표시한다. | While Linux .deb user lacks admin rights, the system **shall** display "manual install required" guidance with GitHub Releases link. |

### RG-PK-6 — Release Drafter 통합 (CHANGELOG → Release notes)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-050 | Event-Driven | release.yml 이 모든 platform artifact 를 GitHub Release 에 attach 완료하면, 시스템은 Release Drafter draft 를 publish 트리거한다 (workflow_dispatch 또는 API 호출). | When release.yml finishes attaching artifacts, the system **shall** trigger Release Drafter draft publish. |
| REQ-PK-051 | Ubiquitous | publish 된 release notes 는 Release Drafter 카테고리 (`## Added`, `## Fixed`, `## Security`, `## Performance`, `## Refactored`, `## Documentation`, `## Internal`) 를 그대로 사용한다. | The published release notes **shall** preserve Release Drafter categories. |
| REQ-PK-052 | Ubiquitous | 시스템은 Release Drafter 의 version bump 라벨 (`release/major|minor|patch`) 을 인식하여 다음 tag suggestion 을 표시한다 (CLAUDE.local.md §5.3 carry). | The system **shall** recognize Release Drafter version bump labels for next tag suggestion. |

### RG-PK-7 — CI release workflow (`.github/workflows/release.yml`)

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-PK-060 | Ubiquitous | 시스템은 `.github/workflows/release.yml` 을 신설하며 trigger 는 `on.push.tags: ['v*.*.*']` (USER-DECISION-PK-C 결정에 따라 `v*.*.*-rc*` 추가 가능). | The system **shall** create `.github/workflows/release.yml` with `on.push.tags` trigger. |
| REQ-PK-061 | Ubiquitous | release.yml 의 `build` job 은 4 OS × target matrix (REQ-PK-001) 를 실행하며 `fail-fast: false` 로 parallel 진행한다. | The `build` job **shall** run a 4-OS matrix with `fail-fast: false`. |
| REQ-PK-062 | Ubiquitous | release.yml 의 `release` job 은 모든 build artifact 를 download 하여 macOS lipo + notarize + dmg, Linux deb + AppImage, Windows msi sign 을 수행한 후 `softprops/action-gh-release` 로 attach 한다. | The `release` job **shall** download all build artifacts, perform platform-specific finalization, and attach via `softprops/action-gh-release`. |
| REQ-PK-063 | Event-Driven | tag 가 `v*.*.*-rc*` 패턴인 동안 (USER-DECISION-PK-C (b) 결정 시), 시스템은 GitHub Release 를 `prerelease: true` 로 생성한다. | When tag matches `v*.*.*-rc*` (USER-DECISION-PK-C (b)), the system **shall** create GitHub Release as `prerelease: true`. |
| REQ-PK-064 | Ubiquitous | release.yml 은 `workflow_dispatch` trigger 도 지원하여 dry-run (skip-deploy flag) 검증을 가능하게 한다. CI billing 해소 전까지 dry-run 만 활용. | The release.yml **shall** support `workflow_dispatch` trigger for dry-run validation; only dry-run is used until CI billing is resolved. |
| REQ-PK-065 | Unwanted | 시스템은 moai-studio 코드베이스 (crates/) 를 변경하지 않는다 (G12 carry). 본 SPEC 의 신규 update 모듈은 단일 신규 crate 또는 app crate 의 신규 디렉터리로 격리. | The system **shall not** modify moai-studio crates/ codebase; new update module is isolated to a new crate or new directory in the app crate. |

---

## 6. Acceptance Criteria

| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 |
|------|--------------|----------|----------|---------|
| AC-PK-1 | macOS universal binary 가 `lipo -info` 로 arm64 + x86_64 양쪽 architecture 포함을 보고 | `lipo -info target/universal/release/moai-studio` 출력에 `arm64 x86_64` 또는 `i386 x86_64 arm64` 포함 | CI assertion (release.yml step) | RG-PK-1 |
| AC-PK-2 | `.deb` 패키지가 `dpkg-deb --info` 로 maintainer / depends / version 메타를 정확히 보고 | metadata 의 Package=moai-studio, Version 이 Cargo.toml version 과 일치, Depends 에 libgtk-3-0 등 명시 | CI assertion (lintian 옵션 통과) | RG-PK-3 |
| AC-PK-3 | `.AppImage` 가 `ldd` 검증 시 누락 의존 없이 실행되며 `--appimage-extract-and-run` 으로 smoke 실행 통과 | extract-and-run 의 exit code = 0, GUI window 가 5초 이내 표시 | integration (CI ubuntu-22.04 + ubuntu-24.04 dual) | RG-PK-3 |
| AC-PK-4 | `.msi` 가 Windows 10/11 VM 에서 silent install (`msiexec /i ... /qn`) 후 `Program Files\\moai-studio\\moai-studio.exe` 실행 가능 | install exit code = 0, exe 파일 존재 + 실행 시 main window 표시 | integration (windows-2022 runner) | RG-PK-4 |
| AC-PK-5 | macOS `.app` 이 `codesign --verify --verbose` 로 Developer ID 서명 검증 통과 + `xcrun notarytool history` 에서 Accepted 상태 | codesign 출력 `valid on disk` + notarytool 의 last submission status=Accepted | CI assertion + manual log inspection | RG-PK-2 |
| AC-PK-6 | macOS `.dmg` 가 `xcrun stapler validate` 로 stapled 검증 통과 + offline 머신에서 Gatekeeper 통과 | stapler validate 출력 `The validate action worked!` + 네트워크 차단 머신 첫 실행 시 경고 없음 | CI assertion + manual e2e | RG-PK-2 |
| AC-PK-7 | Windows `.msi` 가 `signtool verify /pa /v` 로 EV 서명 검증 통과 + 깨끗한 Win10/11 VM 첫 실행 시 SmartScreen 경고 없음 | signtool verify 의 exit code = 0 + manual SmartScreen 무경고 | CI assertion + manual e2e | RG-PK-4 |
| AC-PK-8 | release tag 시점에 `update.json` 이 GitHub Release 에 자동 attach + schema 가 REQ-PK-040 의 4-platform entry 모두 포함 + 각 sha256 + Ed25519 signature 유효 | update.json 의 platforms 필드에 4 key + sha256 일치 + ed25519-dalek verify 통과 | CI assertion (release.yml step) | RG-PK-5 |
| AC-PK-9 | 앱이 update.json 을 polling 하여 신버전 발견 시 in-app notification UI 가 5초 이내 표시 | mock update.json 서버 + 앱 시작 + UI 노드 assertion | unit (mock HTTP) + e2e | RG-PK-5 |
| AC-PK-10 | 사용자 동의 후 in-place update — macOS .app 교체 후 신버전 실행, Windows msiexec quiet 후 신버전, Linux AppImage self-replace 후 신버전, Linux .deb 시 안내 메시지 표시 | 4 platform 각각 신버전 실행 (version string 비교) + .deb 의 경우 GitHub Releases 링크 노출 | integration (4 platform e2e) | RG-PK-5 |
| AC-PK-11 | release.yml 이 `git tag v0.1.0-test && git push` (테스트 tag) 시 4-platform artifact 를 모두 빌드 → 서명 → notarize → GitHub Release 에 attach 까지 60분 이내 완료 | workflow run 의 모든 job success + GitHub Release 의 attached artifact 4 개 모두 존재 | integration (실 release dry-run) | RG-PK-7 |
| AC-PK-12 | Release Drafter draft 가 release tag publish 직후 자동 publish + release notes 에 Added/Fixed/Security 카테고리가 PR 라벨대로 채워짐 | publish 된 release 의 body 에 Release Drafter 템플릿 일치 + 최소 1 PR 항목 포함 | integration (manual + workflow log) | RG-PK-6 |

---

## 7. 비기능 요구사항

| 항목 | 요구 |
|------|------|
| 빌드 시간 (release tag → 모든 artifact attach) | 60분 이내 (notarization wait 포함) |
| update polling 빈도 | 24h 주기 (시작 시 1회 + cooldown 24h) |
| update.json 크기 | 4KB 이하 (4 platform entry × 1KB) |
| update download 크기 | macOS dmg ~30MB, Linux AppImage ~40MB, Linux deb ~25MB, Windows msi ~30MB |
| update polling latency | 사용자 인지 가능한 stutter 없음 (background tokio task) |
| 서명 검증 (sha256 + Ed25519) | 100ms 이내 (다운로드 완료 후) |
| OS support | macOS 14+ (Sonoma), Ubuntu 22.04+, Windows 10/11 |
| Rust toolchain | workspace `rust-toolchain` (현행 1.92+) |
| code_comments 언어 | `ko` (`.moai/config/sections/language.yaml`) |
| 코드베이스 변경 | 금지 (G12, RG-PK-7.6) — 외부 빌드 인프라 + 신규 update 모듈만 |
| 외부 차단 | CI billing 해소 + 서명 인증서 보유 (USER-DECISION-PK-B) — implement 진입 prerequisite |

---

## 8. 의존성 / 통합 인터페이스

### 8.1 선행 SPEC

- **SPEC-V3-001 ~ V3-010**: 모든 v3 functional SPEC complete 권장. 본 SPEC 은 packaging/distribution 인프라이므로 코드 안정 후 진입. 진입 임계값 = AC pass count 합산 80%+ AND 사용자가 v0.1.0-rc1 결정.

### 8.2 병행 가능 SPEC

- 없음. 본 SPEC 은 v3 functional SPEC 의 successor 이며, parallel 진행 시 outdated artifact 위험.

### 8.3 외부 의존

- **macOS**: Apple Developer Program ($99/year), Developer ID Application 인증서, Apple ID + app-specific password.
- **Windows**: DigiCert (또는 Sectigo) EV cert + DigiCert KeyLocker (cloud HSM, GitHub Actions 호환).
- **Linux**: optional GPG key (apt repo 운영 시).
- **CI runners**: GitHub Actions `macos-14` / `ubuntu-22.04` (+ optional `ubuntu-24.04`) / `windows-2022`. private repo 비용 (CI billing 해소 전제).
- **Tooling**: cargo-bundle (선택), cargo-deb, cargo-wix, create-dmg (brew/npm), linuxdeploy, signtool (Windows SDK), xcrun notarytool (Xcode 13+).
- **Rust crates (신규)**: ed25519-dalek v2 (서명 검증), reqwest v0.12 또는 ureq v3 (HTTP polling), semver v1 (버전 비교), serde_json v1 (manifest 파싱).

### 8.4 외부 차단 (research §12 carry)

implement 진입은 다음 차단 해소가 전제:

1. CI billing — GitHub Actions private repo macOS runner 비용. 사용자 결정 시까지 release.yml 실 트리거 보류.
2. 서명 인증서 — USER-DECISION-PK-B 의 (a) 결정 시까지 MS-2 진입 불가.
3. Apple Developer Program 등록.
4. DigiCert EV cert 또는 KeyLocker 계약.
5. (옵션) GPG key.

본 SPEC 의 RG / AC 는 위 차단 해소 후의 상태를 정의하되, **MS-1 (unsigned 빌드 검증)** 까지는 위 차단 없이 진입 가능.

---

## 9. 마일스톤 (priority-based, 시간 추정 없음)

### MS-1 (Priority: High) — 3-platform 빌드 + 기본 packaging (서명 없음)

산출:
- `.github/workflows/release.yml` 신설 (build job matrix, dry-run trigger)
- `Cargo.toml` 의 `[package.metadata.bundle]`, `[package.metadata.deb]`, `[package.metadata.wix]` 메타 추가
- macOS unsigned `.app` + universal binary lipo 검증
- Linux unsigned `.deb` + `.AppImage` 생성
- Windows unsigned `.msi` 생성
- USER-DECISION-PK-C 게이트 (tag naming)
- AC-PK-1, AC-PK-2, AC-PK-3, AC-PK-4 통과
- 외부 차단 (CI billing, 서명) 없이 진입 가능

### MS-2 (Priority: High) — 서명 + notarization (서명 인증서 보유 시)

산출:
- macOS codesign + xcrun notarytool + xcrun stapler 통합
- `.dmg` create-dmg 스크립트 + background image
- Windows signtool + DigiCert KeyLocker 통합
- (옵션) Linux GPG signing
- USER-DECISION-PK-B 게이트 (서명 인증서 보유 — P0 차단)
- AC-PK-5, AC-PK-6, AC-PK-7 통과
- **Prerequisite**: 서명 인증서 + Apple Developer Program 등록 + DigiCert KeyLocker 계약

### MS-3 (Priority: High) — Auto-updater + release workflow + 자동화

산출:
- update.json schema 정의 + Ed25519 keypair 생성 (private key → GitHub Actions secret, public key → app embed)
- 신규 crate `crates/moai-studio-updater/` 또는 app crate 의 `update/` 모듈 (단일 신규 격리)
- update polling + UI notification + 사용자 동의 + in-place update path (4 platform)
- release.yml `release` job (artifact aggregation + update.json 생성 + Release Drafter trigger)
- USER-DECISION-PK-A 게이트 (auto-update 메커니즘 — 자체 vs Sparkle)
- AC-PK-8, AC-PK-9, AC-PK-10, AC-PK-11, AC-PK-12 통과
- 코드베이스 무변경 검증 (RG-PK-7.6)

---

## 10. USER-DECISION 게이트

### 10.1 USER-DECISION-PK-A — Auto-update 메커니즘 (MS-3 진입)

질문: "자동 업데이트 메커니즘은?"

옵션:
- (a) **권장: 자체 (GitHub Releases JSON manifest)** — 외부 의존 0, Rust 100~200 LOC, Ed25519 서명 검증. 플랫폼 일관성. moai-studio 의 외부 의존 최소화 원칙 정합.
- (b) Sparkle (macOS) + WinSparkle (Win) + 자체 (Linux) — 검증된 라이브러리. 하지만 Objective-C bridge + 분기 로직.
- (c) skipping auto-update for v0.1.0 — v0.2.0 에서 추가. v0.1.0 사용자는 수동으로 신버전 다운로드.

영향 범위: RG-PK-5 전체, MS-3 산출.

### 10.2 USER-DECISION-PK-B — 서명 인증서 보유 (P0 차단, MS-2 진입)

질문: "macOS Developer ID + Windows EV cert 를 보유했는가?"

옵션:
- (a) **권장: 보유함 (Apple Developer Program $99 + DigiCert EV $300+)** — MS-2 진입 가능. release-ready 산출.
- (b) 보유 안함 — MS-2 차단. MS-1 (unsigned 빌드) 까지만 진행. 사용자 경고: macOS Gatekeeper 우회 (우클릭 열기) 필요, Windows SmartScreen 경고.
- (c) self-signed (개발 단계만) — production 부적격. internal testing 한정.

영향 범위: RG-PK-2 (macOS sign+notarize), RG-PK-4 (Windows sign), MS-2 진입 가능 여부.

[HARD] 본 게이트는 P0 — (b) 또는 (c) 결정 시 MS-2 차단. release-ready 산출 무효화.

### 10.3 USER-DECISION-PK-C — Release tag naming (MS-1 진입)

질문: "릴리스 tag 명명 컨벤션은?"

옵션:
- (a) **권장: `v{x.y.z}` (CLAUDE.local.md §1.2 준수)** — `v0.1.0`, `v0.1.1`. 현재 hotfix 명명 (`hotfix/v0.1.1-{slug}`) 와 직접 매칭. release.yml trigger regex 단순.
- (b) `v{x.y.z}-rc{n}` 추가 — `v0.1.0-rc1` → `v0.1.0`. release candidate 단계 분리. release.yml 가 `-rc*` 는 prerelease=true 로 mark (REQ-PK-063).
- (c) calendar versioning (`2026.04.25`) — semver 포기. 본 프로젝트와 부적합.

영향 범위: REQ-PK-060 (tag trigger regex), REQ-PK-063 (prerelease mark), Release Drafter 정합 (CLAUDE.local.md §5.3).

### 10.4 USER-DECISION-PK-D — Crash reporting (선택, default opt-out)

질문: "크래시 리포팅을 도입할 것인가?"

옵션:
- (a) **권장: opt-out (도입 안함)** — 사용자 프라이버시 우선. v0.1.0 단계 적합. 이슈 리포트는 GitHub Issues 로.
- (b) Sentry SaaS opt-in — 사용자가 명시 동의 시만 전송. $26/month base.
- (c) 자체 crash reporter — `panic_hook` + 로컬 파일 + 사용자 명시 업로드. v0.2.0+ 후보.

영향 범위: 본 SPEC 의 RG 그룹에는 포함 안함 (별 SPEC). 본 게이트는 v0.2.0 SPEC 진입 시 재논의.

---

## 11. 위험 (Risk Register)

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-PK-1 | Apple Notarization 정책 변경 (entitlements 추가 요구) | macOS 빌드 실패 | RG-PK-2 의 entitlements file 명시 + MS-2 entry spike |
| R-PK-2 | Windows EV cert 미보유 → SmartScreen 경고 | 사용자 설치 진입장벽 | USER-DECISION-PK-B (a/b) gate, REQ-PK-034 |
| R-PK-3 | GPUI Metal 의존성 → entitlements 추가 (allow-jit) | 첫 macOS 빌드 실패 | MS-1 spike + entitlements 사전 정의 |
| R-PK-4 | Linux distro fragmentation (Ubuntu LTS 22 vs 24, libwebkit 버전 차) | AppImage 의존 누락 | MS-1 ubuntu-22.04 + ubuntu-24.04 dual matrix (REQ-PK-001 확장 옵션) |
| R-PK-5 | universal binary lipo 실패 (architecture flag 불일치) | macOS 빌드 fail | MS-1 spike + lipo -info 검증 step (AC-PK-1) |
| R-PK-6 | update.json hosting (GitHub Releases throttle) | update polling 빈도 제한 | etag/cache-control 활용 + 24h poll 주기 (REQ-PK-042) |
| R-PK-7 | self-update Windows admin 권한 부재 | silent update 실패 | user-install (`%LOCALAPPDATA%`) 옵션 + 사용자 안내 (REQ-PK-045) |
| R-PK-8 | Ed25519 private key leak | malicious update 가능 | Hardware token (YubiKey) + GitHub Actions secret env-only + key rotation 정책 |
| R-PK-9 | Cargo.toml version vs git tag 불일치 | release artifact 의 버전 메타 오류 | REQ-PK-004 의 verify step (tag regex match Cargo.toml) |
| R-PK-10 | macos-14 runner Xcode 버전 변경 | 빌드 도구 break | `actions/setup-xcode@v1` pin 명시 |
| R-PK-11 | .deb dependency 누락 (libgtk 등 GPUI 의존) | 설치 후 실행 실패 | REQ-PK-024 + AC-PK-3 의 dpkg -i + 실행 smoke |
| R-PK-12 | CI billing 해소 지연 | release workflow 실 트리거 검증 불가 | REQ-PK-064 의 dry-run + workflow_dispatch 로 무release 검증 가능 |
| R-PK-13 | DigiCert KeyLocker 가격 인상 / 정책 변경 | Windows 서명 차단 | USER-DECISION-PK-B 재논의 + alternative HSM (Azure Key Vault) 검토 |
| R-PK-14 | Sparkle 미선택으로 macOS delta update 부재 | 사용자 update 다운로드 시간 ↑ | v0.2.0 SPEC 으로 delta 추가 검토 (USER-DECISION-PK-A 의 (b) 재고) |

---

## 12. 외부 인터페이스 (불변 약속)

본 SPEC 은 다음 인터페이스를 fix 한다. 후속 SPEC 이 본 SPEC 의 산출물을 consume 할 때 신뢰할 수 있다:

```rust
// crates/moai-studio-updater/src/lib.rs (개념적 export)

pub struct UpdateManifest {
    pub version: semver::Version,
    pub released_at: chrono::DateTime<chrono::Utc>,
    pub notes_url: url::Url,
    pub platforms: HashMap<PlatformKey, UpdateEntry>,
}

pub enum PlatformKey {
    MacosUniversal,
    LinuxX86_64Deb,
    LinuxX86_64AppImage,
    WindowsX86_64Msi,
}

pub struct UpdateEntry {
    pub url: url::Url,
    pub sha256: [u8; 32],
    pub signature: ed25519_dalek::Signature,
}

pub trait UpdateClient {
    async fn poll(&self) -> Result<Option<UpdateManifest>, UpdateError>;
    async fn download_and_verify(&self, entry: &UpdateEntry) -> Result<PathBuf, UpdateError>;
    fn current_platform() -> PlatformKey;
}

pub fn apply_update(downloaded: &Path, platform: PlatformKey) -> Result<(), UpdateError>;
```

GitHub Release 의 update.json schema:
```json
{
  "version": "0.1.1",
  "released_at": "2026-05-01T12:00:00Z",
  "notes_url": "https://github.com/GoosLab/moai-studio/releases/tag/v0.1.1",
  "platforms": {
    "macos-universal":      { "url": "...", "sha256": "...", "signature": "..." },
    "linux-x86_64-deb":     { "url": "...", "sha256": "...", "signature": "..." },
    "linux-x86_64-appimage":{ "url": "...", "sha256": "...", "signature": "..." },
    "windows-x86_64-msi":   { "url": "...", "sha256": "...", "signature": "..." }
  }
}
```

후속 SPEC 이 변경할 수 없는 부분: PlatformKey enum variant 이름, UpdateManifest schema 의 top-level 필드. 신규 platform 추가는 가능 (semver minor).

---

## 13. 추적성

### 13.1 CLAUDE.local.md ↔ 본 SPEC

| CLAUDE.local.md 섹션 | 본 SPEC 매핑 |
|---------------------|--------------|
| §1.1 Branch model (release/v{x.y.z}, hotfix/v{x.y.z+1}) | REQ-PK-060 의 tag trigger 정합 |
| §5 Release Drafter | RG-PK-6 전체 |
| §5.3 Version bump labels | REQ-PK-052 |
| §6.2 Release 준비 워크플로 | RG-PK-7 (release.yml) 가 자동화 |
| §6.3 Hotfix 워크플로 | REQ-PK-060 의 tag regex (hotfix tag 도 동일 트리거) |
| §8 v0.1.0 임시 규칙 | 본 SPEC implement 진입 시점 = v0.1.0 분기 직전 |

### 13.2 v3 functional SPEC 무변경 (G12)

SPEC-V3-001 ~ V3-010 의 코드는 본 SPEC 으로 인해 변경되지 않는다. 본 SPEC 은 `.github/workflows/release.yml` + `Cargo.toml` 메타 + 신규 update crate (또는 app crate 의 격리된 디렉터리) 만 변경. CI 검증 (path-filter diff) 으로 보장.

---

## 14. 용어 정의

| 용어 | 정의 |
|------|------|
| universal binary | macOS 의 multi-architecture binary. arm64 + x86_64 양쪽을 single file 에 포함 (Mach-O fat binary). `lipo -create` 로 합성. |
| notarization | Apple 의 공증 절차. 서명된 macOS 앱을 Apple 서버에 submit, 자동 분석 후 ticket 발행. Gatekeeper 의 사전 조건 (2019년 이후). |
| stapling | notarization ticket 을 .app 또는 .dmg 안에 embed. offline 사용자도 ticket 검증 가능. `xcrun stapler staple`. |
| Gatekeeper | macOS 의 앱 실행 보안 메커니즘. 서명 + notarization 미통과 시 "확인되지 않은 개발자" 경고. |
| SmartScreen | Windows 10/11 의 다운로드 평판 검사. EV cert 서명만 즉시 신뢰. |
| EV cert | Extended Validation 코드 서명 인증서. Hardware Security Module (HSM, USB token 또는 cloud) 기반. Standard cert 와 달리 즉시 SmartScreen 신뢰. |
| AppImage | Linux portable 앱 형식. 단일 실행 파일 안에 의존 라이브러리 bundling. 어떤 배포판에서도 실행. |
| .deb | Debian/Ubuntu 의 패키지 형식. `dpkg` / `apt` 가 처리. metadata + assets + control script. |
| .msi | Microsoft Installer. Windows 의 표준 인스톨러 형식. `msiexec` 가 처리. |
| Ed25519 | Edwards-curve digital signature 알고리즘. 32 byte public key + 64 byte signature. update.json 서명에 사용. |
| update manifest | release 시점에 생성되는 JSON 파일. 각 platform binary 의 url + sha256 + signature 를 담음. 앱이 polling. |
| stream-json | Claude CLI / moai-adk Go 의 line-delimited JSON 프로토콜. (본 SPEC 과 직접 관계 없음 — V3-009 carry 참조). |

---

## 15. 변경 이력 정책

본 spec.md 는 추가 revision 누적 시 `## 16. Sprint Contract Revisions` section 을 신설하고 `### 16.1 / 16.2 / ...` 로 누적한다 (SPEC-V3-003 §10.x 패턴 따름). RG-PK-* 의 self-application — 본 SPEC 자신이 본 SPEC 의 release 검증 fixture 가 된다 (v0.1.0 release 자체).

---

작성 종료. 본 spec.md 는 plan.md (구현 milestone × task) + research.md (배경 분석) 와 함께 SPEC-V3-011 implement 진입의 입력이다. implement 는 별도 feature 브랜치 (`feature/SPEC-V3-011-packaging`) 에서 v3 functional SPEC 의 80%+ AC pass 후 + 사용자 v0.1.0-rc1 결정 후 시작한다.
