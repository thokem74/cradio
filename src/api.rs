use serde::Deserialize;

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
            limit: 50,
            offset: 0,
        }
    }
}

fn resolve_api_server() -> String {
    // Try to resolve a working radio-browser.info server
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

    Ok(stations)
}
