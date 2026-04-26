# SPEC-V3-011 Implementation Plan — Cross-platform Packaging & Auto-update

작성: MoAI (manager-spec, 2026-04-25)
브랜치 (현행 SPEC 작성): `feature/SPEC-V3-004-render`
브랜치 (implement 진입 시): `feature/SPEC-V3-011-packaging` (v3 functional SPEC 80%+ AC pass 후 develop 에서 분기 — CLAUDE.local.md §1.3 명명 규칙 준수)
범위: SPEC-V3-011 spec.md 의 RG-PK-1 ~ RG-PK-7, AC-PK-1 ~ AC-PK-12 를 MS-1 / MS-2 / MS-3 으로 분할 구현.
선행: SPEC-V3-001 ~ V3-010 (functional SPEC complete 권장), CI billing 해소, USER-DECISION-PK-B RESOLVED (b) — MS-2 BLOCKED, MS-1 만 v0.1.0 진행.

---

## 1. Milestone × Task 표

| Task | Milestone | 책임 영역 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|----------|----------------------|-----|----|
| **T0** | MS-1 | USER-DECISION-PK-C | (게이트, tag naming) | — | (게이트) |
| **T1** | MS-1 | release.yml 신설 (build matrix) | `.github/workflows/release.yml` | T0 | (구조) |
| **T2** | MS-1 | Cargo.toml metadata (bundle/deb/wix) | `Cargo.toml` (workspace + app crate) | T1 | AC-PK-1 ~ AC-PK-4 (메타 부분) |
| **T3** | MS-1 | macOS unsigned `.app` + lipo universal | release.yml `build-macos` job 단계 | T2 | AC-PK-1 |
| **T4** | MS-1 | Linux unsigned `.deb` (cargo-deb) | release.yml `build-linux` job + `assets/moai-studio.desktop`, `assets/icons/256x256/moai-studio.png` | T2 | AC-PK-2 |
| **T5** | MS-1 | Linux unsigned `.AppImage` (linuxdeploy) | release.yml `build-linux` job 추가 step + `scripts/build-appimage.sh` | T4 | AC-PK-3 |
| **T6** | MS-1 | Windows unsigned `.msi` (cargo-wix) | release.yml `build-windows` job + `wix/main.wxs` | T2 | AC-PK-4 |
| **T7** | MS-1 | dry-run trigger (workflow_dispatch) + path-filter diff check | release.yml `dispatch` 입력, CI assertion (코드베이스 무변경) | T3 ~ T6 | (gate) |
| **T8** | MS-2 | USER-DECISION-PK-B (P0 차단) | (게이트, 서명 인증서 보유) | T7 | (게이트) |
| **T9** | MS-2 | macOS codesign + entitlements | `assets/moai-studio.entitlements`, release.yml step | T8 | AC-PK-5 |
| **T10** | MS-2 | macOS notarytool + stapler | release.yml step | T9 | AC-PK-5, AC-PK-6 |
| **T11** | MS-2 | macOS create-dmg + background image | `scripts/create-dmg.sh`, `assets/dmg-background.png` | T10 | AC-PK-6 |
| **T12** | MS-2 | Windows signtool + DigiCert KeyLocker | release.yml step | T8 | AC-PK-7 |
| **T13** | MS-2 | (옵션) Linux GPG sign | release.yml step + GPG_PRIVATE_KEY secret | T8 | (옵션 검증) |
| **T14** | MS-3 | USER-DECISION-PK-A (auto-update 메커니즘) | (게이트) | T11, T12 | (게이트) |
| **T15** | MS-3 | Ed25519 keypair 생성 + secret/public key 등록 | `assets/update-pubkey.bin` (앱 embed), GitHub secret `ED25519_PRIVATE_KEY` | T14 | AC-PK-8 |
| **T16** | MS-3 | 신규 crate `moai-studio-updater` 또는 app crate `update/` 모듈 | `crates/moai-studio-updater/Cargo.toml`, `src/{lib,manifest,client,verify,apply}.rs` | T15 | AC-PK-9, AC-PK-10 |
| **T17** | MS-3 | update polling (24h) + 시작 시 1회 + tokio task | `crates/moai-studio-updater/src/client.rs` | T16 | AC-PK-9 |
| **T18** | MS-3 | UI notification (in-app prompt) | app crate UI 레이어 추가 (격리된 디렉터리) | T17 | AC-PK-9 |
| **T19** | MS-3 | platform-specific apply_update | `crates/moai-studio-updater/src/apply/{macos,windows,linux}.rs` | T16 | AC-PK-10 |
| **T20** | MS-3 | release.yml `release` aggregation job (artifact download + update.json 생성) | release.yml step + `scripts/generate-update-json.sh` | T15, T19 | AC-PK-8, AC-PK-11 |
| **T21** | MS-3 | Release Drafter publish trigger | release.yml step (workflow_dispatch 호출 또는 GitHub API) | T20 | AC-PK-12 |
| **T22** | 전체 | 코드베이스 무변경 regression check | CI assertion (path-filter: crates/moai-studio-{terminal,workspace,ui-shell}/, panes/, tabs/) | T16 ~ T21 | RG-PK-7.6 |
| **T23** | 전체 | progress.md 갱신 + commit | (git 작업, sprint contract revision §16.1 추가) | T1 ~ T22 | (회수) |

