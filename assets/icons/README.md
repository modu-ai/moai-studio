# moai-studio Icon Assets

## Status

| File | Status | Notes |
|------|--------|-------|
| `256x256/moai-studio.png` | PLACEHOLDER (dark gray 256x256) | Replace with real icon before v0.1.0 GA |
| `icon.icns` | NOT PROVIDED | Deferred to MS-2 — requires Apple tooling (`iconutil`) |

## Requirements for v0.1.0 GA

### macOS (.app bundle)

`icon.icns` is required for a production macOS .app bundle. To generate:

1. Create PNG assets at all required resolutions (16, 32, 64, 128, 256, 512, 1024 px)
2. Place them in an `icon.iconset/` directory with names: `icon_16x16.png`, `icon_32x32.png`, etc.
3. Run: `iconutil -c icns icon.iconset -o assets/icons/icon.icns`
4. Update `[package.metadata.bundle] icon` in `crates/moai-studio-app/Cargo.toml` from PNG to ICNS

### Linux (.deb + .AppImage)

`256x256/moai-studio.png` is used for:
- `.deb` package: installed to `/usr/share/icons/hicolor/256x256/apps/`
- `.AppImage`: passed to linuxdeploy as `--icon-file`
- `.desktop` file: referenced as `Icon=moai-studio`

Replace the placeholder with a real 256x256 PNG before v0.1.0 GA.

### Windows (.msi)

cargo-wix reads icon from the binary's resource section. Embedding an icon requires
the `embed-resource` or `winres` crate (deferred to MS-2, no new Cargo deps in MS-1).

## Design Notes

- App identifier: `kr.ai.mo.moai-studio`
- Copyright: `Copyright © 2026 MoAI`
- The MoAI stone (🗿) motif is recommended as the primary icon element
