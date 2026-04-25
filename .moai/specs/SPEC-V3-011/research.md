# SPEC-V3-011 Research — Cross-platform Packaging & Auto-update

작성: MoAI (manager-spec, 2026-04-25)
브랜치: `feature/SPEC-V3-004-render` (현재 브랜치, 본 SPEC 은 문서 산출만)
선행: SPEC-V3-001 ~ V3-010 (모든 v3 implementation SPEC complete 권장 — 본 SPEC 은 packaging/distribution 인프라이므로 코드 안정 후 진입)
범위: macOS / Linux / Windows 3-platform 바이너리 빌드 + 서명/공증 + 배포 채널 + 자동 업데이트 — moai-studio v0.1.0+ 정식 릴리스의 외부 인프라.

---

## 1. 동기 — v0.1.0 release infrastructure

### 1.1 사용자 가치

moai-studio 가 사용자에게 도달하는 마지막 km. 본 SPEC 의 산출은:

- macOS 사용자가 `.dmg` 를 다운받아 drag-to-Applications 만으로 설치한다 (Gatekeeper 통과, Apple notarized).
- Linux 사용자가 `.deb` (Ubuntu/Debian) 또는 `.AppImage` (배포 무관) 중 선호하는 형식을 받는다.
- Windows 사용자가 `.msi` 를 더블클릭만으로 설치한다 (SmartScreen 경고 없이, EV signed).
- 사용자가 신버전을 누락 없이 받는다 — 앱 실행 시 자동으로 update manifest 를 polling, 새 버전 발견 시 사용자 동의로 in-place upgrade.
- 빌드 인프라가 GitHub Actions 만으로 재현 가능 (private CI 의존성 없음).
- Release Drafter (CLAUDE.local.md §5) 의 CHANGELOG 가 release notes 로 자동 흘러간다.

본 SPEC 은 moai-studio v0.1.0 정식 릴리스의 hard prerequisite 이며, 이 SPEC 이 완료되어야 §1.1 의 사용자 가치가 실현된다.

### 1.2 Enhanced GitHub Flow 와의 정합

CLAUDE.local.md §1 의 branch model 은 `release/v{x.y.z}` → `main` + tag 후 release 트리거를 가정한다. §5 의 Release Drafter 는 PR label 기반 CHANGELOG draft 를 누적한다. 본 SPEC 은 그 release tag 발생 시점에 다음을 자동화한다:

- 3-platform 빌드 (macOS arm64+x86_64, Linux x86_64+arm64, Windows x86_64) — cargo cross-compile 또는 GitHub Actions matrix.
- 서명 (macOS Developer ID, Windows EV cert).
- 공증 (macOS notarytool — Apple 서버 round-trip 5~30 분).
- 패키징 (.dmg / .deb / .AppImage / .msi).
- artifact 를 GitHub Releases 로 upload (tag-attached).
- update manifest JSON (`update.json`) 갱신.
- Release Drafter draft publish.

§1 의 hotfix 흐름 (`hotfix/v0.1.1-{slug}` → main + tag) 도 동일한 release workflow 가 트리거되어야 한다 — release vs hotfix 구분 없음, tag 패턴만으로 인식.

### 1.3 v0.1.0 이후 순서 — 본 SPEC 의 진입 시점

CLAUDE.local.md §8 (v0.1.0 release까지 임시 규칙) 와의 정합:

- 현재 (2026-04-25): pre-release v0.0.x. develop 이 사실상 stable.
- v3 SPEC 진행 중: V3-001 (foundation), V3-002 (panes), V3-003 (tabs), V3-004 (render), V3-005 (file explorer), V3-006 (markdown viewer), V3-008 (terminal), V3-009 (SPEC management UI), V3-010 (agent dashboard).
- 본 SPEC (V3-011) 은 v3 functional SPEC 들이 stabilize 된 후 — 즉 develop 이 release/v0.1.0 분기 직전 — 에 implement 진입해야 한다. 코드가 흐르는 동안 packaging 을 검증하면 outdated artifact 가 release 후보가 되는 위험.

