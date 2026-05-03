use serde_json::json;
use twitch_sankagata_manager_lib::auth::{DeviceFlow, StoredTokens};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn device_flow_polls_until_token_ready() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth2/device"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_code": "dc", "user_code": "ABCD", "verification_uri": "https://twitch.tv/activate",
            "expires_in": 1800, "interval": 0
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "A", "refresh_token": "R", "expires_in": 14400
        })))
        .mount(&server)
        .await;

    let flow = DeviceFlow::new(server.uri(), "CID", vec!["channel:read:redemptions".into()]);
    let start = flow.start().await.unwrap();
    assert_eq!(start.user_code, "ABCD");
    let tokens: StoredTokens = flow.poll(&start.device_code).await.unwrap();
    assert_eq!(tokens.access_token, "A");
}

#[tokio::test]
async fn refresh_token_obtains_new_access() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "NEW", "refresh_token": "NEW_R", "expires_in": 14400
        })))
        .mount(&server)
        .await;
    let flow = DeviceFlow::new(server.uri(), "CID", vec![]);
    let t = flow.refresh("OLD_R").await.unwrap();
    assert_eq!(t.access_token, "NEW");
}
