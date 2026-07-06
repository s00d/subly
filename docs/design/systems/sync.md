# Cloud sync

Subly keeps a single **sync blob** per cloud provider. UI talks to `src/services/syncClient.ts` → Tauri `sync_*` commands → `src-tauri/src/commands/sync/orchestrator.rs` → provider modules.

## Providers

| Provider | Module | Auth |
|----------|--------|------|
| iCloud | `providers/icloud.rs` + `icloud_native.rs` | Apple ubiquity container (no OAuth) |
| Google Drive | `providers/gdrive.rs` | OAuth PKCE |
| Dropbox | `providers/dropbox.rs` | OAuth |
| OneDrive | `providers/onedrive.rs` | OAuth |
| WebDAV | `providers/webdav.rs` | URL + basic auth (keyring) |

Constants: `src-tauri/src/commands/sync/config.rs`.

## Sync file

| Constant | Value |
|----------|-------|
| `SYNC_FILENAME` | `subly-sync-v2.sublysync` |
| `SYNC_FILENAME_LEGACY_JSON` | `subly-sync-v1.json` (read/migrate only) |
| `SYNC_ICLOUD_FOLDER` | `Subly-v1` |
| `APP_ICLOUD_CONTAINER_ID` | `iCloud.com.s00d.subly` |

**Wire format (v2):** `SS2` magic + zstd(postcard envelope). See `wire.rs`. Legacy JSON is rejected with `sync_pull_legacy_format`.

## iCloud (ubiquity, not bookmarks)

iCloud sync uses the **app ubiquity container**, not security-scoped bookmarks or `UIDocumentPicker` / `NSOpenPanel`. Bookmarks do not sync across devices and are not used for the iCloud provider.

```
iCloud.com.s00d.subly/Documents/Subly-v1/subly-sync-v2.sublysync
```

Visible in Files as **Subly → Subly-v1** when `NSUbiquitousContainers` is set and iCloud Drive is on.

### Apple I/O layer (`icloud_native.rs`)

- `ubiquity_sync_subdir()` — resolve container, create `Documents/Subly-v1`
- `resolve_icloud_sync_file()` — v2, legacy v1, or any `subly-sync*` (via `icloud_resolve.rs`)
- `prepare_ubiquitous_item_local()` — `startDownloadingUbiquitousItem` + poll `NSURLUbiquitousItemDownloadingStatusKey`
- `coordinated_read_bytes` / `coordinated_write_bytes` — always `NSFileCoordinator`; write is atomic (`.tmp` → `fsync` → `rename`)
- macOS one-shot: `migrate_legacy_clouddocs_if_needed()` copies SS2 from old `com~apple~CloudDocs/Subly-v1` when ubiquity is empty

### Entitlements / plist

- iOS + macOS: `CloudDocuments` + `iCloud.com.s00d.subly`
- `NSUbiquitousContainers` in `subly_iOS/Info.plist`, `gen/apple/project.yml`, and `src-tauri/Info.plist` (macOS bundle merge)

### Error message keys

| Key | When |
|-----|------|
| `sync_icloud_unavailable` | Container / iCloud Drive not available |
| `sync_icloud_materializing` | File in cloud, local placeholder still downloading |
| `sync_pull_legacy_format` | Remote file is v1 JSON |
| `sync_pull_no_remote_file` | No candidate file in cloud folder |

Orchestrator maps provider I/O `Err` from `provider_download` / missing OAuth tokens to `SyncOkDto.messageKey` for pull/push/sync-now.

## Connection states

| State | `cfg.enabled` | Runtime | UI |
|-------|---------------|---------|-----|
| disconnected | false | — | provider list |
| `oauth_pending` | false | `oauth_pending_provider` set | OAuth modal + banner |
| connected | true | tokens / iCloud I/O OK | Pull / Push panel |
| setup_required | false | — | credentials form + Save |

OAuth providers (GDrive, Dropbox, OneDrive): `enabled = true` only after `sync_oauth_finish`. iCloud: `enabled` immediately on Connect. WebDAV: `enabled` when URL + username are saved.

## OAuth flow

