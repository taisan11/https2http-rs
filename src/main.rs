use actix_web::{web, App, HttpResponse,HttpRequest, HttpServer, Result};
use env_logger;
use env_logger::Env;
use log::{error, info};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use nojson::Json;
use nojson::{DisplayJson, FromRawJsonValue, JsonFormatter, JsonParseError, RawJsonValue};
use std::fs;

#[derive(Clone)]
struct Config {
    bind_address: String,
    bind_port: u16,
    log_level: String,
    auth: Option<AuthConfig>,
}

#[derive(Clone)]
struct AuthConfig {
    header_auth: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bind_address: "127.0.0.1".to_string(),
            bind_port: 8080,
            log_level: "info".to_string(),
            auth: None,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            header_auth: None,
        }
    }
}

impl DisplayJson for Config {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("bind_address", &self.bind_address)?;
            f.member("bind_port", self.bind_port)?;
            f.member("log_level", &self.log_level)?;
            f.member("auth", &self.auth)
        })
    }
}

impl DisplayJson for AuthConfig {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("header_auth", &self.header_auth)
        })
    }
}

impl<'text> FromRawJsonValue<'text> for Config {
    fn from_raw_json_value(value: RawJsonValue<'text, '_>) -> Result<Self, JsonParseError> {
        let ([bind_address, bind_port, log_level], [auth]) = value.to_fixed_object(
            ["bind_address", "bind_port", "log_level"], 
            ["auth"]
        )?;
        Ok(Config {
            bind_address: bind_address.try_to()?,
            bind_port: bind_port.try_to()?,
            log_level: log_level.try_to()?,
            auth: auth.map(|a| a.try_to()).transpose()?,
        })
    }
}

impl<'text> FromRawJsonValue<'text> for AuthConfig {
    fn from_raw_json_value(value: RawJsonValue<'text, '_>) -> Result<Self, JsonParseError> {
        let ([], [header_auth]) = value.to_fixed_object(
            [], 
            ["header_auth"]
        )?;
        Ok(AuthConfig {
            header_auth: header_auth.map(|h| h.try_to()).transpose()?,
        })
    }
}

fn load_config() -> Config {
    match fs::read_to_string("config.json") {
        Ok(content) => {
            match content.parse::<Json<Config>>() {
                Ok(config) => {
                    info!("Config loaded from file");
                    config.0
                }
                Err(e) => {
                    error!("Failed to parse config: {}", e);
                    info!("Using default config");
                    Config::default()
                }
            }
        }
        Err(_) => {
            info!("Config file not found, using default config");
            Config::default()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let config = load_config();
    let bind_address = config.bind_address.clone();
    let bind_port = config.bind_port;

    let client = Client::new();

    HttpServer::new(move || {
        let mut app =App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/", web::get().to(hello))
            .route("/proxy", web::to(proxy_handler));
        #[cfg(debug_assertions)]
        {
            app = app.route("/__dev", web::to(dev_endpoint));
        }
        app
    })
    .bind((bind_address, bind_port))?
    .run()
    .await
}

async fn hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Hello, world!"))
}

#[cfg(debug_assertions)]
async fn dev_endpoint(req: HttpRequest, body: web::Bytes) -> Result<HttpResponse> {
    let method = req.method().to_string();
    let uri = req.uri().to_string();
    let headers: Vec<String> = req.headers()
        .iter()
        .map(|(name, value)| format!("{}: {}", name, value.to_str().unwrap_or("")))
        .collect();
    
    let body_str = String::from_utf8_lossy(&body);
    
    let response_body = format!(
        "Method: {}\nURI: {}\nHeaders:\n{}\nBody:\n{}",
        method,
        uri,
        headers.join("\n"),
        body_str
    );
    
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(response_body))
}

fn convert_headers(actix_headers: &actix_web::http::header::HeaderMap) -> HeaderMap {
    let mut reqwest_headers = HeaderMap::new();

    for (key, value) in actix_headers.iter() {
        // Skip Header_Auth header
        if key.as_str().to_lowercase() == "header_auth" {
            continue;
        }
        
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            reqwest_headers.insert(name, val);
        }
    }

    reqwest_headers
}

async fn proxy_handler(
    req: actix_web::HttpRequest,
    body: web::Bytes,
    query: web::Query<std::collections::HashMap<String, String>>,
    client: web::Data<Client>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let method = req.method();
    let params = query.into_inner();
    //ヘッダーauthのチェック
    if let Some(auth_config) = &config.auth {
        if let Some(expected_auth) = &auth_config.header_auth {
            if let Some(auth_header) = req.headers().get("header_auth") {
                if let Ok(auth_value) = auth_header.to_str() {
                    if auth_value != expected_auth {
                        error!("Unauthorized access attempt with invalid header_auth");
                        return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
                    }
                } else {
                    error!("Invalid header_auth value");
                    return Ok(HttpResponse::BadRequest().body("Invalid header_auth value"));
                }
            } else {
                error!("Missing header_auth in request headers");
                return Ok(HttpResponse::BadRequest().body("Missing header_auth in request headers"));
            }
        }
    }
    match params.get("url") {
        Some(url) => {
            info!("Proxying request to URL: {}", url);
            let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::GET);
            let reqa = client.request(reqwest_method, url);
            let reqa = if !body.is_empty() {
                reqa.body(body.to_vec())
            } else {
                reqa
            };
            let reqa = if !params.get("header").is_none() {
                reqa.headers(convert_headers(req.headers()))
            } else {
                reqa
            };
            let resp = reqa.send().await.map_err(|e| {
                error!("Failed to send request: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to send request")
            })?;

            let status = resp.status();
            let headers = resp.headers().clone();
            let body = resp.bytes().await.map_err(|e| {
                error!("Failed to read response body: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to read response")
            })?;

            let actix_status = actix_web::http::StatusCode::from_u16(status.as_u16()).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
            let mut response = HttpResponse::build(actix_status);
            for (name, value) in headers.iter() {
                response.insert_header((name.as_str(), value.to_str().unwrap_or("")));
            }
            Ok(response.body(body.to_vec()))
        }
        None => {
            error!("No URL provided in query parameters");
            Ok(HttpResponse::BadRequest().body("Missing 'url' query parameter"))
        }
    }
}