implement 진입 조건:
- v3 functional SPEC 의 AC pass count 가 합산 80% 이상.
- develop 이 v0.1.0-rc1 candidate.
- USER-DECISION (§7) 모두 결정.
- 서명 인증서 보유 확인 (P0 차단).

---

## 2. 코드베이스 분석 — 현재 가용한 building block

### 2.1 워크스페이스 구조

`Cargo.toml` workspace members (현재 11+ crate). Binary crate 후보:

- `crates/moai-studio-app/` (또는 동등 entry crate) — 실제 GUI 진입점. 본 SPEC 의 packaging 대상.

빌드 산출:
- macOS: `target/release/moai-studio` 또는 `.app` bundle (Cargo.toml `[package.metadata.bundle]`).
- Linux: `target/release/moai-studio` ELF.
- Windows: `target/release/moai-studio.exe` PE.

### 2.2 GitHub Actions 현황

`.github/workflows/` 의 기존 workflow:
- `ci-rust.yml` (예상) — rustfmt + clippy + test on push/PR.
- `release-drafter.yml` (CLAUDE.local.md §5 참조) — PR label 기반 draft 누적.

본 SPEC 의 신규 workflow (`release.yml`) 는 위 두 개와 별도이며, `tag push` 트리거로만 동작한다.

### 2.3 Cargo metadata / 버전 단일 source

- `Cargo.toml` 워크스페이스 root 의 `[workspace.package] version = "0.1.0"` (또는 각 member 의 `version`).
- 본 SPEC 은 release tag 가 `v{x.y.z}` 일 때 Cargo.toml version 과 일치하는지 verify 한다 (RG-PK-7).

### 2.4 stream-json IPC 와 packaging 의 관계

본 SPEC 은 **코드베이스를 변경하지 않는다** (제약 §1.3 명시). packaging 은 외부 빌드 인프라이며, `crates/moai-stream-json`, `crates/moai-studio-spec`, terminal/panes/tabs core 의 코드는 unchanged. 만약 packaging 과정에서 코드 변경이 필요해지면 (예: feature flag) 별도 SPEC 으로 분리한다.

---

## 3. macOS packaging — Apple ecosystem deep-dive

### 3.1 빌드: Universal Binary (arm64 + x86_64)

Apple Silicon (M1+) 과 Intel Mac 양쪽에 single `.dmg` 를 배포하기 위해 universal binary 를 만든다.

옵션:
- (A) `cargo-bundle` + `lipo` post-process. 간단하지만 cargo-bundle 의 universal 지원이 불안정.
- (B) `cargo build --target aarch64-apple-darwin` + `cargo build --target x86_64-apple-darwin` 별도 빌드 후 `lipo -create -output` 으로 fat binary 합성. **권장** — 검증된 패턴 (Tauri, Bevy 등 모두 이 방식).

단계:
```
cargo build --release --target aarch64-apple-darwin -p moai-studio-app
cargo build --release --target x86_64-apple-darwin -p moai-studio-app
mkdir -p target/universal/release
lipo -create -output target/universal/release/moai-studio \
  target/aarch64-apple-darwin/release/moai-studio \
  target/x86_64-apple-darwin/release/moai-studio
```

### 3.2 .app bundle 구성

`.app` 는 macOS 의 application bundle 디렉터리 구조:

```
moai-studio.app/
└── Contents/
    ├── Info.plist           # bundle metadata
    ├── MacOS/
    │   └── moai-studio      # universal binary
    ├── Resources/
    │   └── icon.icns        # app icon (multi-resolution)
    └── _CodeSignature/      # 서명 산출 (codesign)
```

Info.plist 필수 필드:
- `CFBundleIdentifier`: `kr.ai.mo.moai-studio`
- `CFBundleVersion`, `CFBundleShortVersionString`: Cargo.toml version 동기화
- `NSHighResolutionCapable`: true
- `LSMinimumSystemVersion`: 14.0 (macOS Sonoma)

