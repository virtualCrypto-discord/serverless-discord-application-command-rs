pub use serverless_discord_interactions_base::rest;

use serverless_discord_interactions_base::rest::AuthPrefix;
pub(crate) use serverless_discord_interactions_cloudflare_backend as backend;

pub fn rest_client<'a>(
    api: &'a str,
    version: &'a str,
    token: Option<&'a str>,
    auth_prefix: AuthPrefix,
) -> impl rest::REST + 'a {
    backend::REST::new(api, version, token, auth_prefix)
}
