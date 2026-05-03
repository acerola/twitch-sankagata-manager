fn main() {
    // Load .env from repo root if present. Shell-exported vars win.
    let _ = dotenvy::from_path("../.env");

    for key in [
        "TWITCH_CLIENT_ID",
        "TWITCH_API_BASE",
        "TWITCH_ID_BASE",
        "TWITCH_EVENTSUB_URL",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    println!("cargo:rerun-if-changed=../.env");

    // Resolve overrides, fall back to production endpoints.
    let resolve = |key: &str, default: &str| -> String {
        std::env::var(key)
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| default.to_string())
    };

    let client_id = resolve("TWITCH_CLIENT_ID", "DEV_PLACEHOLDER_CLIENT_ID");
    let api_base = resolve("TWITCH_API_BASE", "https://api.twitch.tv");
    let id_base = resolve("TWITCH_ID_BASE", "https://id.twitch.tv");
    let eventsub_url = resolve("TWITCH_EVENTSUB_URL", "wss://eventsub.wss.twitch.tv/ws");

    println!("cargo:rustc-env=TWITCH_CLIENT_ID={client_id}");
    println!("cargo:rustc-env=TWITCH_API_BASE={api_base}");
    println!("cargo:rustc-env=TWITCH_ID_BASE={id_base}");
    println!("cargo:rustc-env=TWITCH_EVENTSUB_URL={eventsub_url}");

    tauri_build::build()
}