생성 방법:
- `cargo-bundle` 가 자동 생성 (Cargo.toml `[package.metadata.bundle]` 설정 기반). 검증 필요.
- 또는 GitHub Actions 에서 직접 plist 템플릿 + envsubst.

### 3.3 코드 서명 (Apple Developer ID)

요건:
- Apple Developer Program 등록 ($99/year).
- Developer ID Application 인증서 (Keychain Access 또는 Apple Developer 포털 → Certificates).
- 인증서를 GitHub Actions secret 으로 등록 (`APPLE_CERTIFICATE_P12_BASE64`, `APPLE_CERTIFICATE_PASSWORD`).

서명 명령:
```
codesign --force --deep --sign "Developer ID Application: <Team>" \
  --options runtime \
  --entitlements moai-studio.entitlements \
  moai-studio.app
```

`--options runtime` 은 hardened runtime 활성화 (notarization 필수 조건).

entitlements (필요 시):
- `com.apple.security.cs.allow-jit` — JIT 사용 시 (GPUI 의 metal-rs 가 필요할 수 있음).
- `com.apple.security.cs.disable-library-validation` — 외부 dylib 로드 시.

### 3.4 Notarization (Apple 공증)

서명만으로 Gatekeeper 통과 불가 — Apple 서버 공증 필수 (2019년 이후).

방법: `xcrun notarytool` (Xcode 13+, 2021년 이후 표준).

```
# .dmg 또는 .zip 으로 묶어서 submit
xcrun notarytool submit moai-studio.dmg \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_APP_SPECIFIC_PASSWORD" \
  --wait
```

`--wait` 는 동기 대기 (5~30분, 일반적으로 5~10분). 비동기 완료 후 `xcrun notarytool history` 로 확인 가능.

공증 통과 후 stapling:
```
xcrun stapler staple moai-studio.dmg
```

stapling 은 공증 ticket 을 .dmg 안에 embed 하므로 사용자 머신이 offline 이어도 Gatekeeper 가 검증 가능.

### 3.5 .dmg 생성

옵션:
- (A) `create-dmg` (npm/brew, 검증된 도구) — background image, drag-to-Applications shortcut 등 customization 풍부. **권장**.
- (B) `hdiutil create` (macOS 내장) — 기본만 가능, custom 어려움.

```
create-dmg \
  --volname "moai-studio" \
  --background "assets/dmg-background.png" \
  --window-size 600 400 \
  --icon-size 100 \
  --icon moai-studio.app 150 200 \
  --app-drop-link 450 200 \
  --hdiutil-quiet \
  moai-studio.dmg \
  moai-studio.app/
```

### 3.6 GitHub Actions runner — `macos-14`

`macos-14` runner 가 Apple Silicon (M1) 기반. 빌드 + 서명 + 공증 모두 가능. 주의:
- runner 마다 Xcode 버전 다름 (`xcode-select` 또는 `actions/setup-xcode` 으로 지정).
- private repo 는 macOS runner 분당 비용 10x (현재 GitHub Actions billing 차단 상태 — RG-PK-7 의 외부 차단으로 명시).

---

## 4. Linux packaging — .deb + .AppImage 듀얼 트랙

### 4.1 .deb (Debian/Ubuntu 패키지)

도구: `cargo-deb` (Rust 생태계 표준).

`Cargo.toml` 의 `[package.metadata.deb]` 설정:
```toml
[package.metadata.deb]
maintainer = "MoAI <email@mo.ai.kr>"
copyright = "2026, MoAI"
license-file = ["LICENSE", "0"]
extended-description = "moai-studio is the GUI shell for MoAI-ADK..."
depends = "$auto, libgtk-3-0, libwebkit2gtk-4.0-37"
section = "utility"
priority = "optional"
assets = [
    ["target/release/moai-studio", "usr/bin/", "755"],
    ["assets/moai-studio.desktop", "usr/share/applications/", "644"],
    ["assets/icons/256x256/moai-studio.png", "usr/share/icons/hicolor/256x256/apps/", "644"],
]
```

