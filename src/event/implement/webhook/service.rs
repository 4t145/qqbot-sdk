use axum::{Json, extract::State};
use http::StatusCode;
use serde_json::Value as JsonValue;

use crate::event::{implement::webhook::WebHookServiceApp, model::*};
pub async fn event_listen_service(
    State(app): State<WebHookServiceApp>,
    Json(payload): Json<GeneralPayload>,
) -> Result<Json<JsonValue>, StatusCode> {
    let ack = payload.new_http_callback_ack();
    let inbound = payload.into_inbound().map_err(|e| {
        tracing::debug!(%e, "failed to convert payload to inbound payload");
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    })?;

    match inbound {
        InboundPayloadKind::Dispatch(event) => {
            tracing::info!(?event, "event inbound");
            let result = app.event_tx.send(event).await;
            match result {
                Ok(_) => Ok(Json(
                    serde_json::to_value(ack).expect("ack should be a valid json"),
                )),
                Err(e) => {
                    tracing::error!(%e, "failed to dispatch event");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        InboundPayloadKind::HttpCallbackValidation(request) => serde_json::to_value(
            request.valid(&app.bot_secret),
        )
        .map_err(|e| {
            tracing::error!(%e, "failed to convert http callback validation response to json");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(Json),
    }
}
