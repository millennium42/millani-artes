fn main() {
    let attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "create_account",
            "create_category",
            "list_accounts",
            "create_entry",
            "create_exit",
            "create_transfer",
            "create_product",
            "security_probe_report",
        ]));
    tauri_build::try_build(attributes).expect("TAURI_BUILD_FAILED");
}