빌드:
```
cargo deb -p moai-studio-app --target x86_64-unknown-linux-gnu
```

산출: `target/x86_64-unknown-linux-gnu/debian/moai-studio_0.1.0_amd64.deb`

### 4.2 .AppImage (배포 무관, portable)

도구: `appimagetool` (linuxdeploy 권장).

장점: 단일 파일로 어떤 Linux 배포판에서도 실행 (Ubuntu / Fedora / Arch 등). 사용자가 `chmod +x` 후 더블클릭.

빌드 단계:
1. `linuxdeploy --appdir AppDir --executable target/release/moai-studio --desktop-file moai-studio.desktop --icon-file icon.png`
2. `appimagetool AppDir moai-studio-x86_64.AppImage`

dependency bundling: linuxdeploy 가 자동으로 ELF 의 dynamic library 를 AppDir 에 복사 (libfontconfig, libgcc 등). GPUI 는 OpenGL/Vulkan 의존이 있을 수 있음 — 검증 필요.

### 4.3 GPG 서명 (Linux 표준)

apt repository 에 .deb 를 publish 하려면 GPG signed 필수. 다만 GitHub Releases 에 직접 attach 만 한다면 optional (사용자가 `dpkg -i` 로 직접 설치).

```
gpg --armor --detach-sign moai-studio_0.1.0_amd64.deb
# 산출: moai-studio_0.1.0_amd64.deb.asc
```

GPG key 는 GitHub Actions secret 으로 등록 (`GPG_PRIVATE_KEY`, `GPG_PASSPHRASE`).

### 4.4 GitHub Actions runner — `ubuntu-22.04`

표준 runner. Linux 패키징 비용 1x (가장 저렴). 빌드 시간 5~15 분.

---

## 5. Windows packaging — .msi + WinUI manifest

### 5.1 .msi 생성

도구: `cargo-wix` (Wix Toolset 래퍼). 산출은 Microsoft Installer 표준 .msi.

`Cargo.toml` 의 `[package.metadata.wix]`:
```toml
[package.metadata.wix]
upgrade-guid = "<GUID>"
path-guid = "<GUID>"
license = false
eula = false
```

`wix/main.wxs` (자동 생성 후 customize):
- `<Product>` element 의 Name, Version, Manufacturer.
- Start Menu shortcut.
- File association (옵션, .pen 등).

빌드:
```
cargo wix --no-build --output target/wix/moai-studio-0.1.0-x86_64.msi
```

### 5.2 코드 서명 (Authenticode)

Windows 가 SmartScreen 경고를 표시하지 않으려면 EV (Extended Validation) 코드 서명 인증서 필수.

요건:
- EV 인증서 ($300~$500/year, DigiCert/Sectigo/Comodo).
- USB hardware token (HSM) — EV 인증서는 software 키 export 불가.
- HSM 기반이라 GitHub Actions hosted runner 에서 직접 서명 불가 → **DigiCert KeyLocker** 같은 cloud HSM 서비스 사용 또는 self-hosted runner.

서명 명령 (`signtool.exe` — Windows SDK 포함):
```
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 \
  /a moai-studio-0.1.0-x86_64.msi
```

### 5.3 WinUI 3 manifest (선택)

moai-studio 가 GPUI 위에서 구동되므로 native WinUI 사용은 불필요. 다만 `.msi` 안의 application manifest 에:
- `requestedExecutionLevel`: `asInvoker` (관리자 권한 불필요).
- `dpiAware`: `true/PM` (high-DPI display 지원).
- `supportedOS`: Windows 10/11 GUID.

### 5.4 GitHub Actions runner — `windows-2022`

표준 runner. Windows SDK 와 Wix Toolset 사전 설치됨 (또는 `microsoft/setup-msbuild` action). 비용 2x (Linux 대비).

---

## 6. Auto-update — 자체 vs Sparkle/Squirrel

### 6.1 옵션 비교

