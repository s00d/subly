# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.4] - 2026-05-13

### Added

- Shared **`dashboardIpc` TypeScript types** for dashboard command payloads so `commandClient` stays free of circular imports with `dashboardClient`.
- **`readClipboardText()`** in `useClipboard`, backed by `@tauri-apps/plugin-clipboard-manager` with a **`navigator.clipboard.readText()`** fallback for paste flows (including the AI assist dialog).

### Changed

- **Dashboard client** refactored to use the shared IPC types and leaner invoke helpers.
- **`commandClient` / `invokeCommand`** extended for dashboard payloads and **`app_ready`** lifecycle wiring in **`App.vue`**.
- **Frontend secrets** now use **`tauri-plugin-keyring-store-api` only**; legacy **`secure_storage_*`** Tauri commands were removed from Rust while keeping the same logical key prefix, so existing keychain entries remain valid.
- **`keyring_store`** Rust module simplified in line with dropping the redundant command layer.
- **AI Smart dialog**, **subscriptions / expenses** pages, **`aiClient`**, **`aiConfigStore`**, and **`SubscriptionForm`** updated for the new IPC and clipboard behavior.
- **Tauri capabilities** updated where clipboard read is required; **iOS and widget `Info.plist`** metadata aligned with the current app build.
- **Locale files** refreshed for new or adjusted copy across all supported languages.
- **`pnpm-lock.yaml`** and **`Cargo.lock`** updated with dependency refreshes tied to the release.

### Removed

- **`src-tauri/src/commands/secure_storage.rs`** — redundant now that the UI talks to the keyring plugin directly.

## [1.0.3] - 2026-05-12

### Added

- **AI-powered assistance** (opt-in, off by default), exposed through a single
  `ai_smart_input` backend command and unified `AiSmartDialog` on the frontend.
  One entry point handles all four capabilities — the model decides whether
  the input is one item or a list based on the picture / file / text:
  - **Quick-add subscription** — natural-language input ("Telegram premium 300₽ monthly card") that extracts price, currency, billing cycle, category, payment method and tags using the existing catalog as context.
  - **Quick-add expense** — same flow for one-off transactions with description, amount, date and tag parsing.
  - **Receipt OCR** — point a vision-capable model at a photo / scan of a receipt; line items, totals, currency, merchant and date come back as a draft expense.
  - **Statement import** — bulk import of bank/card statements (CSV, XLSX, PDF, JSON) with heuristic pre-parsing and per-row preview before saving.
  Backend uses the `aisdk` crate for text generation, direct `reqwest` for vision endpoints, prompted-JSON output (instead of `aisdk` schema validation) and a robust JSON parser that tolerates LLM markdown fences.
- **Seven AI providers** out of the box: OpenRouter, OpenAI, OpenAI-compatible (Ollama / custom), Groq, DeepSeek, Google Gemini (OpenAI-compatible endpoint), Mistral — each with curated recommended-model presets and vision-capability flags.
- New **AI Settings panel** with master toggle, per-feature switches, provider/model picker, custom model input, test-connection button, and an auto-enable shortcut on first successful save.
- **Single AI header button** on subscriptions and expenses pages: opens the AI assist dialog when AI is ready, or shortcuts to settings when no provider/key is configured.
- **Drag-and-drop, paste-from-clipboard and image preview** in the AI assist dialog. Drops are wired through Tauri's `onDragDropEvent` (the WebView's HTML5 `drop` event is suppressed in Tauri 2); files are read via `@tauri-apps/plugin-fs` and capped at 64 MiB. Images get a checkerboard-backed preview with a one-tap clear button; non-image files render as a rich row with icon + size + filename.
- **Responsive AI dialog layout**: larger touch targets, full-width dropzone on mobile, wrap-friendly result header with thumbnail, and a 44×44 cancel target during parsing.
- **`SecretInput` component** — reusable password-style field that displays a saved-value mask, clears on focus, and disables Save until the user actually edits.
- **Backend signal for saved secrets**: `ProviderField.has_saved_value` is now returned by CloudSync provider descriptors so the UI can show a masked field with "API key saved" instead of an empty input.
- **GitHub Actions workflow + setup doc** for publishing Android AAB to Google Play (signed release, internal/alpha/beta/production tracks, release notes auto-extracted from this changelog, native debug symbols, mapping upload).
- **Splash screens** on every platform: dedicated splash window on desktop with theme-aware background, native iOS `LaunchScreen.storyboard` with brand logo, and Android `androidx.core.splashscreen` flow with light/dark color variants.
- **Skeleton loaders** for the subscriptions list while data is loading.
- **`LazyVChart`** wrapper for `vue-echarts` that delays mount until the container has a real size, eliminating ECharts "Can't get DOM width or height" race.
- **Animated mobile tab bar**: sliding pill indicator for the active item, scale-on-tap feedback, haptic vibration on navigation, translucent blurred background, and stronger shadow.
- **Backdrop-blur overlay** for the mobile sidebar drawer and safe-area-aware insets for header / drawer / main content (`--sat`, `--sab` CSS variables).
- **iOS WKWebView full-screen fix** (`ios_webview_fix.rs`) so the web view extends across `safeAreaLayoutGuide` and the bottom navigation actually sits on the bottom of the screen.
- Improved empty state on **Calendar** page with illustration and new i18n strings.
- New shared `brand-logo.svg` and regenerated desktop icons (macOS `.icns`, Windows `.ico`, Linux PNGs) with transparent rounded corners; iOS app icons kept square per Apple guidelines.

### Changed

