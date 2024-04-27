use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serverless_discord_interactions_base::rest::{self, AuthPrefix, DiscordAPIError};

pub struct REST<'a> {
    api: &'a str,
    version: &'a str,
    client: reqwest::Client,
    token: Option<&'a str>,
    auth_prefix: AuthPrefix,
}

impl<'a> REST<'a> {
    pub fn new(
        api: &'a str,
        version: &'a str,
        token: Option<&'a str>,
        auth_prefix: AuthPrefix,
    ) -> Self {
        Self {
            api,
            version,
            client: reqwest::Client::new(),
            token,
            auth_prefix,
        }
    }
    fn resolve_request<T>(
        &self,
        method: rest::RequestMethod,
        route: rest::RouteLike,
        data: rest::RequestData<T>,
    ) -> anyhow::Result<reqwest::RequestBuilder>
    where
        T: Serialize,
    {
        let rest::RequestData {
            query,
            versioned,
            files,
            append_to_form_data,
            body,
            auth,
            auth_prefix,
            headers,
            reason,
        } = data;
        let url = if versioned {
            format!("{}/v{}{}", self.api, self.version, route)
        } else {
            format!("{}{}", self.api, route)
        };
        let url = reqwest::Url::parse_with_params(&url, query.into_iter())?;
        let method = match method {
            rest::RequestMethod::DELETE => reqwest::Method::DELETE,
            rest::RequestMethod::GET => reqwest::Method::GET,
            rest::RequestMethod::PATCH => reqwest::Method::PATCH,
            rest::RequestMethod::POST => reqwest::Method::POST,
            rest::RequestMethod::PUT => reqwest::Method::PUT,
        };
        let mut request = self.client.request(method, url);
        if auth {
            if let Some(token) = self.token {
                request = request.header(
                    "Authorization",
                    format!(
                        "{} {}",
                        auth_prefix.as_ref().unwrap_or(&self.auth_prefix),
                        token
                    ),
                );
            } else {
                return Err(anyhow::anyhow!(
                    "Expected token to be set for this request, but none was present"
                ));
            }
        }
        if files.is_empty() {
            if let Some(body) = body {
                let body = serde_json::to_vec(&body)?;
                request = request
                    .body(body)
                    .header("Content-Type", "application/json");
            }
        } else {
            let mut form = reqwest::multipart::Form::new();
            for (index, file) in files.into_iter().enumerate() {
                let part = reqwest::multipart::Part::bytes(file.data)
                    .file_name(file.name)
                    .mime_str(&file.content_type)?;

                form = form.part(file.key.unwrap_or_else(|| format!("files[{index}]")), part);
            }
            for (k, v) in append_to_form_data {
                form = form.text(k, v);
            }
            if let Some(body) = body {
                let body = serde_json::to_vec(&body)?;
                form = form.part("payload_json", reqwest::multipart::Part::bytes(body));
            }
            request = request.multipart(form);
        }
        if let Some(reason) = reason {
            let reason = utf8_percent_encode(&reason, NON_ALPHANUMERIC);
            request = request.header("X-Audit-Log-Reason", reason.to_string());
        }
        for (k, v) in headers {
            request = request.header(k, v);
        }
        Ok(request)
    }
    async fn request_inner<T>(
        &self,
        method: rest::RequestMethod,
        route: rest::RouteLike,
        data: rest::RequestData<T>,
    ) -> anyhow::Result<(reqwest::StatusCode, Vec<u8>)>
    where
        T: Serialize,
    {
        let request = self.resolve_request(method, route, data)?;
        let res = request.send().await?;
        Ok((res.status(), res.bytes().await?.to_vec()))
    }
}
impl<'a> rest::REST for REST<'a> {
    async fn request<T>(
        &mut self,
        method: rest::RequestMethod,
        route: rest::RouteLike,
        data: rest::RequestData<T>,
    ) -> Result<Vec<u8>, rest::RESTError>
    where
        T: Serialize + Send,
    {
        let (status, data) = self.request_inner(method, route, data).await?;
        if status.is_success() {
            return Ok(data);
        }
        Err(rest::RESTError::DiscordAPIError(DiscordAPIError {
            raw: data,
        }))
    }

    async fn request_json<T, R>(
        &mut self,
        method: rest::RequestMethod,
        route: rest::RouteLike,
        data: rest::RequestData<T>,
    ) -> Result<R, rest::RESTError>
    where
        T: Serialize + Send,
        R: for<'b> Deserialize<'b> + Send,
    {
        let res = self.request(method, route, data).await?;
        Ok(serde_json::from_slice(&res).map_err(|err| anyhow::Error::from(err))?)
    }
}
