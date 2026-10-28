# AGENTS — Subly

Short entry for coding agents. Full graph: [docs/README.md](docs/README.md).

## Stack

Tauri v2 · Vue 3 · TypeScript · Rust · redb · Pinia · Tailwind v4

## Read first

| Doc | Use when |
|-----|----------|
| [docs/DOCS_PROFILE.md](docs/DOCS_PROFILE.md) | Paths, verification commands |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layers, module registry |
| [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) | Find files (Vue + repo) |
| [docs/BACKEND_STRUCTURE.md](docs/BACKEND_STRUCTURE.md) | Rust / Tauri / commands / IPC |
| [docs/design/README.md](docs/design/README.md) | Subsystem behavior |

## Rules

- UI calls `src/services/*Client.ts` → Tauri `invoke`; business logic stays in `src-tauri/src/commands/`.
- Do not invent APIs — grep `#[tauri::command]` and client wrappers.
- Local-first: assume offline; sync is optional.
- Doc language: English (`doc_language` in profile).
- Version bumps: follow [.cursor/rules/app-release-versioning.mdc](.cursor/rules/app-release-versioning.mdc).

## Verify before PR

```bash
pnpm build
pnpm test:backend
pnpm test:restart-smoke
```

Optional widget JSON contract: `pnpm test:ios-widget`, `pnpm test:android-widget` (JDK 17 for Gradle).

Optional known persistence risks: `pnpm test:known-risks` — see [docs/restart-risk-matrix.md](docs/restart-risk-matrix.md).

## App Store Connect (`asc` CLI)

Use [`asc`](https://github.com/rorkai/App-Store-Connect-CLI) (Homebrew, `asc version`) for App Store work instead of driving the web UI. Auth: API-key profile `MyKey` in the system keychain — check with `asc auth doctor`, never print or copy the key.

| | |
|---|---|
| App ID | `6767137924` (`com.s00d.subly`, "Subly Money Tracker") |
| Platforms | `IOS` (iPhone + iPad), `MAC_OS` — separate versions and separate review submissions |
| Locales | `en-US` (primary), `ru` — version texts **and** app-info subtitle per locale |
| Release type | `MANUAL`: after approval run `asc versions release --version-id … --confirm` only when the owner asks |

**Safety.** Start every session read-only: `asc --read-only …` (or `ASC_READ_ONLY=1`). Mutating commands need `--confirm`; run them with `--dry-run` first. Ask the owner before anything outward-facing or binding: submitting for review, releasing, price changes, export-compliance (encryption) answers, deleting assets.

**Release flow (one version, both platforms):**

```bash
asc --read-only status --app 6767137924 --platform IOS        # and --platform MAC_OS
# Builds: bump with `pnpm bump:version X.Y.Z`, tag, then upload via CI (not locally):
gh workflow run appstore-ios.yml   --ref vX.Y.Z -f upload_to_appstore=true
gh workflow run appstore-macos.yml --ref vX.Y.Z -f upload_to_appstore=true
asc builds wait --app 6767137924 --latest
asc versions create --app 6767137924 --version X.Y.Z --platform IOS --copy-metadata-from <prev>   # same for MAC_OS
asc localizations update --version <VERSION_ID> --locale en-US --whats-new "…" --promotional-text "…"   # repeat for ru
asc versions attach-build --version-id <VERSION_ID> --build-id <BUILD_ID>
asc validate --app 6767137924 --version X.Y.Z --platform IOS          # readiness report
asc review submit --app 6767137924 --version X.Y.Z --platform IOS --latest --dry-run   # then --confirm, after OK
asc status --app 6767137924 --until review-done --timeout 48h
```

Draft "What's New" from git: `asc release-notes generate --since-tag vPREV`. Keep en-US and ru in sync; ru keywords and subtitle must be Russian.

**Screenshots.** Six screens per set (dashboard, subscriptions, expenses, calendar, rates, settings), light theme, captured from the UI mock (`pnpm dev`, URL `?lang=en|ru&theme=light|dark`) at exact sizes; repo copies live in `store-assets/`.

| Device type | Size | Repo folder |
|---|---|---|
| `IPHONE_65` | 1284×2778 | `store-assets/ios/appstore_6_5` |
| `IPAD_PRO_3GEN_129` | 2752×2064 (store) / 2048×2732 (repo) | `store-assets/ios/appstore_ipad_12_9` |
| `DESKTOP` | 2560×1600 | `store-assets/mac_app_store/mac_2560x1600` |

`asc screenshots upload --app 6767137924 --version X.Y.Z --locale en-US --path <dir> --device-type IPHONE_65 --replace --dry-run` (then `--confirm`). Check sizes with `asc screenshots sizes --all`.

**Pricing.** Paid app, base territory USA $9.99, plus manual per-country prices in tiers (~$6.99 / ~$4.99 / ~$2.99 by purchasing power, values from Apple equalizations: `asc pricing price-points equalizations --price-point <USA_POINT>`). Rules: only lower prices, never raise; do not touch countries with past sales (Kazakhstan, Georgia — check Sales and Trends before any change). **Do not use `asc pricing schedule create`** — it sets only the base price and wipes the manual territory prices; post the full schedule (base + every manual price) with `asc api POST /v1/appPriceSchedules --body-file … --confirm`. Inspect with `asc pricing schedule manual-prices` / `automatic-prices`.

**Export compliance.** Both apps use only HTTPS/TLS: `ITSAppUsesNonExemptEncryption = false` is set in `src-tauri/Info.plist` (macOS) and `src-tauri/gen/apple/subly_iOS/Info.plist` (iOS), so builds should not wait on the question. If a build still shows it unanswered, ask the owner before answering.

**Not covered by the public API** (use the web UI or `asc web`, and confirm with the owner): Sales and Trends drill-down by country, some age-rating questionnaire changes. `asc capabilities` lists what is CLI, web-only or API-limited.

## Google Play (`gplay` CLI)

[`gplay`](https://gplay.sh) (Homebrew) drives the Play Developer API; service account is stored in the keyring (`gplay auth doctor`). Package `com.s00d.subly`, personal developer account: production access needs a closed test (track `alpha`) with **≥ 12 testers for 14 days**, then "Apply for production" in the Console.

```bash
gplay tracks list --package com.s00d.subly
gplay metadata pull --package com.s00d.subly --dir <dir>          # en-US + ru-RU texts
gplay metadata apply --package com.s00d.subly --dir <dir> --dry-run   # then --confirm
gplay metadata images apply --package com.s00d.subly --dir <dir> \
  --type phoneScreenshots --type sevenInchScreenshots --type tenInchScreenshots --prune --dry-run
gplay releases upload app.aab --package com.s00d.subly --track alpha --release-name X.Y.Z \
  --release-notes-dir <dir-with-en-US.txt,ru-RU.txt> --draft --confirm
```

Known quirks (gplay 2.3.2): non-draft uploads fail with "COMPLETED release must not have fraction" — upload `--draft`, then roll out in the Console; `images list/pull` fail on `promoGraphic`, so always pass `--type`; any committed edit can reset the contact email/website — re-check with `gplay apps details view` and fix with `gplay apps details set`. Prices for a paid app are Console-only (Monetize → App pricing). Screenshot tree: `<locale>/images/<type>/NN.png`; repo copies in `store-assets/google_play/` (en) and `store-assets/google_play/ru/`.

## Doc agents

Provisioning: `doc-capture-existing-project` skill → `.cursor/agents/doc-graph-scan.md`, `doc-capture.md`.