---

## 2. T0 — USER-DECISION-PK-C (release tag naming)

**결정 (2026-04-27): RESOLVED → 옵션 (a) `v{x.y.z}` 단일. CLAUDE.local.md §1.2 준수.**
release.yml `on.push.tags` regex 는 `v[0-9]+.[0-9]+.[0-9]+` 단일 패턴만 사용. -rc 패턴 / prerelease 분기 (REQ-PK-063) 는 본 sprint 제외.

### 2.1 호출

[USER-DECISION-REQUIRED: tag-naming-v3-011-ms1]

질문 (AskUserQuestion):
- "릴리스 tag 명명 컨벤션은?"
- (a) **권장: `v{x.y.z}` (CLAUDE.local.md §1.2 준수)** — `v0.1.0`, `v0.1.1`. release.yml trigger regex 단순. hotfix tag (`v0.1.1`) 와 직접 매칭.
- (b) `v{x.y.z}-rc{n}` 추가 — `v0.1.0-rc1` → `v0.1.0`. release candidate 단계 분리. release.yml 가 `-rc*` 를 prerelease=true 로 mark.
- (c) calendar versioning (`2026.04.25`) — semver 포기. 본 프로젝트와 부적합.

### 2.2 결정 기록

option 결정 시 progress.md MS-1 entry 에 기록. (b) 선택 시 release.yml `on.push.tags` 에 두 패턴 모두 포함:
```yaml
on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'
      - 'v[0-9]+.[0-9]+.[0-9]+-rc[0-9]+'
```

---

## 3. T1 — release.yml 신설 (build matrix)

### 3.1 변경 대상

`.github/workflows/release.yml` 신규:

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'
  workflow_dispatch:
    inputs:
      dry_run:
        description: 'Dry-run (skip publish)'
        type: boolean
        default: true

env:
  CARGO_TERM_COLOR: always

jobs:
  verify-version:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - name: Verify Cargo.toml version matches tag
        run: |
          TAG="${GITHUB_REF#refs/tags/v}"
          CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
          [ "$TAG" = "$CARGO_VERSION" ] || exit 1

  build:
    needs: verify-version
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
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build
        run: cargo build --release --target ${{ matrix.target }} -p moai-studio-app
      - uses: actions/upload-artifact@v4
        with:
          name: build-${{ matrix.kind }}
          path: target/${{ matrix.target }}/release/moai-studio*

  release:
    needs: build
    runs-on: ubuntu-22.04
    if: ${{ !github.event.inputs.dry_run }}
    steps:
      - uses: actions/download-artifact@v4
      # ... aggregate, sign, package, attach
```

### 3.2 빌드 검증

`gh workflow run release.yml -f dry_run=true` 로 dry-run 시 모든 build job 이 success, release job 이 skip 되어야 한다.

---

## 4. T2 — Cargo.toml metadata

### 4.1 변경 대상

루트 `Cargo.toml` 또는 app crate `Cargo.toml` 에 추가:

```toml
[package.metadata.bundle]
name = "moai-studio"
identifier = "kr.ai.mo.moai-studio"
version = "0.1.0"
icon = ["assets/icons/icon.icns"]
copyright = "Copyright © 2026 MoAI"
short_description = "GUI shell for MoAI-ADK"
osx_minimum_system_version = "14.0"

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

[package.metadata.wix]
upgrade-guid = "<생성된 GUID>"
path-guid = "<생성된 GUID>"
license = false
eula = false
```

### 4.2 GUID 생성

`uuidgen` 또는 PowerShell `[Guid]::NewGuid()` 으로 생성. upgrade-guid 는 한 번 생성 후 fix (semver upgrade 시 재사용).

---

## 5. T3 — macOS unsigned `.app` + lipo universal

### 5.1 release.yml `build-macos-finalize` step

```yaml
- name: Lipo universal binary
  if: matrix.os == 'macos-14'
  run: |
    mkdir -p target/universal/release
    lipo -create -output target/universal/release/moai-studio \
      target/aarch64-apple-darwin/release/moai-studio \
      target/x86_64-apple-darwin/release/moai-studio
    lipo -info target/universal/release/moai-studio
- name: Build .app bundle
  run: |
    cargo install cargo-bundle --version 0.6
    cargo bundle --release --target universal --format osx
```

### 5.2 검증

- `lipo -info` 출력에 `arm64 x86_64` 포함 (AC-PK-1).
- `.app/Contents/MacOS/moai-studio` 존재 + universal binary.
- `.app/Contents/Info.plist` 의 CFBundleVersion 이 Cargo.toml version 과 일치.

---

## 6. T4 — Linux unsigned `.deb`

### 6.1 release.yml step

```yaml
- name: Install cargo-deb
  if: matrix.kind == 'linux-x86_64'
  run: cargo install cargo-deb --version 2
