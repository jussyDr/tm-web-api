use reqwest::{
    header::{AUTHORIZATION, USER_AGENT},
    Error,
};
use serde::{Deserialize, Serialize};

// TODO: logging support

const USER_AGENT_VALUE: &str = "JussyDr";

pub struct DedicatedServerClient {
    http_client: reqwest::Client,
    login: &'static str,
    password: &'static str,
    auth_token: Option<AuthToken>,
}

impl DedicatedServerClient {
    pub fn new(login: &'static str, password: &'static str) -> Self {
        Self {
            http_client: reqwest::Client::new(),
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
        config: &ServerConfig,
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
#[serde(rename_all = "camelCase")]
struct AuthToken {
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub title_id: &'static str,
    pub script_file_name: &'static str,
    pub port: u16,
    pub player_count_max: u8,
    pub player_count: u8,
    pub server_name: &'static str,
    pub is_private: bool,
    pub ip: &'static str,
    pub game_mode_custom_data: &'static str,
    pub game_mode: &'static str,
}
