use core::fmt;
use reqwest::StatusCode;
use reqwest::blocking::{Client, multipart};
use std::env;
use std::io::Cursor;
use std::str::FromStr;
use thiserror::Error;

use crate::command::CommandReport;

const WEBHOOK_ID_VAR_NAME: &'static str = "WING_WEBHOOK_ID";
const WEBHOOK_TOKEN_VAR_NAME: &'static str = "WING_WEBHOOK_TOKEN";

fn construct_webhook_url(id: &u64, token: &Token) -> String {
    format!("https://discord.com/api/webhooks/{}/{}", id, token)
}

#[derive(Debug, Error)]
#[error("Invalid webhook id {0}. IDs must be 64 bit integers.")]
pub struct IdError(String);

#[derive(Debug, Error)]
#[error("Invalid webhook token {0}. Token must be alphanumeric or `_`/`-`.")]
pub struct TokenError(String);

#[derive(Debug, Error)]
pub enum WebhookValidationError {
    #[error("Encountered error during GET request to {url}: {error}")]
    RequestError { url: String, error: reqwest::Error },
    #[error("Webhook URL {url} is not valid: {status}")]
    InvalidWebhookUrl { url: String, status: StatusCode },
}

#[derive(Debug, Error)]
pub enum WebhookInfoError {
    #[error("Missing webhook id. Set `{WEBHOOK_ID_VAR_NAME}` to the id.")]
    MissingId,
    #[error("Missing webhook token. Set `{WEBHOOK_TOKEN_VAR_NAME}` to the token.")]
    MissingToken,
    #[error(transparent)]
    InvalidId(#[from] IdError),
    #[error(transparent)]
    InvalidToken(#[from] TokenError),
    #[error(transparent)]
    InvalidWebhook(#[from] WebhookValidationError),
}

pub struct Token(String);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Token {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s
            .chars()
            .all(|c| char::is_alphanumeric(c) || c == '_' || c == '-')
        {
            true => Ok(Self(s.to_owned())),
            false => Err(TokenError(s.to_owned())),
        }
    }
}

pub struct WebhookBuilder;

impl WebhookBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn id(self, s: &str) -> Result<WebhookWithoutToken, IdError> {
        Ok(WebhookWithoutToken {
            id: s.parse::<u64>().map_err(|_| IdError(s.to_owned()))?,
        })
    }
}

pub struct WebhookWithoutToken {
    id: u64,
}

impl WebhookWithoutToken {
    pub fn token(self, s: &str) -> Result<UnvalidatedWebhookInfo, TokenError> {
        Ok(UnvalidatedWebhookInfo {
            id: self.id,
            token: s.parse::<Token>()?,
        })
    }
}

pub struct UnvalidatedWebhookInfo {
    id: u64,
    token: Token,
}

impl UnvalidatedWebhookInfo {
    pub fn check(self) -> Result<WebhookInfo, WebhookValidationError> {
        let url = construct_webhook_url(&self.id, &self.token);
        let client = Client::new();
        let response =
            client
                .get(&url)
                .send()
                .map_err(|e| WebhookValidationError::RequestError {
                    url: url.clone(),
                    error: e,
                })?;

        if response.status().is_success() {
            Ok(WebhookInfo {
                id: self.id,
                token: self.token,
            })
        } else {
            Err(WebhookValidationError::InvalidWebhookUrl {
                url,
                status: response.status(),
            })
        }
    }
}

pub struct WebhookInfo {
    id: u64,
    token: Token,
}

impl WebhookInfo {
    pub fn from_env() -> Result<Self, WebhookInfoError> {
        let webhook_id = env::var("WING_WEBHOOK_ID").map_err(|_| WebhookInfoError::MissingId)?;
        let webhook_token =
            env::var("WING_WEBHOOK_TOKEN").map_err(|_| WebhookInfoError::MissingToken)?;

        Ok(WebhookBuilder::new()
            .id(&webhook_id)?
            .token(&webhook_token)?
            .check()?)
    }

    pub fn report(
        &self,
        command_report: CommandReport,
        stdout: &str,
        stderr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = construct_webhook_url(&self.id, &self.token);
        let client = Client::new();

        let form = multipart::Form::new().text("content", format!("{}", command_report));

        let form = if !stdout.is_empty() {
            let stdout_cursor = Cursor::new(stdout.to_owned());
            let stdout_part = multipart::Part::reader(stdout_cursor)
                .file_name("stdout.txt".to_owned())
                .mime_str("text/plain")?;
            form.part("file1", stdout_part)
        } else {
            form
        };

        let form = if !stderr.is_empty() {
            let stderr_cursor = Cursor::new(stderr.to_owned());
            let stderr_part = multipart::Part::reader(stderr_cursor)
                .file_name("stderr.txt".to_owned())
                .mime_str("text/plain")?;
            form.part("file2", stderr_part)
        } else {
            form
        };

        let response = client.post(&url).multipart(form).send()?;

        if !response.status().is_success() {
            return Err(format!("Failed to send file. Status: {}", response.status()).into());
        }

        Ok(())
    }
}
