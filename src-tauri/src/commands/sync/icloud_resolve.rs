//! Pure iCloud sync filename resolution (no Apple APIs). Shared by `icloud_native` and unit tests.

use std::path::{Path, PathBuf};

use super::config::{SYNC_FILENAME, SYNC_FILENAME_LEGACY_JSON};

pub(crate) fn is_sync_candidate_name(name: &str) -> bool {
    name == SYNC_FILENAME
        || name == SYNC_FILENAME_LEGACY_JSON
        || (name.starts_with("subly-sync") && !name.ends_with(".icloud"))
}

/// Pick sync blob from directory listing (canonical v2, legacy v1, or any `subly-sync*`).
pub(crate) fn resolve_icloud_sync_file_from_names(dir: &Path, names: &[String]) -> Option<PathBuf> {
    if names.iter().any(|n| n == SYNC_FILENAME) {
        return Some(dir.join(SYNC_FILENAME));
    }
    if names.iter().any(|n| n == SYNC_FILENAME_LEGACY_JSON) {
        return Some(dir.join(SYNC_FILENAME_LEGACY_JSON));
    }
    names
        .iter()
        .find(|n| is_sync_candidate_name(n))
        .map(|n| dir.join(n))
}

#[cfg(test)]
mod tests {
    use super::{is_sync_candidate_name, resolve_icloud_sync_file_from_names};
    use std::path::Path;

    #[test]
    fn is_sync_candidate_name_accepts_v2_and_legacy() {
        assert!(is_sync_candidate_name("subly-sync-v2.sublysync"));
        assert!(is_sync_candidate_name("subly-sync-v1.json"));
        assert!(is_sync_candidate_name("subly-sync-backup.sublysync"));
        assert!(!is_sync_candidate_name("subly-sync-v2.sublysync.icloud"));
        assert!(!is_sync_candidate_name("notes.txt"));
    }

    #[test]
    fn resolve_prefers_canonical_v2_name() {
        let dir = Path::new("/tmp/subly-sync-test");
        let names = vec![
            "subly-sync-v1.json".to_string(),
            "subly-sync-v2.sublysync".to_string(),
        ];
        let resolved = resolve_icloud_sync_file_from_names(dir, &names).expect("resolved");
        assert_eq!(resolved, dir.join("subly-sync-v2.sublysync"));
    }

    #[test]
    fn resolve_falls_back_to_legacy_json_listing() {
        let dir = Path::new("/tmp/subly-sync-test");
        let names = vec!["subly-sync-v1.json".to_string()];
        let resolved = resolve_icloud_sync_file_from_names(dir, &names).expect("resolved");
        assert_eq!(resolved, dir.join("subly-sync-v1.json"));
    }

    #[test]
    fn resolve_picks_any_subly_sync_glob() {
        let dir = Path::new("/tmp/subly-sync-test");
        let names = vec!["subly-sync-old.sublysync".to_string()];
        let resolved = resolve_icloud_sync_file_from_names(dir, &names).expect("resolved");
        assert_eq!(resolved, dir.join("subly-sync-old.sublysync"));
    }
}