1. User taps **Connect** → `sync_enable_provider` returns `awaitingOAuth: true` + `authUrl` (does **not** set `enabled` yet).
2. UI opens browser (`openUrl`) and shows `SyncOAuthModal`.
3. Provider redirects to `subly://oauth/callback?code=…&state=gdrive|dropbox|onedrive`.
4. `App.vue` → `finishOAuth` → `sync_oauth_finish` stores tokens, sets `enabled = true`.
5. **Cancel** → `sync_disable_provider` clears pending state.

`SyncStatusDto`: `oauthPending`, `oauthPendingProvider`.

### OAuth / provider message keys

| Key | When |
|-----|------|
| `sync_oauth_required` | Pull/push without stored tokens |
| `sync_oauth_pkce_missing` | GDrive PKCE verifier lost (restart mid-flow) |
| `sync_dropbox_credentials_missing` | Connect before Save key/secret |
| `sync_onedrive_credentials_missing` | Connect before Save client ID |
| `sync_webdav_credentials_missing` | Connect / test without URL+user |
| `sync_gdrive_unauthorized` | GDrive HTTP 401 |
| `sync_webdav_auth_failed` | WebDAV 401/403 |
| `sync_webdav_unreachable` | WebDAV network/TLS errors |

## Lifecycle

1. **Connect** — `sync_enable_provider` (credentials per `ProviderInfo.requiresSaveBeforeConnect`)
2. **OAuth finish** (if applicable) — deep link → `sync_oauth_finish`
3. **Upload** — encode local snapshot → provider upload
4. **Check** — download metadata path / compare revision
5. **Pull** — download → merge → persist → emit `app:data-changed`

WebDAV: optional `sync_webdav_test_connection` (HEAD/PROPFIND) before Connect.

Logs: filter `[subly][icloud]` in Console.app.

## Manual smoke checklist (pre-release)

### iCloud

| # | Scenario | Expected |
|---|----------|----------|
| 1 | iPhone A: Connect iCloud → Upload | Files → Subly → Subly-v1 → `subly-sync-v2.sublysync` |
| 2 | iPhone B (TestFlight): Connect → Pull | Data applied, not “no file” |
| 3 | Mac MAS: same | Container resolve, upload/pull |
| 4 | B offline → edit → A edit → B online | Revision conflict UI |
| 5 | File evicted (Settings → storage) → Pull | `sync_icloud_materializing`, not “no file” |
| 6 | iCloud off | `sync_icloud_unavailable` |
| 7 | App restart | No re-Connect required |

### All cloud providers

| # | Provider | Scenario | Expected |
|---|----------|----------|----------|
| 1 | GDrive | Connect → modal → browser → deep link | Connected panel; no false “success” before OAuth |
| 2 | Dropbox | Save keys → Connect → OAuth → Upload/Pull | Works on second device |
| 3 | OneDrive | Save client ID → Connect → OAuth → Upload/Pull | Works |
| 4 | WebDAV | Save → Test connection → Connect → Upload/Pull | Wrong password → `sync_webdav_auth_failed` |
| 5 | iCloud | After OAuth fixes | Connected panel unchanged |
| 6 | Any OAuth | Connect → close browser → Cancel | Not left in “Connected” state |

## Verification

```bash
pnpm test:backend   # includes icloud_resolve unit tests
pnpm build
```

Real-device / TestFlight smoke required; simulator may strip iCloud entitlements.

## Relation to `tauri-plugin-icloud-container`

Subly keeps a **custom** `icloud_native.rs` (objc2) instead of the community plugin because:

| | Plugin | Subly |
|---|--------|-------|
| macOS | Not implemented | Required (MAS + Developer ID) |
| Scope | Generic FS API (21 commands) | Single sync blob + orchestrator |
| Watch events | `NSMetadataQuery` / `NSFilePresenter` | Not yet (manual `sync_check_remote`) |

Ideas already ported from the plugin design:

- `ubiquityIdentityToken` → `sync_icloud_not_signed_in` vs `sync_icloud_unavailable`
- Cached container root URL (invalidate manually via `invalidate_container_cache` if needed)
- `NSURL` resource keys for download status / errors
- Directory listing before `Path::is_file` for cloud placeholders
- `sync_icloud_debug_state` Tauri command (container path, entries, resolved file)

**Future (optional):** `NSMetadataQuery` watch on `Documents/Subly-v1` to set `pending_update` when another device uploads — similar to plugin `watch_directory` + `icloud://directory-changed`. Not needed for the current pull/push UX but would reduce reliance on background polls.
