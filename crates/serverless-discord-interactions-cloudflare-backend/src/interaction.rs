use anyhow::Ok;
use base::{interaction::InteractionHandler, twilight_model};
use ed25519_dalek::{Signature, VerifyingKey};
use serverless_discord_interactions_base as base;
use twilight_model::{
    application::interaction::{Interaction, InteractionType},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use worker::{Request, Response};

pub struct RequestProcessor<T: InteractionHandler> {
    vk: VerifyingKey,
    handler: T,
}

impl<T: InteractionHandler> RequestProcessor<T> {
    pub fn new(vk: VerifyingKey, handler: T) -> Self {
        Self { vk, handler }
    }
    pub async fn process_request(&mut self, mut req: Request) -> anyhow::Result<Response> {
        let sign = req
            .headers()
            .get("X-Signature-Ed25519")?
            .ok_or_else(|| anyhow::anyhow!("missing request header: X-Signature-Ed25519"))?;
        let timestamp = req
            .headers()
            .get("X-Signature-Timestamp")?
            .ok_or_else(|| anyhow::anyhow!("missing request header: X-Signature-Timestamp"))?;
        let body = req.text().await?;
        match verify(&self.vk, &body, &timestamp, &sign) {
            anyhow::Result::Ok(_) => {
                let interaction: Interaction = serde_json::from_str(&body)?;
                let res = if interaction.kind == InteractionType::Ping {
                    InteractionResponse {
                        data: None,
                        kind: InteractionResponseType::Pong,
                    }
                } else {
                    self.handler.on_interaction(interaction).await?
                };
                let res = Response::from_json(&res)?;
                Ok(res)
            }
            Err(_) => Ok(Response::error("invalid request signature", 401)?),
        }
    }
}
fn verify(vk: &VerifyingKey, body: &str, timestamp: &str, sign: &str) -> anyhow::Result<()> {
    let mut sign_array: [u8; 64] = [0; 64];
    hex::decode_to_slice(sign, &mut sign_array)?;

    vk.verify_strict(
        format!("{}{}", timestamp, body).as_bytes(),
        &Signature::from_bytes(&sign_array),
    )?;
    Ok(())
}
