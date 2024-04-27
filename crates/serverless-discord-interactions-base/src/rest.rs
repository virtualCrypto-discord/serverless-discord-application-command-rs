use core::fmt;
use std::{collections::HashMap, future::Future};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub enum RequestMethod {
    DELETE,
    GET,
    PATCH,
    POST,
    PUT,
}
#[derive(Debug, Clone, Copy)]
pub enum AuthPrefix {
    Bot,
    Bearer,
}
impl fmt::Display for AuthPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthPrefix::Bearer => write!(f, "Bearer"),
            AuthPrefix::Bot => write!(f, "Bot"),
        }
    }
}

pub type RouteLike = String;

pub struct FileData {
    pub content_type: String,
    pub data: Vec<u8>,
    pub key: Option<String>,
    pub name: String,
}

// https://github.com/discordjs/discord.js/blob/main/packages/rest/src/lib/utils/types.ts#L269
pub struct RequestData<T>
where
    T: serde::Serialize,
{
    pub append_to_form_data: HashMap<String, String>,
    pub auth_prefix: Option<AuthPrefix>,
    pub auth: bool,
    pub body: Option<T>,
    pub files: Vec<FileData>,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub reason: Option<String>,
    pub versioned: bool,
}

impl<T: serde::Serialize> Default for RequestData<T> {
    fn default() -> Self {
        Self {
            auth: true,
            versioned: true,
            append_to_form_data: Default::default(),
            auth_prefix: None,
            body: None,
            files: Default::default(),
            headers: Default::default(),
            query: Default::default(),
            reason: None,
        }
    }
}

#[derive(Debug)]
pub struct DiscordAPIError {
    pub raw: Vec<u8>,
}
#[derive(Error, Debug)]
pub enum RESTError {
    #[error("discord api error")]
    DiscordAPIError(DiscordAPIError),
    #[error("request error")]
    RequestError(#[from] anyhow::Error),
}

pub trait REST {
    fn request<T>(
        &mut self,
        method: RequestMethod,
        route: RouteLike,
        data: RequestData<T>,
    ) -> impl Future<Output = Result<Vec<u8>, RESTError>>
    where
        T: Serialize + Send;
    fn request_json<T, R>(
        &mut self,
        method: RequestMethod,
        route: RouteLike,
        data: RequestData<T>,
    ) -> impl Future<Output = Result<R, RESTError>>
    where
        T: Serialize + Send,
        R: for<'a> Deserialize<'a> + Send;
}
