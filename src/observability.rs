use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{RandomIdGenerator, Sampler, Tracer},
    Resource,
};
use std::collections::HashMap;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub service_name: String,
    pub log_format: LogFormat,
    pub file_logging_enabled: bool,
    pub log_file_dir: String,
    pub log_file_name: String,
    pub otel_enabled: bool,
    pub otel_endpoint: String,
    pub otel_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "hexagonal-rust".to_string(),
            log_format: LogFormat::Pretty,
            file_logging_enabled: false,
            log_file_dir: "./logs".to_string(),
            log_file_name: "hexagonal-rust.log".to_string(),
            otel_enabled: false,
            otel_endpoint: "http://localhost:4317".to_string(),
            otel_headers: HashMap::new(),
        }
    }
}

impl ObservabilityConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(name) = std::env::var("OTEL_SERVICE_NAME") {
            config.service_name = name;
        }

        if let Ok(format) = std::env::var("LOG_FORMAT") {
            config.log_format = match format.to_lowercase().as_str() {
                "json" => LogFormat::Json,
                _ => LogFormat::Pretty,
            };
        }

        if let Ok(enabled) = std::env::var("LOG_FILE_ENABLED") {
            config.file_logging_enabled = enabled.to_lowercase() == "true";
        }

        if let Ok(path) = std::env::var("LOG_FILE_PATH") {
            if let Some(parent) = std::path::Path::new(&path).parent() {
                config.log_file_dir = parent.to_string_lossy().to_string();
            }
            if let Some(name) = std::path::Path::new(&path).file_name() {
                config.log_file_name = name.to_string_lossy().to_string();
            }
        }

        if let Ok(enabled) = std::env::var("OTEL_TRACING_ENABLED") {
            config.otel_enabled = enabled.to_lowercase() == "true";
        }

        if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            config.otel_endpoint = endpoint;
        }

        if let Ok(headers_str) = std::env::var("OTEL_EXPORTER_OTLP_HEADERS") {
            config.otel_headers = parse_otlp_headers(&headers_str);
        }

        config
    }
}

fn parse_otlp_headers(headers_str: &str) -> HashMap<String, String> {
    headers_str
        .split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.trim().to_string(), value.trim().to_string())),
                _ => None,
            }
        })
        .collect()
}

pub struct ObservabilityGuard {
    _file_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

fn init_tracer(config: &ObservabilityConfig) -> Result<Tracer, opentelemetry::trace::TraceError> {
    let mut metadata = tonic::metadata::MetadataMap::new();

    for (key, value) in &config.otel_headers {
        if let (Ok(header_name), Ok(header_value)) = (
            tonic::metadata::MetadataKey::from_bytes(key.as_bytes()),
            tonic::metadata::MetadataValue::try_from(value.as_str()),
        ) {
            metadata.insert(header_name, header_value);
        }
    }

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otel_endpoint)
        .with_metadata(metadata);

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", config.service_name.clone()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(runtime::Tokio)
}

pub fn init_observability(config: ObservabilityConfig) -> anyhow::Result<ObservabilityGuard> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("hexagonal_rust=debug,tower_http=debug"));

    let console_layer: Box<dyn Layer<_> + Send + Sync> = if config.log_format == LogFormat::Json {
        Box::new(
            fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(true)
                .with_level(true)
                .with_file(false)
                .with_line_number(false),
        )
    } else {
        Box::new(
            fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(true)
                .with_level(true),
        )
    };

    let (file_layer, file_guard): (Option<Box<dyn Layer<_> + Send + Sync>>, _) = if config
        .file_logging_enabled
    {
        let file_appender =
            RollingFileAppender::new(Rotation::DAILY, &config.log_file_dir, &config.log_file_name);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_level(true)
            .with_ansi(false);

        (Some(Box::new(layer)), Some(guard))
    } else {
        (None, None)
    };

    let otel_layer: Option<tracing_opentelemetry::OpenTelemetryLayer<_, _>> = if config.otel_enabled
    {
        match init_tracer(&config) {
            Ok(tracer) => {
                eprintln!("OpenTelemetry tracer initialized successfully for endpoint: {}", config.otel_endpoint);
                Some(tracing_opentelemetry::layer().with_tracer(tracer))
            }
            Err(e) => {
                eprintln!("Failed to initialize OpenTelemetry tracer: {}", e);
                eprintln!("  Endpoint: {}", config.otel_endpoint);
                eprintln!("  Headers: {:?}", config.otel_headers.keys().collect::<Vec<_>>());
                None
            }
        }
    } else {
        None
    };

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .with(otel_layer);

    subscriber.init();

    tracing::info!(
        service.name = %config.service_name,
        log_format = ?config.log_format,
        file_logging = config.file_logging_enabled,
        otel_enabled = config.otel_enabled,
        "observability initialized"
    );

    Ok(ObservabilityGuard {
        _file_guard: file_guard,
    })
}

pub fn shutdown_tracer() {
    opentelemetry::global::shutdown_tracer_provider();
}
