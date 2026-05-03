use serde_json::Value;

#[test]
fn default_capability_allows_overlay_window() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();

    let windows = capability["windows"]
        .as_array()
        .expect("capability windows must be an array");

    assert!(
        windows.iter().any(|w| w == "overlay"),
        "overlay window must be listed so it can use Tauri IPC/events"
    );

    assert!(
        !windows.iter().any(|w| w == "trash"),
        "trash is rendered in the main window and must not be registered as a separate window"
    );
}

#[test]
fn default_capability_allows_overlay_window_dragging() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();

    let permissions = capability["permissions"]
        .as_array()
        .expect("capability permissions must be an array");

    assert!(
        permissions
            .iter()
            .any(|p| p == "core:window:allow-start-dragging"),
        "overlay drag grip requires core:window:allow-start-dragging"
    );
}

#[test]
fn default_capability_allows_overlay_visibility_toggle() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();

    let permissions = capability["permissions"]
        .as_array()
        .expect("capability permissions must be an array");

    for permission in [
        "core:window:allow-hide",
        "core:window:allow-show",
        "core:window:allow-is-visible",
    ] {
        assert!(
            permissions.iter().any(|p| p == permission),
            "overlay visibility toggle requires {permission}"
        );
    }
}
