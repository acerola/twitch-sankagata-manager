use serde_json::json;
use twitch_sankagata_manager_lib::helix::HelixClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn list_rewards_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/helix/channel_points/custom_rewards"))
        .and(header("Authorization", "Bearer TEST_TOKEN"))
        .and(header("Client-Id", "TEST_CLIENT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "id": "r1", "title": "参加する", "cost": 500 },
                { "id": "r2", "title": "other",    "cost": 100 }
            ]
        })))
        .mount(&server)
        .await;

    let c = HelixClient::new(server.uri(), "TEST_CLIENT", "TEST_TOKEN");
    let rewards = c.list_rewards("12345").await.unwrap();
    assert_eq!(rewards.len(), 2);
    assert_eq!(rewards[0].title, "参加する");
}

#[tokio::test]
async fn refund_redemption_sends_patch() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/helix/channel_points/custom_rewards/redemptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [] })))
        .mount(&server)
        .await;

    let c = HelixClient::new(server.uri(), "CID", "TOK");
    c.refund_redemption("42", "reward1", "redeem1")
        .await
        .unwrap();
}