- name: Build .deb
  run: cargo deb -p moai-studio-app --target x86_64-unknown-linux-gnu --no-build
```

### 6.2 신규 assets

- `assets/moai-studio.desktop`:
  ```
  [Desktop Entry]
  Type=Application
  Name=moai-studio
  Comment=GUI shell for MoAI-ADK
  Exec=/usr/bin/moai-studio
  Icon=moai-studio
  Categories=Development;Utility;
  Terminal=false
  ```
- `assets/icons/256x256/moai-studio.png` (256x256 PNG icon).

### 6.3 검증

- `dpkg-deb --info target/x86_64-unknown-linux-gnu/debian/moai-studio_0.1.0_amd64.deb` 가 maintainer / Depends / Version 메타 정확 보고.
- (옵션) `lintian moai-studio_0.1.0_amd64.deb` warning 0 (AC-PK-2).

---

## 7. T5 — Linux unsigned `.AppImage`

### 7.1 신규 스크립트 `scripts/build-appimage.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# Download linuxdeploy
LINUXDEPLOY=https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
curl -fsSL "$LINUXDEPLOY" -o linuxdeploy
chmod +x linuxdeploy

# Build AppDir
mkdir -p AppDir
./linuxdeploy --appdir=AppDir \
  --executable=target/x86_64-unknown-linux-gnu/release/moai-studio \
  --desktop-file=assets/moai-studio.desktop \
  --icon-file=assets/icons/256x256/moai-studio.png \
  --output=appimage

ls -la moai-studio-*.AppImage
```

### 7.2 release.yml step

```yaml
- name: Build .AppImage
  if: matrix.kind == 'linux-x86_64'
  run: bash scripts/build-appimage.sh
```

### 7.3 검증

- `./moai-studio-x86_64.AppImage --appimage-extract-and-run --version` 또는 smoke 실행.
- ldd 검증: AppDir/usr/lib 에 libfontconfig.so 등 bundled 확인.
- ubuntu-22.04 + ubuntu-24.04 dual matrix (R-PK-4 완화) 옵션.

---

## 8. T6 — Windows unsigned `.msi`

### 8.1 신규 파일 `wix/main.wxs`

`cargo wix init` 으로 자동 생성 후 customize:

```xml
<?xml version="1.0" encoding="windows-1252"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="moai-studio" Language="1033" Version="0.1.0"
             Manufacturer="MoAI" UpgradeCode="<UPGRADE_GUID>">
        <Package InstallerVersion="500" Compressed="yes" />
        <Media Id="1" Cabinet="cab1.cab" EmbedCab="yes" />
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFiles64Folder">
                <Directory Id="INSTALLFOLDER" Name="moai-studio">
                    <Component Id="MainExe" Guid="<PATH_GUID>">
                        <File Id="moai_studio_exe" Source="target\release\moai-studio.exe" KeyPath="yes" />
                    </Component>
                </Directory>
            </Directory>
        </Directory>
        <Feature Id="Main" Level="1">
            <ComponentRef Id="MainExe" />
        </Feature>
    </Product>
</Wix>
```

application manifest 추가 (REQ-PK-031): `requestedExecutionLevel=asInvoker`, `dpiAware=true/PM`, `supportedOS Windows 10/11 GUID`.

### 8.2 release.yml step

```yaml
- name: Install cargo-wix
  if: matrix.kind == 'windows-x86_64'
  run: cargo install cargo-wix --version 0.3
- name: Build .msi
  run: cargo wix --no-build --output target\\wix\\moai-studio-0.1.0-x86_64.msi
```

### 8.3 검증

- Windows 10/11 VM (또는 windows-2022 runner) 에서 `msiexec /i ... /qn` silent install + Program Files 경로 확인 (AC-PK-4).

---

## 9. T7 — dry-run trigger + path-filter diff check

### 9.1 release.yml `dry-run` 입력

```yaml
on:
  workflow_dispatch:
    inputs:
      dry_run:
        type: boolean
        default: true
```

`dry_run: true` 시 release job 의 `if: ${{ !github.event.inputs.dry_run }}` 로 publish skip.

### 9.2 path-filter diff check (코드베이스 무변경)

별도 step:

```yaml
- name: Verify codebase unchanged
  run: |
    set -euo pipefail
    BASE=$(git merge-base HEAD origin/develop)
    git diff --name-only "$BASE" HEAD | tee /tmp/diff-files
    # 코드베이스 path 가 변경되면 fail
    if grep -E '^(crates/moai-studio-(terminal|workspace|panes|tabs|ui|stream-json)/|crates/moai-studio-spec/)' /tmp/diff-files; then
      echo "ERROR: Codebase changed. SPEC-V3-011 must not modify functional code."
      exit 1
    fi
