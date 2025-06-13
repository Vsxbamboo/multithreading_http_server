use serde::Deserialize;
use tokio::{fs, sync::OnceCell};

// #[derive(Debug, Deserialize, Clone)]
#[derive(Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub static_dir: String,
}

#[derive(Debug)]
pub enum ConfigError {
    ReadConfigFileFail,
    ConfigFormatError,
}

const CONFIG_PATH: &str = "./config.json";

// 全局 OnceCell 缓存配置
static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn read_config() -> Result<Config, ConfigError> {
    // 如果已经初始化过，返回副本
    if let Some(cfg) = CONFIG.get() {
        return Ok(cfg.clone());
    }

    // 否则异步读取并存入全局缓存
    let content = fs::read_to_string(CONFIG_PATH)
        .await
        .map_err(|_| ConfigError::ReadConfigFileFail)?;

    let config: Config =
        serde_json::from_str(&content).map_err(|_| ConfigError::ConfigFormatError)?;

    let _ = CONFIG.set(config.clone()); // 并发时可能已经 set，无需报错

    Ok(config)
}
