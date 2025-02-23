const HEADER_SIGNATURE: &str = "X-Signature-Ed25519";
const HEADER_TIMESTAMP: &str = "X-Signature-Timestamp";

use crate::event::implement::webhook::WebHookServiceApp;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use http::StatusCode;

use ed25519_dalek::{SIGNATURE_LENGTH, Signature};

pub async fn signature_check(
    State(app): State<WebHookServiceApp>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let signature = {
        let signature = headers
            .get(HEADER_SIGNATURE)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let signature: [u8; SIGNATURE_LENGTH] = hex::decode(signature.as_bytes())
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .try_into()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        Signature::from_bytes(&signature)
    };
    let timestamp = headers
        .get(HEADER_TIMESTAMP)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, app.config.body_size_limit as usize)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let mut message = Vec::with_capacity(timestamp.len() + body.len());
    message.extend_from_slice(timestamp.as_bytes());
    message.extend_from_slice(&body);
    if crate::utils::verify_signature(&app.bot_secret, &message, &signature) {
        Ok(next.run(Request::from_parts(parts, body.into())).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