| 항목 | (A) 자체 (GitHub Releases JSON) | (B) Sparkle (macOS) + WinSparkle (Win) + 자체 (Linux) |
|------|---------------------------------|------------------------------------------------------|
| 외부 의존성 | 0 (HTTP + JSON 만) | Sparkle (Objective-C), WinSparkle (C++) |
| 구현 복잡도 | Rust 100~200 LOC | macOS 측 Objective-C bridge 필요 |
| 차등 update (delta) | 미지원 (full download) | Sparkle 지원 |
| 자동 silent update | 가능 (사용자 동의 후) | 가능 |
| 서명 검증 | 자체 구현 필요 (Ed25519 권장) | Sparkle 가 EdDSA 자동 검증 |
| 플랫폼 일관성 | 단일 코드베이스 | 플랫폼별 분기 (Sparkle / WinSparkle / 자체) |
| 인프라 비용 | GitHub Releases (무료) | 동일 |
| 권장 | **default ✓** | 외부 의존 최소화 원칙과 충돌 |

### 6.2 자체 update 메커니즘 (default)

설계:

```
1. 앱 시작 시 (또는 24h 주기) update.json 을 polling
   - URL: https://github.com/GoosLab/moai-studio/releases/latest/download/update.json
2. update.json schema:
   {
     "version": "0.1.1",
     "released_at": "2026-05-01T12:00:00Z",
     "notes_url": "https://github.com/.../releases/tag/v0.1.1",
     "platforms": {
       "macos-universal": {
         "url": "https://.../moai-studio-0.1.1.dmg",
         "sha256": "...",
         "signature": "<Ed25519 signature of sha256>"
       },
       "linux-x86_64-deb": { ... },
       "linux-x86_64-appimage": { ... },
       "windows-x86_64-msi": { ... }
     }
   }
3. 현재 버전 vs update.json.version 비교 (semver)
4. 새 버전 발견 시 사용자에게 in-app notification:
   "v0.1.1 이 출시되었습니다. 지금 업데이트하시겠습니까? (변경사항)"
5. 사용자 동의 시:
   - 플랫폼 맞는 url 다운로드 → /tmp 또는 %TEMP%
   - sha256 검증 + Ed25519 signature 검증
   - macOS: 다운로드된 .dmg 마운트 후 .app 교체 (앱 재시작 필요)
   - Linux: .AppImage 의 경우 self-replace, .deb 는 사용자에게 수동 설치 안내
   - Windows: .msi 를 quiet mode 로 실행 (`msiexec /i ... /qb`)
6. 다음 실행 시 새 버전.
```

서명 검증용 public key 는 앱 바이너리에 embed (build time). Rust crate: `ed25519-dalek` v2.

### 6.3 Linux self-replace 의 함정

- `.deb` 사용자 — sudo 권한 없는 in-app update 불가능. 수동 설치 안내만.
- `.AppImage` 사용자 — 단일 파일이므로 self-replace 가능 (`std::fs::rename` 후 재실행).

→ Linux 는 deb vs AppImage 에 따라 update path 분기. update.json 에 양쪽 url 모두 포함.

### 6.4 macOS / Windows admin 권한

- macOS: `.app` 이 `/Applications/` 또는 `~/Applications/` 에 설치되어 있는지에 따라 admin prompt 발생. user-installed 면 무권한 update 가능.
- Windows: `.msi` 설치 시 보통 `Program Files` (admin 필요). user install (`%LOCALAPPDATA%`) 옵션 추가 검토.

### 6.5 결정 권장: (A) 자체

근거:
- moai-studio 의 외부 의존성 최소화 원칙 (현재 코드베이스도 third-party crate 신중).
- 서명 검증은 Ed25519 + ed25519-dalek 으로 충분.
- delta update 는 v0.1.0 단계에서 불필요 (full download 가 5~20MB).
- 플랫폼 일관성 (단일 Rust 코드).

USER-DECISION-PK-A 게이트로 최종 결정.

---

