use std::sync::Arc;

use serde::Deserialize;
use tokio::{sync::Semaphore, task::JoinSet};

const API_SERVER: &str = "all.api.radio-browser.info";

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

fn search_query(params: &SearchParams) -> Vec<(&'static str, String)> {
    let mut query = vec![
        ("limit", params.limit.to_string()),
        ("offset", params.offset.to_string()),
        ("hidebroken", "true".to_string()),
        ("order", "clickcount".to_string()),
        ("reverse", "true".to_string()),
    ];

    let name = params.name.trim();
    if !name.is_empty() {
        query.push(("name", name.to_string()));
    }

    let tags = params.tags.trim();
    if !tags.is_empty() {
        query.push(("tagList", tags.to_string()));
    }

    let country = params.country.trim();
    if !country.is_empty() {
        query.push(("countrycode", country.to_uppercase()));
    }

    let language = params.language.trim();
    if !language.is_empty() {
        query.push(("language", language.to_lowercase()));
    }

    if let Some(bitrate) = params.bitrate {
        query.push(("bitrateMin", bitrate.to_string()));
    }

    query
}

fn filter_stations_by_bitrate(mut stations: Vec<Station>, bitrate: Option<u32>) -> Vec<Station> {
    if let Some(bitrate) = bitrate {
        stations.retain(|station| station.bitrate >= bitrate);
    }

    stations
}

pub async fn search_stations(
    client: &reqwest::Client,
    params: &SearchParams,
) -> Result<Vec<Station>, String> {
    let url = format!("https://{}/json/stations/search", API_SERVER);
    let query = search_query(params);

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

    let stations: Vec<Station> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(filter_stations_by_bitrate(stations, params.bitrate))
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

    let server = API_SERVER.to_string();
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

#[cfg(test)]
mod tests {
    use super::{SearchParams, Station, filter_stations_by_bitrate, search_query};

    fn station(id: &str, bitrate: u32) -> Station {
        Station {
            stationuuid: id.to_string(),
            name: format!("Station {}", id),
            url: format!("https://{}", id),
            url_resolved: String::new(),
            tags: String::new(),
            country_code: String::new(),
            language: String::new(),
            bitrate,
        }
    }

    #[test]
    fn search_query_contains_defaults_for_empty_filters() {
        let params = SearchParams::default();
        let query = search_query(&params);

        assert_eq!(
            query,
            vec![
                ("limit", "50".to_string()),
                ("offset", "0".to_string()),
                ("hidebroken", "true".to_string()),
                ("order", "clickcount".to_string()),
                ("reverse", "true".to_string()),
            ]
        );
    }

    #[test]
    fn search_query_normalizes_filter_values() {
        let params = SearchParams {
            name: " Jazz FM ".to_string(),
            tags: " jazz,blues ".to_string(),
            country: "de".to_string(),
            language: "EN".to_string(),
            bitrate: Some(128),
            limit: 25,
            offset: 50,
        };

        let query = search_query(&params);

        assert_eq!(
            query,
            vec![
                ("limit", "25".to_string()),
                ("offset", "50".to_string()),
                ("hidebroken", "true".to_string()),
                ("order", "clickcount".to_string()),
                ("reverse", "true".to_string()),
                ("name", "Jazz FM".to_string()),
                ("tagList", "jazz,blues".to_string()),
                ("countrycode", "DE".to_string()),
                ("language", "en".to_string()),
                ("bitrateMin", "128".to_string()),
            ]
        );
    }

    #[test]
    fn bitrate_filter_keeps_only_matching_stations() {
        let stations = vec![
            station("low", 64),
            station("mid", 128),
            station("high", 192),
        ];
        let filtered = filter_stations_by_bitrate(stations, Some(128));

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|station| station.bitrate >= 128));
        assert_eq!(filtered[0].stationuuid, "mid");
        assert_eq!(filtered[1].stationuuid, "high");
    }

    #[test]
    fn bitrate_filter_leaves_stations_unchanged_without_threshold() {
        let stations = vec![station("a", 32), station("b", 256)];
        let filtered = filter_stations_by_bitrate(stations.clone(), None);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].stationuuid, stations[0].stationuuid);
        assert_eq!(filtered[1].stationuuid, stations[1].stationuuid);
    }
}
