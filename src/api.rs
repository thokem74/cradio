use std::sync::Arc;

use serde::Deserialize;
use tokio::{sync::Semaphore, task::JoinSet};

#[derive(Debug, Clone, Deserialize)]
pub struct Station {
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub url_resolved: String,
    #[serde(default)]
    pub tags: String,
    #[serde(rename = "countrycode", default)]
    pub country_code: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub bitrate: u32,
}

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub name: String,
    pub tags: String,
    pub country: String,
    pub language: String,
    pub bitrate: Option<u32>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            name: String::new(),
            tags: String::new(),
            country: String::new(),
            language: String::new(),
            bitrate: None,
            limit: 50,
            offset: 0,
        }
    }
}

fn resolve_api_server() -> String {
    let fallback = "all.api.radio-browser.info".to_string();
    match dns_lookup::lookup_host("all.api.radio-browser.info") {
        Ok(addrs) if !addrs.is_empty() => fallback,
        _ => fallback,
    }
}

pub async fn search_stations(
    client: &reqwest::Client,
    params: &SearchParams,
) -> Result<Vec<Station>, String> {
    let server = resolve_api_server();
    let url = format!("https://{}/json/stations/search", server);

    let mut query: Vec<(&str, String)> = vec![
        ("limit", params.limit.to_string()),
        ("offset", params.offset.to_string()),
        ("hidebroken", "true".to_string()),
        ("order", "clickcount".to_string()),
        ("reverse", "true".to_string()),
    ];

    if !params.name.is_empty() {
        query.push(("name", params.name.clone()));
    }
    if !params.tags.is_empty() {
        query.push(("tagList", params.tags.clone()));
    }
    if !params.country.is_empty() {
        query.push(("countrycode", params.country.to_uppercase()));
    }
    if !params.language.is_empty() {
        query.push(("language", params.language.to_lowercase()));
    }
    if let Some(bitrate) = params.bitrate {
        query.push(("bitrateMin", bitrate.to_string()));
    }

    let response = client
        .get(&url)
        .header("User-Agent", "cradio/0.1")
        .query(&query)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let mut stations: Vec<Station> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    if let Some(bitrate) = params.bitrate {
        stations.retain(|station| station.bitrate >= bitrate);
    }

    Ok(stations)
}

async fn fetch_station_by_uuid(
    client: &reqwest::Client,
    server: &str,
    station_uuid: &str,
) -> Result<Option<Station>, String> {
    let url = format!("https://{}/json/stations/byuuid/{}", server, station_uuid);
    let response = client
        .get(&url)
        .header("User-Agent", "cradio/0.1")
        .send()
        .await
        .map_err(|e| format!("Request failed for {}: {}", station_uuid, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "API error for {}: {}",
            station_uuid,
            response.status()
        ));
    }

    let stations: Vec<Station> = response
        .json()
        .await
        .map_err(|e| format!("Parse error for {}: {}", station_uuid, e))?;

    Ok(stations.into_iter().next())
}

pub async fn fetch_stations_by_uuids(
    client: &reqwest::Client,
    station_uuids: Vec<String>,
) -> (Vec<Station>, Vec<String>) {
    if station_uuids.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let server = resolve_api_server();
    let semaphore = Arc::new(Semaphore::new(8));
    let mut join_set = JoinSet::new();

    for station_uuid in station_uuids {
        let client = client.clone();
        let server = server.clone();
        let semaphore = Arc::clone(&semaphore);
        join_set.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .map_err(|e| format!("Concurrency control error: {}", e))?;
            let result = fetch_station_by_uuid(&client, &server, &station_uuid).await;
            Ok::<(String, Result<Option<Station>, String>), String>((station_uuid, result))
        });
    }

    let mut stations = Vec::new();
    let mut failed_uuids = Vec::new();

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok((station_uuid, Ok(Some(station))))) => {
                stations.push(station);
                let _ = station_uuid;
            }
            Ok(Ok((station_uuid, Ok(None)))) => failed_uuids.push(station_uuid),
            Ok(Ok((station_uuid, Err(_)))) => failed_uuids.push(station_uuid),
            Ok(Err(_)) => {}
            Err(_) => {}
        }
    }

    (stations, failed_uuids)
}

pub async fn fetch_station_stream_urls(
    client: &reqwest::Client,
    station_uuid: &str,
) -> Result<Vec<String>, String> {
    let server = resolve_api_server();
    let m3u_url = format!("https://{}/m3u/url/{}", server, station_uuid);
    let m3u_text = client
        .get(&m3u_url)
        .header("User-Agent", "cradio/0.1")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch station playlist: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Failed to fetch station playlist: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read station playlist: {}", e))?;

    let mut urls = parse_m3u_urls(&m3u_text);
    if urls.is_empty() {
        let pls_url = format!("https://{}/pls/url/{}", server, station_uuid);
        let pls_text = client
            .get(&pls_url)
            .header("User-Agent", "cradio/0.1")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch station playlist: {}", e))?
            .error_for_status()
            .map_err(|e| format!("Failed to fetch station playlist: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read station playlist: {}", e))?;
        urls = parse_pls_urls(&pls_text);
    }

    Ok(urls)
}

fn parse_m3u_urls(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_pls_urls(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("File"))
        .filter_map(|line| line.split_once('='))
        .map(|(_, url)| url.trim().to_string())
        .filter(|url| !url.is_empty())
        .collect()
}
