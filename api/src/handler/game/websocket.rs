use crate::{
    body::MessageOut,
    error::Result,
    handler::game::{field::list_fields, player::ger_players},
    server::{AppState, Identity},
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path, TypedHeader,
    },
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

pub async fn ws(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    identity: Identity,
    Path(game_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, &state, identity.user_id, game_id).await
    })
}

async fn handle_socket(socket: WebSocket, state: &AppState, user_id: Uuid, game_id: Uuid) {
    if let Err(err) = send_game_update_messages(socket, state, user_id, game_id).await {
        // if let Err(err) = socket.close().await {
        //     tracing::error!("closing socket failes: {:?}", err);
        // }
        tracing::error!("sending game update messgages failes: {:?}", err);
    }
}

pub async fn send_game_update_messages(
    mut socket: WebSocket,
    state: &AppState,
    user_id: Uuid,
    game_id: Uuid,
) -> Result<Vec<String>> {
    let pool = &state.pool;
    let mut receiver = state.receiver.clone();
    let mut socket_healthy = true;

    // check if game exists

    let game = sqlx::query!(
        r#"
            select 
                g.id,
                g.game_template_id
            from 
                bingo.games g
            inner join
                bingo.players p on p.game_id = g.id
            where 
                p.user_id = $1 and g.id = $2 and closed = false
        "#,
        user_id,
        game_id,
    )
    .fetch_one(pool)
    .await?;

    // while socket is healthy: listen for changes in tokio watch and send respective updates to client

    let mut latest_game_update_at = Utc::now() - Duration::days(1);

    while socket_healthy {
        if receiver.changed().await.is_ok() {
            let game_updated_recently = receiver
                .borrow()
                .get(&game_id)
                .map(|v| v > &latest_game_update_at)
                .unwrap_or(false);

            if game_updated_recently {
                latest_game_update_at = Utc::now();
                let mut messages = Vec::new();

                if game_id == game.id {
                    let mut conn = pool.acquire().await?;

                    let fields = list_fields(game_id, user_id, &mut conn).await?;
                    messages.push(serde_json::to_string(&MessageOut::Fields(fields))?);

                    let players = ger_players(game_id, user_id, &mut conn).await?;
                    messages.push(serde_json::to_string(&MessageOut::Players(players))?);

                    for message in messages {
                        if let Err(err) = socket.send(Message::Text(message)).await {
                            tracing::warn!("Failed to send message: {:?}", err);
                            socket_healthy = false;
                        }
                    }
                }
            }
        } else {
            tracing::error!(
                "channel connecting postgres notifications listener to websocket is broken"
            );
            break;
        }
    }

    Ok(vec![])
}
