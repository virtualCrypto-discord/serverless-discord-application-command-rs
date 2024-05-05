use std::future::Future;

use twilight_model::application::interaction::Interaction;
use twilight_model::http::interaction::InteractionResponse;
pub trait InteractionHandler {
    fn on_interaction(
        self,
        interaction: Interaction,
    ) -> impl Future<Output = anyhow::Result<InteractionResponse>>;
}
