use reqwest::Client;
use serde_json::json;
use crate::config::AppConfig;
use crate::db::models::EhGalleryMetadata;
use std::time::Duration;

const EH_API: &str = "https://api.e-hentai.org/api.php";
const EX_API: &str = "https://exhentai.org/api.php";

pub struct EhClient {
    http: Client,
    use_exhentai: bool,
}

impl EhClient {
    pub fn new(config: &AppConfig) -> Self {
        Self::new_from(config.use_exhentai)
    }

    pub fn new_from(use_exhentai: bool) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("failed to build http client");
        Self { http, use_exhentai }
    }

    pub fn api_url(&self) -> &str {
        if self.use_exhentai { EX_API } else { EH_API }
    }

    pub async fn fetch_gdata(
        &self,
        gidlist: &[(i64, String)],
        cookies: &str,
    ) -> Result<Vec<EhGalleryMetadata>, String> {
        let gid_json: Vec<Vec<serde_json::Value>> = gidlist.iter()
            .map(|(gid, token)| vec![json!(*gid), json!(token)])
            .collect();
        let body = json!({ "method": "gdata", "gidlist": gid_json, "namespace": 1 });

        let resp = self.http.post(self.api_url())
            .header("Cookie", cookies)
            .json(&body)
            .send().await.map_err(|e| e.to_string())?;

        if resp.status() == reqwest::StatusCode::FORBIDDEN {
            return Err("IP_BANNED".into());
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let gmetadata = data.get("gmetadata").ok_or("no gmetadata in response")?;
        serde_json::from_value(gmetadata.clone()).map_err(|e| e.to_string())
    }

    pub async fn search(
        &self,
        keyword: &str,
        page: i32,
        cookies: &str,
    ) -> Result<serde_json::Value, String> {
        let base = if self.use_exhentai { "https://exhentai.org" } else { "https://e-hentai.org" };
        let url = format!("{}/?f_search={}&page={}", base, keyword, page);
        let resp = self.http.get(&url).header("Cookie", cookies).send().await
            .map_err(|e| e.to_string())?;
        if resp.status() == reqwest::StatusCode::FORBIDDEN {
            return Err("IP_BANNED".into());
        }
        let text = resp.text().await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({ "html": text }))
    }

    pub fn build_cookie_string(config: &AppConfig) -> String {
        format!(
            "ipb_member_id={}; ipb_pass_hash={}; igneous={}",
            config.eh_cookies.ipb_member_id,
            config.eh_cookies.ipb_pass_hash,
            config.eh_cookies.igneous
        )
    }
}