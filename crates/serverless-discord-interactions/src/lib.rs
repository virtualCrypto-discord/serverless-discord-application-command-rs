pub use serverless_discord_interactions_base::rest;

use serverless_discord_interactions_base::rest::AuthPrefix;
pub(crate) use serverless_discord_interactions_cloudflare_backend as backend;
const DEFAULT_API: &str = "https://discord.com/api";
const DEFAULT_VERSION: &str = "10";

pub struct RESTConfig<'a> {
    pub api: &'a str,
    pub version: &'a str,
    pub token: Option<&'a str>,
    pub auth_prefix: AuthPrefix,
}
impl<'a> RESTConfig<'a> {
    pub fn new(token: &'a str) -> Self {
        Self {
            api: DEFAULT_API,
            version: DEFAULT_VERSION,
            token: Some(token),
            auth_prefix: AuthPrefix::Bot,
        }
    }
}
pub fn rest_client<'a>(config: RESTConfig<'a>) -> impl rest::REST + 'a {
    let RESTConfig {
        api,
        version,
        token,
        auth_prefix,
    } = config;
    backend::REST::new(api, version, token, auth_prefix)
}
