use ext_php_rs::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

#[php_class]
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: i32,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[php_impl]
impl HttpResponse {
    pub fn __construct() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    /// Get response status
    pub fn status(&self) -> i32 {
        self.status
    }

    /// Get response headers
    pub fn headers(&self, name: String) -> Option<String> {
        self.headers.get(&name.to_lowercase()).cloned()
    }

    /// Get response body
    pub fn body(&self) -> String {
        self.body.clone()
    }

    /// Parse JSON from response body
    pub fn json(&self) -> PhpResult<HashMap<String, String>> {
        match serde_json::from_str::<serde_json::Value>(&self.body) {
            Ok(value) => {
                let mut result = HashMap::new();
                if let serde_json::Value::Object(map) = value {
                    for (k, v) in map {
                        result.insert(k, v.to_string());
                    }
                }
                Ok(result)
            }
            Err(e) => Err(PhpException::default(format!("JSON parse error: {}", e)).into()),
        }
    }
}

// HTTP functions in namespace Elephant\Net\Http

#[php_function]
pub fn request(
    method: String,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    timeout: Option<i32>,
) -> PhpResult<HttpResponse> {
    let mut req = match method.to_uppercase().as_str() {
        "GET" => ureq::get(&url),
        "POST" => ureq::post(&url),
        "PUT" => ureq::put(&url),
        "DELETE" => ureq::delete(&url),
        "PATCH" => ureq::patch(&url),
        "HEAD" => ureq::head(&url),
        _ => return Err(PhpException::default(format!("Unsupported method: {}", method)).into()),
    };

    if let Some(timeout_secs) = timeout {
        req = req.timeout(Duration::from_secs(timeout_secs as u64));
    }

    if let Some(headers_map) = headers {
        for (key, value) in headers_map {
            req = req.set(&key, &value);
        }
    }

    let response = if let Some(body_data) = body {
        req.send_string(&body_data)
    } else {
        req.call()
    };

    match response {
        Ok(resp) => {
            let status = resp.status() as i32;

            let mut headers = HashMap::new();
            for header_name in resp.headers_names() {
                if let Some(header_value) = resp.header(&header_name) {
                    headers.insert(header_name.to_lowercase(), header_value.to_string());
                }
            }

            let body = resp.into_string()
                .map_err(|e| PhpException::default(format!("Response body error: {}", e)))?;

            Ok(HttpResponse {
                status,
                headers,
                body,
            })
        }
        Err(e) => Err(PhpException::default(format!("HTTP request failed: {}", e)).into()),
    }
}

/// HTTP GET request
#[php_function]
pub fn get(url: String, headers: Option<HashMap<String, String>>) -> PhpResult<HttpResponse> {
    request("GET".to_string(), url, headers, None, Some(30))
}

/// HTTP POST request
#[php_function]
pub fn post(
    url: String,
    body: Option<String>,
    headers: Option<HashMap<String, String>>,
) -> PhpResult<HttpResponse> {
    let mut final_headers = headers.unwrap_or_default();

    // If not set "Content-Type", by default JSON
    if !final_headers.contains_key("content-type") && !final_headers.contains_key("Content-Type") {
        final_headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    request("POST".to_string(), url, Some(final_headers), body, Some(30))
}

/// HTTP PUT request
#[php_function]
pub fn put(
    url: String,
    body: Option<String>,
    headers: Option<HashMap<String, String>>,
) -> PhpResult<HttpResponse> {
    let mut final_headers = headers.unwrap_or_default();

    if !final_headers.contains_key("content-type") && !final_headers.contains_key("Content-Type") {
        final_headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    request("PUT".to_string(), url, Some(final_headers), body, Some(30))
}

/// HTTP DELETE request
#[php_function]
pub fn delete(url: String, headers: Option<HashMap<String, String>>) -> PhpResult<HttpResponse> {
    request("DELETE".to_string(), url, headers, None, Some(30))
}

/// HTTP PATCH request
#[php_function]
pub fn patch(
    url: String,
    body: Option<String>,
    headers: Option<HashMap<String, String>>,
) -> PhpResult<HttpResponse> {
    let mut final_headers = headers.unwrap_or_default();

    if !final_headers.contains_key("content-type") && !final_headers.contains_key("Content-Type") {
        final_headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    request("PATCH".to_string(), url, Some(final_headers), body, Some(30))
}

/// HTTP HEAD request
#[php_function]
pub fn head(url: String, headers: Option<HashMap<String, String>>) -> PhpResult<HttpResponse> {
    request("HEAD".to_string(), url, headers, None, Some(30))
}

/// HTTP OPTIONS request
#[php_function]
pub fn options(url: String, headers: Option<HashMap<String, String>>) -> PhpResult<HttpResponse> {
    request("OPTIONS".to_string(), url, headers, None, Some(30))
}


// TODO: Move to Url

#[php_function]
pub fn build_query(params: HashMap<String, String>) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{}={}", urlencoding::encode(key), urlencoding::encode(value)))
        .collect::<Vec<String>>()
        .join("&")
}

#[php_function]
pub fn parse_url(url: String) -> PhpResult<HashMap<String, String>> {
    match url::Url::parse(&url) {
        Ok(parsed) => {
            let mut result = HashMap::new();
            result.insert("scheme".to_string(), parsed.scheme().to_string());
            result.insert("host".to_string(), parsed.host_str().unwrap_or("").to_string());
            result.insert("path".to_string(), parsed.path().to_string());

            if let Some(query) = parsed.query() {
                result.insert("query".to_string(), query.to_string());
            }

            if let Some(port) = parsed.port() {
                result.insert("port".to_string(), port.to_string());
            }

            Ok(result)
        }
        Err(e) => Err(PhpException::default(format!("Invalid URL: {}", e)).into()),
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}