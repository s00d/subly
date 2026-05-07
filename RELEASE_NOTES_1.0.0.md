## Subly 1.0.0 – Release Notes

### ⚠️ Before you update

**Make a backup first:**  
Go to **Settings → Data Management**, export a `.subly` file, install the new version, and **if anything is broken, restore from that file**.  
This update completely replaces the old sync/storage stack; the backup is your safety net.

### What actually changed (short)

- **Cloud sync is new**
  - Sync file format and merge logic are fully rewritten.
  - Old `subly-sync-v1.json` is no longer used; the first device must use **Upload to Cloud** to create a new sync file.

- **iCloud behaves the same on macOS and iOS**
  - Both platforms use the same app container `iCloud.com.s00d.subly`.
  - No more “file is in iCloud Drive but the app can’t see it”.

- **Secrets moved out of the DB**
  - Cloud tokens, API keys, subscription credentials, Telegram token, etc. are now stored in the OS keychain, not in the local database.

- **macOS build is production-grade**
  - Signed with **Developer ID Application: Pavel Kuzmin (R6VC7JTLE8)** and notarized.
  - iCloud entitlements and the provisioning profile are embedded in the `.app`, so iCloud works on end-user machines without extra manual steps.

If anything goes wrong after updating, restore from your `.subly` backup and open an issue with the sync provider (iCloud/Drive/…) and platform (iOS/macOS/Windows/Linux).


