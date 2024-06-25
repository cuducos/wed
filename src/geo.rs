use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::persistence::SavedEvents;

const NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org/search.php?format=jsonv2&q=";

#[derive(Deserialize, Debug)]
struct Location {
    lat: String,
    lon: String,
}

fn coordinates_from_saved_events(query: &str) -> Option<(f64, f64)> {
    if let Ok(saved) = SavedEvents::from_file() {
        for event in saved.events.iter() {
            if query == event.location {
                return Some((event.latitude, event.longitude));
            }
        }
    }
    None
}

pub async fn coordinates(query: &str) -> Result<(f64, f64)> {
    if let Some(coordinates) = coordinates_from_saved_events(query) {
        return Ok(coordinates);
    }
    let url = format!("{NOMINATIM_URL}{query}");
    let user_agent = format!(
        "{}/{} ({})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY"),
    );

    let resp = Client::new()
        .get(&url)
        .header("User-Agent", user_agent)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "HTTP request to {} returned {}: {}",
            &url,
            resp.status(),
            resp.text().await?
        ));
    }

    let results: Vec<Location> = resp.json().await?;
    if results.is_empty() {
        return Err(anyhow!("No latitude/longitude found for {}", query));
    }
    Ok((
        results[0].lat.parse::<f64>()?,
        results[0].lon.parse::<f64>()?,
    ))
}
