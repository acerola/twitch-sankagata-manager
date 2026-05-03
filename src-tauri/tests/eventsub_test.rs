use futures_util::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use twitch_sankagata_manager_lib::eventsub::{EventSubClient, EventSubMessage};

#[tokio::test]
async fn receives_session_welcome_and_notification() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        ws.send(tokio_tungstenite::tungstenite::Message::Text(r#"{"metadata":{"message_type":"session_welcome"},"payload":{"session":{"id":"S1","status":"connected","keepalive_timeout_seconds":10}}}"#.into())).await.unwrap();
        ws.send(tokio_tungstenite::tungstenite::Message::Text(r#"{"metadata":{"message_type":"notification","subscription_type":"channel.channel_points_custom_reward_redemption.add"},"payload":{"event":{"user_id":"u1","user_name":"alice","user_login":"alice","reward":{"id":"r1","title":"参加する"},"id":"rd1","status":"UNFULFILLED"}}}"#.into())).await.unwrap();
        // keep open a tick so client can drain
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    });
    let url = format!("ws://{addr}");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let client = EventSubClient::new(url, tx);
    tokio::spawn(async move {
        client.run().await.ok();
    });

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(msg, EventSubMessage::Welcome { .. }));
    let msg2 = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(msg2, EventSubMessage::Notification { .. }));

    server.await.ok();
}

#[tokio::test]
async fn follows_session_reconnect_url() {
    let next_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let next_addr = next_listener.local_addr().unwrap();
    let next_url = format!("ws://{next_addr}");
    let next_server = tokio::spawn(async move {
        let (stream, _) = next_listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        ws.send(tokio_tungstenite::tungstenite::Message::Text(r#"{"metadata":{"message_type":"session_welcome"},"payload":{"session":{"id":"S2","status":"connected","keepalive_timeout_seconds":10}}}"#.into())).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    });

    let first_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let first_addr = first_listener.local_addr().unwrap();
    let first_server = tokio::spawn(async move {
        let (stream, _) = first_listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let msg = format!(
            r#"{{"metadata":{{"message_type":"session_reconnect"}},"payload":{{"session":{{"reconnect_url":"{next_url}"}}}}}}"#
        );
        ws.send(tokio_tungstenite::tungstenite::Message::Text(msg))
            .await
            .unwrap();
    });

    let url = format!("ws://{first_addr}");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let client = EventSubClient::new(url, tx);
    tokio::spawn(async move {
        client.run().await.ok();
    });

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(msg, EventSubMessage::Reconnect { .. }));
    let msg2 = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(msg2, EventSubMessage::Welcome { session_id, .. } if session_id == "S2"));

    first_server.await.ok();
    next_server.await.ok();
}
