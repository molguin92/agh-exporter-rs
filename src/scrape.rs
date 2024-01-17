use reqwest::{Client, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::watch::{channel, Receiver};
use tokio::time::Duration;

#[derive(Debug, Deserialize, Default, Clone)]
struct NestedDomainCount(u64);

impl Serialize for NestedDomainCount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("!<|domain==<!", &self.0)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
struct NestedClientCount(u64);

impl Serialize for NestedClientCount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("!<|client==<!", &self.0)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
struct NestedUpstreamCount(u64);

impl Serialize for NestedUpstreamCount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("!<|upstream==<!", &self.0)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
struct NestedUpstreamFloat(f64);

impl Serialize for NestedUpstreamFloat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("!<|upstream==<!", &self.0)
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct Stats {
    num_dns_queries: u64,
    num_blocked_filtering: u64,
    num_replaced_safebrowsing: u64,
    num_replaced_safesearch: u64,
    num_replaced_parental: u64,
    avg_processing_time: f64,
    top_clients: Vec<HashMap<String, NestedClientCount>>,
    top_upstreams_responses: Vec<HashMap<String, NestedUpstreamCount>>,
    top_upstreams_avg_time: Vec<HashMap<String, NestedUpstreamFloat>>,
    top_queried_domains: Vec<HashMap<String, NestedDomainCount>>,
    top_blocked_domains: Vec<HashMap<String, NestedDomainCount>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct Status {
    protection_enabled: bool,
    dhcp_available: bool,
    running: bool,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Metrics {
    #[serde(flatten)]
    status: Status,
    #[serde(flatten)]
    stats: Stats,
}

pub fn start_scrape_loop(
    agh_base_url: Url,
    user: Option<String>,
    pwd: Option<String>,
    scrape_interval: Duration,
) -> Result<Receiver<Metrics>, String> {
    let (tx, rx) = channel(Default::default());

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(scrape_interval);
        let mut agh_api_client = AghApiClient::new(user, pwd, agh_base_url);

        log::info!(
            "Scraping AGH API every {} seconds",
            scrape_interval.as_secs()
        );
        loop {
            match agh_api_client.get_all().await {
                Ok(stats) => {
                    if tx.send(stats).is_err() {
                        // this should never happen
                        unreachable!()
                    }
                }
                Err(s) => {
                    log::error!("Failed to fetch stats from AGH API: {s}");
                    // recreate the client just in case, to clear any existing connections and prevent further errors
                    agh_api_client.reset()
                }
            }

            interval.tick().await;
        }
    });

    Ok(rx)
}

struct AghApiClient {
    client: Client,
    user: Option<String>,
    pwd: Option<String>,
    api_url: Url,
}

impl AghApiClient {
    fn new(user: Option<String>, pwd: Option<String>, api_url: Url) -> Self {
        Self {
            client: Client::new(),
            user,
            pwd,
            api_url,
        }
    }

    /// Resets the inner client to clear any long-lasting connections
    fn reset(&mut self) {
        self.client = Client::new();
    }

    async fn get<R>(&self, sub_url: impl AsRef<str>) -> Result<R, String>
    where
        R: DeserializeOwned,
    {
        let url = self
            .api_url
            .join(sub_url.as_ref())
            .map_err(|e| e.to_string())?;
        let mut req_builder = self.client.get(url.clone());
        if let Some(ref username) = self.user {
            req_builder = req_builder.basic_auth(username.clone(), self.pwd.clone());
        }

        log::info!("Querying {url}...");
        match req_builder.send().await {
            Ok(resp) => match resp.status() {
                StatusCode::OK => Ok(resp
                    .json()
                    .await
                    .map_err(|e| format!("Could not parse response body: {e}"))?),
                _ => Err(format!(
                    "Received non-200 status code when querying {url}: {} {}\n",
                    resp.status().as_u16(),
                    resp.status().canonical_reason().unwrap_or("")
                )),
            },
            Err(e) => Err(format!("Could not query {url}: {e}")),
        }
    }

    async fn get_all(&self) -> Result<Metrics, String> {
        // TODO: parallel calls?
        Ok(Metrics {
            stats: self.get("stats").await?,
            status: self.get("status").await?,
        })
    }
}
