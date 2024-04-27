use interaction::InteractionHandler;

pub mod interaction;
pub mod rest;
pub use twilight_model;
pub trait Service {
    fn register_handler(handler: impl InteractionHandler);
}