```

본 step 은 RG-PK-7.6 (REQ-PK-065) 의 자동 검증.

---

## 10. T8 — USER-DECISION-PK-B (P0 차단, MS-2 진입)

**결정 (2026-04-27): RESOLVED → 옵션 (b) 현재 미보유. MS-2 BLOCKED.**
v0.1.0 스코프는 MS-1 unsigned builds 까지로 한정. T9 ~ T13 은 인증서 보유 후 별 sprint 진입. 사용자 안내 필수: macOS Gatekeeper 우클릭 우회 / Windows SmartScreen 'More info → Run anyway'.

### 10.1 호출

[USER-DECISION-REQUIRED: signing-cert-v3-011-ms2]

질문 (AskUserQuestion):
- "macOS Developer ID + Windows EV cert 를 보유했는가?"
- (a) **권장: 보유함 (Apple Developer Program $99 + DigiCert EV $300+)** — MS-2 진입 가능. release-ready 산출.
- (b) 보유 안함 — MS-2 차단. MS-1 unsigned 에서 정지. 사용자 경고 안내.
- (c) self-signed (개발 단계만) — production 부적격.

### 10.2 결정 기록

(b) 또는 (c) 결정 시 본 SPEC 의 implement 는 MS-1 까지만 진행. 사용자에게 다음 안내:
- "현재 빌드는 unsigned. macOS 사용자는 우클릭 → 열기 로 Gatekeeper 우회 필요."
- "Windows 사용자는 SmartScreen 경고 표시. 'More info → Run anyway' 로 진행."

---

## 11. T9 ~ T11 — macOS sign + notarize + dmg

### 11.1 T9: codesign + entitlements

신규 `assets/moai-studio.entitlements`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
</dict>
</plist>
```

(GPUI Metal 의 JIT 의존성 — R-PK-3 완화)

release.yml step:

```yaml
- name: Import signing certificate
  if: matrix.os == 'macos-14'
  env:
    APPLE_CERTIFICATE_P12_BASE64: ${{ secrets.APPLE_CERTIFICATE_P12_BASE64 }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  run: |
    echo "$APPLE_CERTIFICATE_P12_BASE64" | base64 -d > cert.p12
    security create-keychain -p "" build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "" build.keychain
    security import cert.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
- name: Codesign .app
  run: |
    codesign --force --deep --sign "Developer ID Application: ${{ secrets.APPLE_TEAM_NAME }}" \
      --options runtime \
      --entitlements assets/moai-studio.entitlements \
      target/universal/release/bundle/osx/moai-studio.app
```

### 11.2 T10: notarize + staple

```yaml
- name: Notarize
  env:
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
    APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }}
  run: |
    # zip the .app for submission
    ditto -c -k --keepParent target/universal/release/bundle/osx/moai-studio.app moai-studio.zip
    xcrun notarytool submit moai-studio.zip \
      --apple-id "$APPLE_ID" \
      --team-id "$APPLE_TEAM_ID" \
      --password "$APPLE_APP_SPECIFIC_PASSWORD" \
      --wait
- name: Staple
  run: xcrun stapler staple target/universal/release/bundle/osx/moai-studio.app
```

### 11.3 T11: create-dmg

신규 `scripts/create-dmg.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

create-dmg \
  --volname "moai-studio" \
  --background "assets/dmg-background.png" \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "moai-studio.app" 150 200 \
  --app-drop-link 450 200 \
  --hdiutil-quiet \
  moai-studio.dmg \
  target/universal/release/bundle/osx/moai-studio.app/

xcrun stapler staple moai-studio.dmg
```

`assets/dmg-background.png` 은 600x400 PNG, brand visual.

---

## 12. T12 — Windows signtool + DigiCert KeyLocker

### 12.1 release.yml step

```yaml
- name: Sign .msi
  if: matrix.kind == 'windows-x86_64'
  env:
    SM_HOST: ${{ secrets.SM_HOST }}
    SM_API_KEY: ${{ secrets.SM_API_KEY }}
    SM_CLIENT_CERT_FILE: cert.p12
    SM_CLIENT_CERT_PASSWORD: ${{ secrets.SM_CLIENT_CERT_PASSWORD }}
    SM_CODE_SIGNING_CERT_SHA1_HASH: ${{ secrets.SM_CODE_SIGNING_CERT_SHA1_HASH }}
  shell: pwsh
  run: |
    # Setup KeyLocker
    Invoke-WebRequest -Uri "https://one.digicert.com/signingmanager/api-ui/v1/releases/Keylockertools-windows-x64.msi/download" `
      -Headers @{ "x-api-key" = "$env:SM_API_KEY" } -OutFile Keylockertools-windows-x64.msi
    msiexec /i Keylockertools-windows-x64.msi /quiet /qn
    & "C:\\Program Files\\DigiCert\\DigiCert Keylocker Tools\\smctl.exe" healthcheck

    # Sign
    & signtool sign /sha1 $env:SM_CODE_SIGNING_CERT_SHA1_HASH `
      /tr http://timestamp.digicert.com /td sha256 /fd sha256 `
      target\\wix\\moai-studio-0.1.0-x86_64.msi
    & signtool verify /pa /v target\\wix\\moai-studio-0.1.0-x86_64.msi