## 7. USER-DECISION 게이트 — implement 진입 시 결정

### 7.1 USER-DECISION-PK-A — Auto-update 메커니즘 (MS-3 진입)

질문: "자동 업데이트 메커니즘은?"

옵션:
- (a) **권장: 자체 (GitHub Releases JSON manifest)** — 외부 의존 0, Rust 100~200 LOC, Ed25519 서명 검증. plat 일관성.
- (b) Sparkle (macOS) + WinSparkle (Win) + 자체 (Linux) — 플랫폼별 검증된 라이브러리. 하지만 Objective-C bridge + 분기 로직.
- (c) skipping auto-update for v0.1.0 — 사용자가 수동으로 신버전 다운로드. v0.2.0 에서 추가.

영향 범위: RG-PK-5 전체, MS-3 산출.

### 7.2 USER-DECISION-PK-B — 서명 인증서 보유 (P0 차단)

질문: "macOS Developer ID + Windows EV cert 를 보유했는가?"

옵션:
- (a) **권장: 보유함 (Apple Developer Program $99 + DigiCert EV $300)** — MS-2 진입 가능.
- (b) 보유 안함 — MS-2 차단. MS-1 (unsigned 빌드) 까지만 진행. 사용자 경고: macOS Gatekeeper 우회 (우클릭 열기) 필요, Windows SmartScreen 경고.
- (c) self-signed (개발 단계만) — production 부적격. internal testing 한정.

영향 범위: RG-PK-2 (macOS sign+notarize), RG-PK-4 (Windows sign), MS-2 진입.

[HARD] 본 게이트는 P0 — 차단되면 MS-2 진입 불가. release-ready 산출 무효.

### 7.3 USER-DECISION-PK-C — Release tag naming (Enhanced GitHub Flow 정합)

질문: "릴리스 tag 명명 컨벤션은?"

옵션:
- (a) **권장: `v{x.y.z}` (CLAUDE.local.md §1.2 준수)** — `v0.1.0`, `v0.1.1`. 현재 hotfix 명명 (`hotfix/v0.1.1-{slug}`) 와 직접 매칭.
- (b) `v{x.y.z}-rc{n}` 추가 — `v0.1.0-rc1` → `v0.1.0`. release candidate 단계 분리. release.yml 가 `-rc*` 는 pre-release 로 mark.
- (c) calendar versioning (`2026.04.25`) — semver 포기. 본 프로젝트와 부적합.

영향 범위: RG-PK-7 (tag trigger regex), Release Drafter (CLAUDE.local.md §5.3 의 release/major|minor|patch 라벨과 정합).

### 7.4 USER-DECISION-PK-D — Crash reporting (선택, 권장 default opt-out)

질문: "크래시 리포팅을 도입할 것인가?"

옵션:
- (a) **권장: opt-out (도입 안함)** — 사용자 프라이버시 우선. v0.1.0 단계 적합. 이슈 리포트는 GitHub Issues 로.
- (b) Sentry SaaS opt-in — 사용자가 명시 동의 시만 전송. $26/month base.
- (c) 자체 crash reporter — `panic_hook` + 로컬 파일 + 사용자 명시 업로드. v0.2.0+ 후보.

영향 범위: 선택 기능. 본 SPEC 의 RG 그룹에는 포함 안함 (별 SPEC).

---

## 8. CI / Release workflow 설계

### 8.1 Trigger

```yaml
on:
  push:
    tags:
      - 'v*.*.*'              # v0.1.0, v0.1.1, ...
      - 'v*.*.*-rc*'          # USER-DECISION-PK-C (b) 시
```

### 8.2 Job matrix

```yaml
jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-14
            target: aarch64-apple-darwin
            kind: macos-arm64
          - os: macos-14
            target: x86_64-apple-darwin
            kind: macos-x86_64
          - os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            kind: linux-x86_64
          - os: windows-2022
            target: x86_64-pc-windows-msvc
            kind: windows-x86_64
```

