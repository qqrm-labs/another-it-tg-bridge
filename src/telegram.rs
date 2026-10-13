use anyhow::{Context, Result};
use reqwest::Client;

pub struct TelegramBot {
    token: String,
    chat_id: String,
    client: Client,
    base_url: String,
}

impl TelegramBot {
    pub fn new(token: impl Into<String>, chat_id: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            chat_id: chat_id.into(),
            client: Client::new(),
            base_url: "https://api.telegram.org".into(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let token = std::env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN missing")?;
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").context("TELEGRAM_CHAT_ID missing")?;
        Ok(Self::new(token, chat_id))
    }

    pub async fn send_message(&self, text: &str) -> Result<()> {
        let url = format!("{}/bot{}/sendMessage", self.base_url, self.token);
        let resp = self
            .client
            .post(&url)
            .form(&[("chat_id", self.chat_id.as_str()), ("text", text)])
            .send()
            .await
            .map_err(|error| error.without_url())
            .context("failed to send request")?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("telegram returned HTTP {}", status);
        }
        #[derive(serde::Deserialize)]
        struct Acknowledgement {
            ok: bool,
        }
        let acknowledgement: Acknowledgement =
            resp.json().await.map_err(|error| error.without_url())?;
        anyhow::ensure!(acknowledgement.ok, "Telegram did not confirm delivery");
        Ok(())
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}