```

DigiCert KeyLocker 는 hardware-bound key 를 cloud 에서 관리 (signtool 이 직접 호출).

### 12.2 검증

- `signtool verify /pa /v` 의 exit code = 0 (AC-PK-7).
- 깨끗한 Windows 10/11 VM 에서 `.msi` 첫 실행 시 SmartScreen 경고 없음 (manual e2e).

---

## 13. T13 — (옵션) Linux GPG sign

GPG key 가 GitHub secret 에 등록된 경우만 활성:

```yaml
- name: GPG sign .deb and .AppImage
  if: matrix.kind == 'linux-x86_64' && env.GPG_PRIVATE_KEY != ''
  env:
    GPG_PRIVATE_KEY: ${{ secrets.GPG_PRIVATE_KEY }}
    GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}
  run: |
    echo "$GPG_PRIVATE_KEY" | gpg --batch --import
    gpg --batch --yes --pinentry-mode loopback --passphrase "$GPG_PASSPHRASE" \
      --armor --detach-sign moai-studio_0.1.0_amd64.deb
    gpg --batch --yes --pinentry-mode loopback --passphrase "$GPG_PASSPHRASE" \
      --armor --detach-sign moai-studio-x86_64.AppImage
```

산출: `.asc` 파일 2 개 (옵션, AC 직접 매핑 없음).

---

## 14. T14 — USER-DECISION-PK-A (auto-update 메커니즘)

**결정 (2026-04-27): RESOLVED → 옵션 (a) 자체 (GitHub Releases JSON manifest + Ed25519). 외부 의존 0, Rust 100~200 LOC.**
T15 ~ T21 진입 가능. 단, MS-3 은 MS-2 (서명) 가 BLOCKED 인 동안 sprint 진입 보류. v0.1.0 출시 직후 또는 v0.1.x 패치 sprint 후보.

### 14.1 호출

[USER-DECISION-REQUIRED: auto-update-v3-011-ms3]

질문 (AskUserQuestion):
- "자동 업데이트 메커니즘은?"
- (a) **권장: 자체 (GitHub Releases JSON manifest)** — 외부 의존 0, Rust 100~200 LOC, Ed25519 서명 검증.
- (b) Sparkle (macOS) + WinSparkle (Win) + 자체 (Linux) — 검증된 라이브러리. Objective-C bridge + 분기.
- (c) skipping auto-update for v0.1.0 — v0.2.0 에서 추가.

### 14.2 결정 기록

(c) 결정 시 T15 ~ T19 skip, T20 의 update.json 만 생성 (앱이 polling 안 함 — 사용자 수동 다운로드).

---

## 15. T15 — Ed25519 keypair + secret/public key 등록

### 15.1 keypair 생성 (사용자 작업)

```bash
# 한 번만 실행 — 결과물을 안전하게 보관
cargo install ed25519-keygen-cli
ed25519-keygen --output keypair
# keypair.priv (32 bytes, hex) → GitHub secret ED25519_PRIVATE_KEY
# keypair.pub (32 bytes, hex) → assets/update-pubkey.bin (앱 embed)
```

private key 는 절대 repo 에 커밋 금지. password manager + GitHub Actions secret 만.

### 15.2 신규 파일 `assets/update-pubkey.bin`

32-byte raw public key (Ed25519). app crate 가 `include_bytes!("../../assets/update-pubkey.bin")` 으로 embed.

---

## 16. T16 — 신규 crate `moai-studio-updater`

### 16.1 변경 대상

워크스페이스 `Cargo.toml` 의 `[workspace] members = [...]` 에 `"crates/moai-studio-updater"` 추가.

신규 `crates/moai-studio-updater/Cargo.toml`:

```toml
[package]
name = "moai-studio-updater"
version = "0.1.0"
edition = "2021"

[dependencies]
ed25519-dalek = "2"
semver = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
reqwest = { version = "0.12", features = ["rustls-tls"] }
tokio = { version = "1", features = ["fs", "process", "sync", "time"] }
chrono = { version = "0.4", features = ["serde"] }
url = "2"
thiserror = "1"
tracing = "0.1"

[dev-dependencies]
tempfile = "3"
mockito = "1"
```

### 16.2 신규 모듈

- `src/lib.rs` — public API export (UpdateClient, UpdateManifest, PlatformKey).
- `src/manifest.rs` — UpdateManifest serde struct.
- `src/client.rs` — HTTP polling + 24h cooldown.
- `src/verify.rs` — sha256 + Ed25519 검증.
- `src/apply/{macos,windows,linux}.rs` — platform-specific update path.

---

## 17. T17 — update polling

### 17.1 `crates/moai-studio-updater/src/client.rs` 핵심

```rust
pub struct UpdateClient {
    manifest_url: Url,
    client: reqwest::Client,
    last_poll: Mutex<Option<Instant>>,
    poll_interval: Duration, // 24h
}

