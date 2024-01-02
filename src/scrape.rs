use reqwest::{Client, StatusCode, Url};
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
pub struct AghApiStatistics {
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

pub fn start_scrape_loop(
    agh_base_url: Url,
    user: Option<String>,
    pwd: Option<String>,
    scrape_interval: Duration,
) -> Result<Receiver<AghApiStatistics>, String> {
    let client = Client::new();
    let agh_stats_url = agh_base_url.join("stats").map_err(|e| e.to_string())?;
    let mut interval = tokio::time::interval(scrape_interval);

    let (tx, rx) = channel(Default::default());

    tokio::spawn(async move {
        log::info!(
            "Scraping AGH API every {} seconds",
            scrape_interval.as_secs()
        );
        loop {
            match scrape_agh_stats(&client, &agh_stats_url, &user, &pwd).await {
                Ok(stats) => tx.send(stats).unwrap(),
                Err(s) => {
                    log::error!("Failed to fetch stats from AGH API: {s}");
                    return;
                }
            }

            interval.tick().await;
        }
    });

    Ok(rx)
}

async fn scrape_agh_stats(
    client: &Client,
    agh_stats_url: &Url,
    user: &Option<String>,
    pwd: &Option<String>,
) -> Result<AghApiStatistics, String> {
    let mut req_builder = client.get(agh_stats_url.clone());
    if let Some(username) = user {
        req_builder = req_builder.basic_auth(username, pwd.clone())
    }

    log::info!("Querying AGH API at {agh_stats_url}.");
    match req_builder.send().await {
        Ok(resp) => match resp.status() {
            StatusCode::OK => {
                let body = resp.bytes().await.unwrap();
                let body_len = body.len();
                log::info!("Got {body_len} bytes from AGH API.");

                Ok(serde_json::from_slice(body.iter().as_slice()).unwrap())
            }
            _ => Err(format!(
                "Received non-200 status code when scraping AGH: {} {}\n",
                resp.status().as_u16(),
                resp.status().canonical_reason().unwrap_or("")
            )),
        },
        Err(e) => Err(format!("Could not fetch AGH statistics: {}", e)),
    }
}
