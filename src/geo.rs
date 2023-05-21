use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;

const NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org/search.php?format=jsonv2&q=";

#[derive(Deserialize, Debug)]
struct Location {
    lat: String,
    lon: String,
}

pub async fn coordinates(query: &str) -> Result<(f64, f64)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_coordinates_success() {
        let server = MockServer::start().await;

        let query = "New York";
        let expected_lat = "40.7128";
        let expected_lon = "-74.0060";

        let response_body = format!(r#"[{{"lat": "{expected_lat}", "lon": "{expected_lon}"}}]"#);

        Mock::given(method("GET"))
            .and(path("/search.php"))
            .and(query_param("format", "jsonv2"))
            .and(query_param("q", query))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(response_body, "application/json"),
            )
            .mount(&server)
            .await;

        let result = coordinates(query).await;
        assert!(result.is_ok());

        let (lat, lon) = result.unwrap();
        approx_eq!(f64, lat, expected_lat.parse::<f64>().unwrap(), ulps = 2);
        approx_eq!(f64, lon, expected_lon.parse::<f64>().unwrap(), ulps = 2);
        server.verify().await;
    }

    #[tokio::test]
    async fn test_coordinates_http_error() {
        let server = MockServer::start().await;

        let query = "InvalidQuery";

        Mock::given(method("GET"))
            .and(path("/search.php"))
            .and(query_param("format", "jsonv2"))
            .and(query_param("q", query))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let result = coordinates(query).await;
        assert!(result.is_err());
        server.verify().await;
    }

    #[tokio::test]
    async fn test_coordinates_no_results() {
        let server = MockServer::start().await;

        let query = "NonExistentPlace";

        let response_body = "[]";

        Mock::given(method("GET"))
            .and(path("/search.php"))
            .and(query_param("format", "jsonv2"))
            .and(query_param("q", query))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(response_body, "application/json"),
            )
            .mount(&server)
            .await;

        let result = coordinates(query).await;
        assert!(result.is_err());
        server.verify().await;
    }
}