impl UpdateClient {
    pub async fn poll(&self) -> Result<Option<UpdateManifest>, UpdateError> {
        // 24h cooldown check
        if let Some(last) = *self.last_poll.lock().await {
            if last.elapsed() < self.poll_interval {
                return Ok(None);
            }
        }

        let response = self.client.get(self.manifest_url.clone())
            .timeout(Duration::from_secs(10))
            .send().await?;

        let manifest: UpdateManifest = response.json().await?;
        *self.last_poll.lock().await = Some(Instant::now());

        // semver 비교
        let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
        if manifest.version > current {
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }
}
```

### 17.2 startup polling

app crate 의 main 진입점 (tokio main task):

```rust
let updater = UpdateClient::new(...);
tokio::spawn(async move {
    if let Ok(Some(manifest)) = updater.poll().await {
        ui_notify_update_available(manifest).await;
    }
});
```

---

## 18. T18 — UI notification

### 18.1 격리된 신규 디렉터리

app crate 의 `src/update_ui/` 디렉터리 신규 — 본 SPEC 의 유일한 UI 측 변경. terminal/panes/tabs core (RG-PK-7.6) 와 격리.

```
crates/moai-studio-app/src/update_ui/
├── mod.rs
├── notification.rs    # in-app notification component
└── progress.rs        # download progress UI
```

### 18.2 notification.rs

`UpdateAvailableNotification` GPUI component:
- 상단 toast 형태.
- 텍스트: "v{new_version} 이 출시되었습니다. 변경사항 보기 / 지금 업데이트 / 나중에".
- 버튼 클릭 → `UpdateClient::download_and_verify` + `apply_update`.

기존 design token 재사용 (`status.info`, `text.primary`).

---

## 19. T19 — platform-specific apply_update

### 19.1 `apply/macos.rs`

```rust
pub async fn apply_update_macos(downloaded_dmg: &Path) -> Result<(), UpdateError> {
    // 1. mount .dmg
    let mount_output = Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-quiet", downloaded_dmg.to_str().unwrap()])
        .output().await?;
    let mount_point = parse_mount_point(&mount_output.stdout)?;

    // 2. .app 위치 확인
    let new_app = mount_point.join("moai-studio.app");
    let current_app = std::env::current_exe()?
        .ancestors().nth(2)
        .ok_or(UpdateError::AppPathResolution)?;

    // 3. 기존 .app backup, 신규 .app 으로 교체
    let backup = current_app.with_extension("app.bak");
    fs::rename(&current_app, &backup).await?;
    fs::copy_dir(&new_app, &current_app).await?;

    // 4. unmount
    Command::new("hdiutil").args(["detach", "-quiet", mount_point.to_str().unwrap()]).status().await?;

    // 5. 사용자에게 재시작 prompt
    show_restart_prompt().await?;
    Ok(())
}
```

### 19.2 `apply/windows.rs`

```rust
pub async fn apply_update_windows(downloaded_msi: &Path) -> Result<(), UpdateError> {
    // msiexec /i ... /qb (basic UI) — 사용자에게 진행률 표시
    let status = Command::new("msiexec")
        .args(["/i", downloaded_msi.to_str().unwrap(), "/qb", "/norestart"])
        .status().await?;
    if !status.success() {
        return Err(UpdateError::MsiExecFailed(status));
    }
    show_restart_prompt().await?;
    Ok(())
}
```

### 19.3 `apply/linux.rs`

```rust
pub async fn apply_update_linux(downloaded: &Path, kind: LinuxKind) -> Result<(), UpdateError> {
    match kind {
        LinuxKind::AppImage => {
            // self-replace: 현재 AppImage 를 backup → 신규로 교체 → 재실행
            let current_exe = std::env::current_exe()?;
            let backup = current_exe.with_extension("AppImage.bak");
            fs::rename(&current_exe, &backup).await?;
            fs::rename(downloaded, &current_exe).await?;
            Command::new(&current_exe).spawn()?;
            std::process::exit(0);
        }
        LinuxKind::Deb => {
            // admin 권한 부재 → 안내만
            show_manual_install_guidance("https://github.com/GoosLab/moai-studio/releases/latest").await?;
            Ok(())
        }
    }
}
```

---

## 20. T20 — release.yml `release` aggregation job

### 20.1 신규 스크립트 `scripts/generate-update-json.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"
ED25519_PRIVATE_KEY="$2"

cat > update.json <<EOF
{
  "version": "$VERSION",
  "released_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "notes_url": "https://github.com/GoosLab/moai-studio/releases/tag/v$VERSION",
  "platforms": {
EOF

for platform in macos-universal linux-x86_64-deb linux-x86_64-appimage windows-x86_64-msi; do
  case "$platform" in
    macos-universal)        FILE="moai-studio.dmg" ;;
    linux-x86_64-deb)       FILE="moai-studio_${VERSION}_amd64.deb" ;;
    linux-x86_64-appimage)  FILE="moai-studio-x86_64.AppImage" ;;
    windows-x86_64-msi)     FILE="moai-studio-${VERSION}-x86_64.msi" ;;
  esac

  SHA256=$(shasum -a 256 "$FILE" | awk '{print $1}')
  SIGNATURE=$(echo -n "$SHA256" | ed25519-sign --key "$ED25519_PRIVATE_KEY" | base64)
  URL="https://github.com/GoosLab/moai-studio/releases/download/v${VERSION}/${FILE}"

  cat >> update.json <<EOF
    "$platform": {
      "url": "$URL",
      "sha256": "$SHA256",
      "signature": "$SIGNATURE"
    }$( [ "$platform" != "windows-x86_64-msi" ] && echo "," )
