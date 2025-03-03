use std::net::SocketAddr;

use axum::{
    body::Bytes,
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::stream::StreamExt;
use serde::Serialize;
use tracing::{debug, error, info, trace};

pub async fn root() -> impl IntoResponse {
    #[derive(Serialize)]
    struct Status {
        status: String,
    }
    (
        StatusCode::OK,
        Json(Status {
            status: "ok".to_string(),
        }),
    )
}

#[debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        info!("pinged {who}...");
    } else {
        error!("could not send ping {who}!");
        return;
    }

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (_, mut receiver) = socket.split();

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            trace!("{:?}", msg.to_text());
            cnt += 1;
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => debug!("received {b} messages"),
                Err(b) => error!("Error receiving messages {b:?}")
            }
        }
    }

    // returning from the handler closes the websocket connection
    info!("websocket context {who} destroyed");
}
