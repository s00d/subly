# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-05-08

### Fixed

- Fixed iOS crash when tapping **Take Photo** by adding required privacy usage descriptions in `Info.plist` (`NSCameraUsageDescription`, `NSPhotoLibraryUsageDescription`).
- Fixed full reset flow to recreate default `config:settings` after wipe, preventing app startup from hanging on loading.
- Fixed exchange rates provider UX:
  - Added provider docs links in selection flow and wired opening through Tauri opener.
  - Fixed API key persistence/read flow for providers that require keys.
- Fixed currency rates UI issues:
  - Removed duplicated currencies by code in converter/rates lists.
  - Added manual rate editing in **All Rates**.
  - Added save-by-Enter while editing a rate.

### Changed

- Bumped app version to `1.0.1`.

## [1.0.0] - 2026-05-07

### ⚠️ Upgrade notice

- Before upgrading, create a backup from **Settings -> Data Management** and export a `.subly` file.
- If you upgraded without backup, you can temporarily return to the previous version, export `.subly`, then update to `1.0.0` again and restore.

### Added

- New Rust-first backend command architecture across app data, dashboard, catalogs, expenses, subscriptions, export, sync, notifications, and storage.
- New secure keychain storage layer for secrets (`keyring`) and unified secure-storage commands.
- New cloud sync provider modules for iCloud, Google Drive, Dropbox, OneDrive, and WebDAV with centralized sync orchestrator.
- New zstd-compressed sync wire transport and improved sync state/runtime tracking.
- New restart smoke tests and backend test support modules.
- New CI workflow coverage for tests and production macOS release pipeline requirements.
- New privacy policy document (`PRIVACY.md`) and release notes (`RELEASE_NOTES_1.0.0.md`).

### Changed

- **Major sync rewrite**:
  - Cloud sync internals, merge behavior, payload handling, and provider integrations were rebuilt for production usage.
  - iCloud integration was hardened for both iOS and macOS with explicit app-container behavior and coordinated file handling.
- **Security model update**:
  - Sensitive credentials/tokens were moved from local DB storage to OS keychain-backed storage.
- **macOS distribution pipeline**:
  - Build/sign/notarization configuration was updated for Developer ID + provisioning profile based release flow.
  - Tauri macOS bundle configuration now supports embedding provisioning profiles from CI/local profile paths.
- **Frontend architecture refresh**:
  - Legacy service/store modules were replaced by typed command clients and updated state stores.
  - Settings/data management flows were updated for stricter import behavior and improved cloud-sync UX.
- Version updated from `0.6.1` to `1.0.0`.

### Fixed

- Fixed cloud import command argument shape mismatch for byte-based import.
- Restricted `.subly` import selection to intended backup files only.
- Fixed Linux CI dependency gap for `gdk-sys` builds by installing required GTK/GDK dev packages.
- Fixed multiple sync/merge consistency issues tied to cross-device and cross-provider workflows.

### Removed

- Removed previous frontend-only sync/provider service implementation paths.
- Removed legacy backend migration SQL files superseded by the new state/storage flow.

## [0.6.1] - 2026-04-23

### Changed

- Improved expense/subscription icon fetch UX.
- Fixed expense edit localization issues.

## [0.6.0] - 2026-04-23

### Added

- Added multi-view modes for Currency and Calendar pages.

### Changed

- Refined mobile UX behavior in related views.

## [0.5.1] - 2026-04-23

### Changed

- Bumped app version and aligned subscription form footer controls.

## [0.5.0] - 2026-04-23

### Changed

- Improved bulk subscription import UX.
- Stabilized clipboard behavior.

## [0.4.0] - 2026-04-23

### Added

- Added persisted scope plugin.

### Changed

- Refined list icon styling.

## [0.3.0] - 2026-04-21

### Changed

- Refactored color tokens to semantic Tailwind aliases.
- Improved dashboard/widget/page runtime behavior and diagnostics.

## [0.2.0] - 2026-02-23

### Changed

- Internal stability and bug-fix release.

## [0.1.4] - 2026-02-21
## [0.1.3] - 2026-02-21
## [0.1.2] - 2026-02-11
## [0.1.1] - 2026-02-11
## [0.1.0] - 2026-02-11

[1.0.1]: https://github.com/s00d/subly/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/s00d/subly/compare/v0.6.1...v1.0.0
[0.6.1]: https://github.com/s00d/subly/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/s00d/subly/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/s00d/subly/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/s00d/subly/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/s00d/subly/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/s00d/subly/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/s00d/subly/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/s00d/subly/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/s00d/subly/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/s00d/subly/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/s00d/subly/compare/v0.1.0...v0.1.1