EOF
done

cat >> update.json <<EOF
  }
}
EOF
```

### 20.2 release.yml `release` job

```yaml
release:
  needs: build
  runs-on: ubuntu-22.04
  if: ${{ !github.event.inputs.dry_run }}
  steps:
    - uses: actions/checkout@v4
    - uses: actions/download-artifact@v4
      with:
        path: artifacts/
    - name: Generate update.json
      env:
        ED25519_PRIVATE_KEY: ${{ secrets.ED25519_PRIVATE_KEY }}
      run: |
        VERSION="${GITHUB_REF#refs/tags/v}"
        bash scripts/generate-update-json.sh "$VERSION" "$ED25519_PRIVATE_KEY"
    - uses: softprops/action-gh-release@v2
      with:
        files: |
          artifacts/**/*.dmg
          artifacts/**/*.deb
          artifacts/**/*.AppImage
          artifacts/**/*.msi
          update.json
        prerelease: ${{ contains(github.ref_name, '-rc') }}
```

---

## 21. T21 — Release Drafter publish trigger

### 21.1 release.yml step

```yaml
- name: Trigger Release Drafter publish
  uses: release-drafter/release-drafter@v6
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  with:
    publish: true
    tag: ${{ github.ref_name }}
```

draft 가 이미 누적된 상태이므로 publish 트리거만으로 release notes body 가 채워진다 (CLAUDE.local.md §5.3 의 카테고리 매핑 그대로 carry).

### 21.2 검증

- GitHub Releases 페이지의 v0.1.0 publish 직후 body 에 `## Added`, `## Fixed` 등 카테고리 + 최소 1 PR 항목 (AC-PK-12).

---

## 22. T22 — 코드베이스 무변경 regression check

### 22.1 CI step

T7 의 path-filter diff 와 동일. release.yml 의 `verify-version` job 에 추가 또는 별 step:

```yaml
- name: Verify functional codebase unchanged
  run: |
    set -euo pipefail
    BASE=$(git merge-base HEAD origin/develop)
    DIFF_PATHS=$(git diff --name-only "$BASE" HEAD)
    echo "$DIFF_PATHS" | tee /tmp/diff-files
    # 허용 path: .github/, Cargo.toml, scripts/, assets/, wix/,
    #            crates/moai-studio-updater/, crates/moai-studio-app/src/update_ui/
    # 금지 path: 그 외 crates/
    if echo "$DIFF_PATHS" | grep -E '^crates/' \
       | grep -vE '^crates/moai-studio-updater/' \
       | grep -vE '^crates/moai-studio-app/src/update_ui/'; then
      echo "ERROR: Functional codebase changed. SPEC-V3-011 must not modify v3 functional crates."
      exit 1
    fi
```

본 step 이 fail 하면 PR 차단 (RG-PK-7.6 / REQ-PK-065 enforcement).

---

## 23. T23 — progress.md 갱신 + commit

### 23.1 progress.md 신규

`.moai/specs/SPEC-V3-011/progress.md`:

```markdown
# SPEC-V3-011 Progress

## MS-1 (Priority: High)
- [x] T0: USER-DECISION-PK-C RESOLVED (a) v{x.y.z} (2026-04-27)
- [ ] T1 ~ T7: ...

## MS-2 (Priority: High) — BLOCKED
- [x] T8: USER-DECISION-PK-B RESOLVED (b) 미보유 → MS-2 BLOCKED (2026-04-27)
- [ ] T9 ~ T13: 인증서 보유 시점까지 보류

## MS-3 (Priority: High) — Deferred (post-v0.1.0)
- [x] T14: USER-DECISION-PK-A RESOLVED (a) 자체 manifest + Ed25519 (2026-04-27)
- [ ] T15 ~ T22: Ed25519 keypair 생성 + secret 등록 후 진입

## AC Status
| AC | Status |
|----|--------|
| AC-PK-1 | PENDING |
| AC-PK-2 | PENDING |
| ... | ... |
```

