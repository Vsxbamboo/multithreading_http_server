use std::path::Path;
use std::env;

use tokio::fs;

pub async fn list_directory(path: &Path) -> tokio::io::Result<String> {
    let mut entries = fs::read_dir(path).await?;
    let mut result = format!(
        "Directory listing for /{}:\n\n",
        path.strip_prefix(env::current_dir().unwrap().join("public"))
            .unwrap()
            .display()
    );

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        if file_type.is_dir() {
            result.push_str(&format!("[DIR]  {}\n", file_name));
        } else if file_type.is_file() {
            result.push_str(&format!("       {}\n", file_name));
        } else {
            result.push_str(&format!("[???]  {}\n", file_name));
        }
    }

    Ok(result)
}