각 job 은:
1. Rust toolchain setup (workspace `rust-toolchain` 따름).
2. cargo build --release --target.
3. 플랫폼별 packaging (cargo-bundle / cargo-deb / cargo-wix / linuxdeploy).
4. 서명 (macOS codesign, Windows signtool).
5. artifact upload (`actions/upload-artifact`).

### 8.3 Aggregation job

`release` job 이 모든 build job 의 artifact 를 download:

1. macOS arm64 + x86_64 → `lipo` universal binary → `.dmg` → notarize → staple.
2. Linux .deb + .AppImage → optional GPG sign.
3. Windows .msi → already signed in build job.
4. update.json 생성 (sha256 + Ed25519 sign).
5. `softprops/action-gh-release` 로 GitHub Release 에 모두 attach.
6. Release Drafter draft publish 트리거 (workflow_dispatch).

### 8.4 외부 차단 — CI billing

CLAUDE.local.md §8 / 사용자 프롬프트의 "CI billing 해소 의존성" 명시. 현재 GitHub Actions private repo macOS runner 가 분당 $0.08, 빌드+공증에 평균 30분 → release 1회당 $7.2~$15 (macOS 만).

본 SPEC 의 RG-PK-7 는 workflow 정의 + 검증 까지가 범위이며, **실 release 트리거는 billing 해소 후**. 즉 implement complete 시점에 dry-run (workflow_dispatch + skip-deploy flag) 까지만 검증. tag push 실 트리거는 v0.1.0 release 시점 (사용자 결정).

---

## 9. 위험 (Risk Register — pre-implement)

| ID | 위험 | 영향 | 완화 시점 |
|----|------|------|----------|
| R-PK-1 | Apple Notarization 정책 변경 (entitlements 추가 요구) | macOS 빌드 실패 | MS-2 entry 시 notarize log 점검 |
| R-PK-2 | Windows EV cert 미보유 → SmartScreen 경고 | 사용자 설치 진입장벽 | USER-DECISION-PK-B (a/b) |
| R-PK-3 | GPUI Metal 의존성 → entitlements 추가 (allow-jit) | 첫 빌드 실패 | MS-1 spike |
| R-PK-4 | Linux distro fragmentation (Ubuntu LTS 22 vs 24, libwebkit 버전 차) | AppImage 의존 누락 | MS-1 ubuntu-22.04 + ubuntu-24.04 dual matrix 검증 |
| R-PK-5 | universal binary lipo 실패 (architecture flag 불일치) | macOS 빌드 fail | MS-1 spike |
| R-PK-6 | update.json hosting 단점 (GitHub Releases throttle) | update polling 빈도 제한 | etag/cache-control 활용 + 24h poll 주기 |
| R-PK-7 | self-update Windows admin 권한 부재 | silent update 실패 | user-install (`%LOCALAPPDATA%`) 옵션 + 사용자 안내 |
| R-PK-8 | Ed25519 private key leak | malicious update 가능 | Hardware token (YubiKey) + GitHub Actions secret env-only |
| R-PK-9 | Cargo.toml version vs git tag 불일치 | release artifact 의 버전 메타 오류 | RG-PK-7 의 verify step (tag regex match Cargo.toml) |
| R-PK-10 | macos-14 runner Xcode 버전 변경 | 빌드 도구 break | `actions/setup-xcode@v1` pin 명시 |
| R-PK-11 | .deb dependency 누락 (libgtk 등 GPUI 의존) | 설치 후 실행 실패 | MS-1 cargo-deb depends 명시 + e2e install test |
| R-PK-12 | CI billing 해소 지연 | release workflow 검증 불가 | dry-run + workflow_dispatch 로 무release 검증 가능 |

---

## 10. SPEC 의존성 그래프

```
SPEC-V3-001 (foundation) — 선행
SPEC-V3-002 (panes)      — 선행
SPEC-V3-003 (tabs)       — 선행
SPEC-V3-004 (render)     — 선행
SPEC-V3-005 (file expl)  — 선행
SPEC-V3-006 (markdown)   — 선행
SPEC-V3-008 (terminal)   — 선행
SPEC-V3-009 (SPEC mgmt)  — 선행
SPEC-V3-010 (agent dash) — 선행
                              ↓
                    SPEC-V3-011 (본 SPEC)
                              ↓
                       v0.1.0 release
```