### 23.2 commit

implement 단계 commit 형식 (CLAUDE.local.md §4.1):

```
docs(spec): SPEC-V3-011 Cross-platform Packaging v1.0.0 (research/plan/spec)

🗿 MoAI <email@mo.ai.kr>
```

본 SPEC 작성 단계에서는 progress.md 미생성 (implement 진입 시 manager-ddd 가 생성).

---

## 24. 의존성 / 차단 사항 — implement 진입 prerequisite

### 24.1 외부 차단 (research §12 carry)

implement 진입은 다음 차단 해소가 전제:

1. **CI billing 해소** — GitHub Actions private repo macOS runner 비용. 사용자 결정 시까지 release.yml 실 트리거 보류.
2. **서명 인증서 보유** — USER-DECISION-PK-B 의 (a) 결정 시까지 MS-2 진입 불가.
3. **Apple Developer Program 등록** — Team ID, Apple ID, app-specific password.
4. **DigiCert KeyLocker 계약** — Windows MS-2 prerequisite.
5. **(옵션) GPG key** — Linux apt repo 운영 시.

### 24.2 v3 functional SPEC 진척도

- v3 functional SPEC (V3-001 ~ V3-010) 의 AC pass count 합산 80%+ 달성.
- 사용자가 v0.1.0-rc1 결정 (release branch 분기 직전).

### 24.3 USER-DECISION 4 게이트

- [x] T0: USER-DECISION-PK-C RESOLVED (a) v{x.y.z}
- [x] T8: USER-DECISION-PK-B RESOLVED (b) 미보유 → MS-2 BLOCKED
- [x] T14: USER-DECISION-PK-A RESOLVED (a) 자체 manifest + Ed25519
- USER-DECISION-PK-D RESOLVED: opt-out (도입 안함). v0.2.0+ 재논의.

---

## 25. 리스크 완화 매핑

| 위험 (spec.md §11) | 완화 task |
|--------------------|----------|
| R-PK-1 (Apple Notarization 정책 변경) | T9 entitlements + T10 notarytool log inspection |
| R-PK-2 (EV cert 미보유) | T8 USER-DECISION-PK-B gate |
| R-PK-3 (Metal entitlements) | T9 entitlements file (allow-jit) |
| R-PK-4 (Linux distro fragmentation) | T5 ubuntu-22.04 + ubuntu-24.04 dual matrix 옵션 |
| R-PK-5 (lipo 실패) | T3 lipo -info 검증 step |
| R-PK-6 (GitHub Releases throttle) | T17 24h cooldown + etag |
| R-PK-7 (Windows admin 권한) | T19 user-install 옵션 + 안내 |
| R-PK-8 (Ed25519 key leak) | T15 hardware token + secret env-only + rotation 정책 |
| R-PK-9 (Cargo.toml version 불일치) | T1 verify-version job |
| R-PK-10 (Xcode 버전 변경) | release.yml `actions/setup-xcode@v1` pin |
| R-PK-11 (.deb dependency 누락) | T4 + dpkg -i smoke (AC-PK-2/3) |
| R-PK-12 (CI billing 해소 지연) | T7 dry-run + workflow_dispatch |
| R-PK-13 (DigiCert 가격 인상) | USER-DECISION-PK-B 재논의 |
| R-PK-14 (delta update 부재) | v0.2.0 SPEC follow-up |

---

## 26. 결론

본 plan.md 는 SPEC-V3-011 spec.md 의 RG-PK-1 ~ RG-PK-7, AC-PK-1 ~ AC-PK-12 를 23 task (T0 ~ T23) × 3 milestone 으로 분할한다. 각 task 는 산출 파일과 AC 매핑을 명시하며, 외부 차단 (CI billing, 서명 인증서) 해소 후 진입 가능하다.

특별히 강조할 사항:

- **MS-1 (unsigned 빌드)** 은 외부 차단 없이 진입 가능. v3 functional SPEC complete 즉시 진행 가능.
- **MS-2 (서명 + notarization)** 은 PK-B (b) 미보유 결정으로 v0.1.0 범위 제외. 인증서 보유 시점에 별 sprint 진입.
- **MS-3 (auto-update)** 은 PK-A (a) 자체 manifest 결정. Ed25519 keypair 생성 (`age` 또는 `ssh-keygen -t ed25519`) 후 GitHub Actions secret 등록 시 진입 가능. v0.1.0 출시 직후 또는 v0.1.x 패치 sprint 후보.
- **코드베이스 무변경 regression** 은 T22 의 CI assertion 으로 자동 enforcement (RG-PK-7.6).
- **CI billing 해소 전까지** release.yml 의 실 트리거 (tag push) 는 보류, dry-run + workflow_dispatch 만 활용.

implement 는 별도 feature 브랜치 (`feature/SPEC-V3-011-packaging`) 에서 진행. 본 plan.md 는 manager-ddd 의 입력이다.
