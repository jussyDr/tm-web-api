use std::ops::Deref;

use reqwest::{
    header::{AUTHORIZATION, USER_AGENT},
    Error,
};
use serde::{Deserialize, Serialize};

// TODO: logging support

const USER_AGENT_VALUE: &str = "JussyDr";

pub struct Client {
    http_client: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn client_config(&self) -> Result<ClientConfig, Error> {
        self.http_client
            .get("https://prod.trackmania.core.nadeo.online/client/config")
            .send()
            .await?
            .json()
            .await
    }
}

pub struct DedicatedServerClient {
    client: Client,
    login: &'static str,
    password: &'static str,
    auth_token: Option<AuthToken>,
}

impl Deref for DedicatedServerClient {
    type Target = Client;

    fn deref(&self) -> &Client {
        &self.client
    }
}

impl DedicatedServerClient {
    pub fn new(login: &'static str, password: &'static str) -> Self {
        Self {
            client: Client::new(),
            login,
            password,
            auth_token: None,
        }
    }

    async fn get_access_token(&mut self) -> Result<&str, Error> {
        match self.auth_token {
            None => {
                let auth_token: AuthToken = self
                    .http_client
                    .post("https://prod.trackmania.core.nadeo.online/v2/authentication/token/basic")
                    .header(USER_AGENT, USER_AGENT_VALUE)
                    .basic_auth(self.login, Some(self.password))
                    .send()
                    .await?
                    .json()
                    .await?;

                self.auth_token = Some(auth_token);

                Ok(&self.auth_token.as_ref().unwrap().access_token)
            }
            Some(ref auth_token) => Ok(&auth_token.access_token),
        }
    }

    pub async fn register_dedicated_server(
        &mut self,
        account_id: &str,
        config: &ServerConfig<'_>,
    ) -> Result<(), Error> {
        let access_token = self.get_access_token().await?;

        let url = format!("https://prod.trackmania.core.nadeo.online/servers/{account_id}");

        let authorization = format!("nadeo_v1 t={access_token}");

        self.http_client
            .put(url)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(AUTHORIZATION, authorization)
            .json(config)
            .send()
            .await?;

        Ok(())
    }

    pub async fn deregister_dedicated_server(&mut self, account_id: &str) -> Result<(), Error> {
        let access_token = self.get_access_token().await?;

        let url = format!("https://prod.trackmania.core.nadeo.online/servers/{account_id}");

        let authorization = format!("nadeo_v1 t={access_token}");

        self.http_client
            .delete(url)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(AUTHORIZATION, authorization)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ClientConfig {
    #[serde(rename = "ClientIP")]
    pub client_ip: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthToken {
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig<'a> {
    pub title_id: &'a str,
    pub script_file_name: &'a str,
    pub port: u16,
    pub player_count_max: u8,
    pub player_count: u8,
    pub server_name: &'a str,
    pub is_private: bool,
    pub ip: &'a str,
    pub game_mode_custom_data: &'a str,
    pub game_mode: &'a str,
}