본 SPEC 의 implement 진입 조건 (재확인): v3 functional SPEC 의 AC pass count 합산 80%+ AND 사용자가 v0.1.0-rc1 결정.

---

## 11. AC 후보 (spec.md 로 확정 예정)

총 12 개 AC 후보 (사용자 프롬프트 "10-14" 범위 중간):

1. AC-PK-1: cargo-bundle (또는 lipo) 가 macOS universal binary 를 생성. file 명령으로 architecture 검증.
2. AC-PK-2: cargo-deb 가 .deb 패키지 생성. lintian (옵션) 통과.
3. AC-PK-3: linuxdeploy 가 .AppImage 생성. dynamic library bundling 완료 (ldd 검증).
4. AC-PK-4: cargo-wix 가 .msi 생성. Windows 10/11 에서 dry install 성공.
5. AC-PK-5: macOS .app 가 Developer ID 로 서명되고 notarize 통과 (xcrun notarytool history).
6. AC-PK-6: macOS .dmg 가 stapled 되어 offline 머신에서 Gatekeeper 통과.
7. AC-PK-7: Windows .msi 가 EV cert 로 서명되어 SmartScreen 경고 없음.
8. AC-PK-8: update.json 이 release tag 트리거 시 자동 생성 + sha256 + Ed25519 서명 포함.
9. AC-PK-9: 앱이 update.json 을 polling, 새 버전 발견 시 사용자 동의 UI 표시.
10. AC-PK-10: 사용자 동의 후 in-place update — macOS .app 교체, Windows msiexec quiet, Linux self-replace (AppImage) 또는 안내 (deb).
11. AC-PK-11: release.yml workflow 가 tag push 트리거로 모든 platform artifact 를 GitHub Release 에 attach.
12. AC-PK-12: Release Drafter draft 가 release tag 시점에 자동 publish (CHANGELOG → release notes).

---

## 12. 외부 차단 명시

본 SPEC 의 implement 진입은 다음 외부 차단의 해소가 전제다:

1. **CI billing 해소** — GitHub Actions private repo macOS runner 비용. 사용자가 billing 활성 결정 시까지 release.yml 의 실 트리거는 보류.
2. **서명 인증서** — USER-DECISION-PK-B 의 (a) 결정 시까지 MS-2 진입 불가.
3. **Apple Developer Program 등록** — 단일 Team ID, Apple ID, app-specific password 확보. GitHub Actions secret 입력 가능해야 함.
4. **DigiCert EV cert 또는 KeyLocker 계약** — Windows MS-2 진입 prerequisite.
5. **GPG key (Linux 옵션)** — 선택, .deb apt repo 운영 시.

본 SPEC 의 RG / AC 는 위 차단이 해소된 후의 상태를 정의하되, **MS-1 (서명 없는 빌드)** 까지는 위 차단 없이 진입 가능하도록 단계 분리한다.

---

## 13. 결론

본 SPEC 은 moai-studio v0.1.0 정식 릴리스의 외부 인프라 정의다. 코드베이스는 변경하지 않으며, `.github/workflows/release.yml` + `Cargo.toml` 메타데이터 + 자체 update 모듈 (별 crate 또는 app crate 내 `update/` 모듈) 만 추가/변경한다.

3 milestone 으로 분할:
- MS-1: 3-platform unsigned 빌드 + 기본 packaging (.app/.deb/.AppImage/.msi). 서명 없음.
- MS-2: 서명 + notarization (서명 인증서 보유 시).
- MS-3: Auto-update + release workflow + 자동화.

USER-DECISION 4 게이트, AC 12 개, 외부 차단 5 항목 명시.

본 research.md 는 spec.md / plan.md 의 입력이다.
