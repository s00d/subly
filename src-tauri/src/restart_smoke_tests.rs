#[cfg(test)]
mod tests {
    use crate::commands::export::{export_build_subly_archive, import_parse_subly_archive};
    use crate::state::{load_app_data_typed, save_app_data_typed};

    #[test]
    fn restart_smoke_subly_import_then_reload_keeps_data() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let before = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let bytes = export_build_subly_archive(before.clone()).expect("archive");
        let imported = import_parse_subly_archive(bytes)
            .expect("parse")
            .expect("data");

        save_app_data_typed(&db, &imported, &crate::test_support::default_config()).expect("save imported");
        let (after, _) = load_app_data_typed(&db).expect("reload");
        crate::test_support::assert_core_invariants(&before, &after);
    }

    #[test]
    fn restart_smoke_json_import_then_reload_keeps_data() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let before = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let bytes = serde_json::to_vec(&before).expect("serialize");
        let imported: crate::models::AppDataDoc = serde_json::from_slice(&bytes).expect("deserialize");

        save_app_data_typed(&db, &imported, &crate::test_support::default_config()).expect("save imported");
        let (after, _) = load_app_data_typed(&db).expect("reload");
        crate::test_support::assert_core_invariants(&before, &after);
    }
}