- **Keyring migration**: replaced the legacy v3 `keyring` crate (which silently fell back to an in-memory mock without explicit platform features — causing API keys and OAuth tokens to "disappear" after each restart) with `keyring-core` 1.x and platform-specific backends — `apple-native-keyring-store`, `windows-native-keyring-store`, `dbus-secret-service-keyring-store`, `android-native-keyring-store`. AI keys, sync credentials and Telegram tokens now reliably persist on every platform without the previous redb fallback layer.
- **Telegram bot token** and **exchange-rate provider API keys** migrated to the new `SecretInput` flow — fields no longer pre-fill the actual secret into the input, just show a masked placeholder.
- **CloudSync** secret fields (Dropbox app secret, WebDAV password, ...) use `SecretInput` with `has_saved_value` from the backend, so the form correctly reflects "credentials already stored" without leaking the values.
- **Android release builds are now signed** via `keystore.properties` (local dev) or env vars (CI). Unsigned release stays as fallback for dev machines without a keystore.
- Android release bundle ships with **native debug symbols** (`ndk.debugSymbolLevel = "SYMBOL_TABLE"`) so Play Console can deobfuscate Rust crash traces.
- Major **Rust error-handling refactor**: all `commands/*` migrated to a unified `AppError` enum (new `src/errors.rs`) with `From` impls for `redb` / IO / serde / keyring errors, enabling clean `?` propagation across the backend.
- **AI prompts now ship the user's real taxonomy** — categories, enabled payment methods and a tag pool (favourites first, then by `sortOrder`, capped at 30) are rendered directly into every `ai_smart_input` system prompt. The model is told to reuse existing names instead of inventing new ones, which cuts duplicate categories/tags on save and improves payment-method matching.
- **`commands/ai/extract/smart.rs` split** into a `smart/` module (`mod.rs` dispatch, `expense.rs`, `subscription.rs`) so each surface owns its parsing/mapping/heuristics in isolation. The prior single-file extractor is gone along with the obsolete per-feature commands (`ai_quick_add_*`, `ai_extract_receipt`, `ai_import_statement_file`, `ai_import_subscriptions_file`).
- **Prompt fragments** gained reusable `payment_method_rules` and `tag_rules` helpers in `commands/ai/prompts/fragments.rs`; the `category_rules` helper now gracefully degrades to "omit the field" when the catalog is empty.
- **iOS widget snapshot export** moved to a background thread with a shared `ureq` agent and deduplication flags. The HTTP fetch for subscription logos no longer blocks app startup, sync-pull, or subscription save.
- Settings → Appearance: color-theme swatches now render as perfect circles on all platforms (iOS Safari previously squashed them into ovals) and the row is centered so the trailing gap on the right no longer looks crooked.
- Dashboard **Stats cards** redesigned into a clean tile layout.
- Subscriptions list: compact view collapses metadata progressively on small screens to keep everything on a single line; default view leaves more room for long names on mobile.
- ECharts is now tree-shaken with `LegacyGridContainLabel` (silences the ECharts 6+ deprecation warning) and lazy-loaded from the dashboard page instead of the global bundle, so the splash screen disappears noticeably faster on cold start.
- Chart theme is transparent by default, so widgets blend with both light and dark surfaces.
- Tauri / npm / cargo dependencies refreshed to current versions (Tauri 2.11.1, plugins, Vite 8, Vue 3.5, Tailwind 4, etc.).

### Fixed

- **API keys / OAuth tokens no longer "disappear" after app restart** — see Keyring migration above. Affects AI provider keys, sync credentials and Telegram bot tokens on all platforms.
- **AI feature visibility race** on subscriptions/expenses pages: the Pinia `features` slot was a `reactive()` object instead of a `ref()`, so `storeToRefs()` couldn't track it and the header button stayed grey-disabled until a full reload. Now correct everywhere.
- **AI prompts speak the user's language**: prompts are kept English (more reliable LLM behaviour) but explicitly instruct the model to write free-form fields (descriptions, tag names, notes) in the user's UI locale.
- **Android crash on launch** ("Unable to resolve local data directory"). On Android the `dirs` crate has no `$HOME` / XDG paths and always returned `None`, panicking the setup hook. The DB path is now resolved via Tauri's path API on Android (`Context.getFilesDir()` → `/data/user/0/<package>/files/`); desktop + iOS keep the existing `~/Library/Application Support/Subly/` location so no data migration is required.
- **Android Gradle build** "Cannot find module '...src-tauri/tauri'" fixed by resolving the Tauri CLI through the project's package manager (`TAURI_CLI_PACKAGE_MANAGER` → pnpm/npm/yarn/node) instead of invoking `node tauri` directly.
- **iOS bottom tab bar** no longer floats with a gap below it — WKWebView is now stretched to `view.bounds` and `contentInsetAdjustmentBehavior = .never`, so `position: fixed; bottom: 0` reaches the actual bottom of the screen.
- Mobile sidebar no longer overlaps the status bar / Dynamic Island; header and main content respect `safe-area-inset-*`.
- ECharts **"Expenses by Tags"** widget rendered empty: `itemStyle.borderColor` referenced a CSS variable that ECharts cannot parse — the value is now resolved with `getComputedStyle` and re-resolved on theme change.
- ECharts **"Expenses by Day of Week"** widget no longer overflows below its card on mobile (conflicting `min-h-*` constraints removed).
- Charts no longer paint a solid black background in dark mode.
- Content Security Policy updated to allow Tauri 2 mobile IPC (`ipc:` and `http://ipc.localhost`), fixing notification permission probes on iOS / Android.

## [1.0.2] - 2026-05-09

### Changed

- Bumped app version to `1.0.2`.

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

[1.0.4]: https://github.com/s00d/subly/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/s00d/subly/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/s00d/subly/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/s00d/subly/compare/v1.0.0...v1.0.1
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
