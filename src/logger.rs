use std::path::PathBuf;
use tracing::{info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

pub fn init_logger(log_dir: &str) {
    // 创建日志目录
    let log_path = PathBuf::from(log_dir);
    std::fs::create_dir_all(&log_path).expect("Failed to create log directory");

    // 配置文件日志
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_path,
        "server.log",
    );

    // 配置控制台日志
    let console_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE);

    // 配置文件日志
    let file_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(file_appender);

    // 配置日志过滤器
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // 初始化日志系统
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Logger initialized");
} 
