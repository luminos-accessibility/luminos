# Tauri 2.0 Bundler Research (2026-03-17)

## Verified Native Bundle Targets
From JSON schema v2.10.3 (schema.tauri.app/config/2) and config reference (v2.tauri.app/reference/config/):
- `deb` - Debian package (.deb)
- `rpm` - RPM package (.rpm)
- `appimage` - AppImage (.AppImage)
- `nsis` - NSIS installer (.exe)
- `msi` - MSI/WiX installer (.msi)
- `app` - macOS application bundle (.app)
- `dmg` - macOS disk image (.dmg)

## NOT Native Targets
- Flatpak: Open feature request #3619. No docs page (404 at v2.tauri.app/distribute/flatpak/). Must use flatpak-builder externally.
- Snap: Docs at v2.tauri.app/distribute/snapcraft/. Workflow: build .deb with Tauri, repackage with snapcraft externally.
- AUR: Docs mention it as distribution option but uses PKGBUILD externally.

## Signing Summary
- macOS: Built-in codesign + notarization via xcrun notarytool. Env: APPLE_SIGNING_IDENTITY, APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, APPLE_API_ISSUER/KEY/KEY_PATH or APPLE_ID/PASSWORD/TEAM_ID.
- Windows: Built-in via signtool.exe (Authenticode). Config: certificateThumbprint, digestAlgorithm, timestampUrl, signCommand. Env: WINDOWS_CERTIFICATE, WINDOWS_CERTIFICATE_PASSWORD. Supports EV certs.
- Linux AppImage: GPG via gpg/gpg2. Env: SIGN=1, SIGN_KEY, APPIMAGETOOL_SIGN_PASSPHRASE, APPIMAGETOOL_FORCE_SIGN=1.
- Linux RPM: GPG. Env: TAURI_SIGNING_RPM_KEY, TAURI_SIGNING_RPM_KEY_PASSPHRASE.
- Linux .deb: No native signing support.

## Updater Plugin
- Name: tauri-plugin-updater / @tauri-apps/plugin-updater
- Config: plugins.updater.pubkey + plugins.updater.endpoints
- Signing: Ed25519 via `tauri signer generate`. Env: TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD.
- createUpdaterArtifacts: true | false | "v1Compatible"
