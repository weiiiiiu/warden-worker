use worker::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// 备份数据库到坚果云 WebDAV
pub async fn backup_to_jianguoyun(env: &Env) -> Result<String> {
    let db = env.d1("vault1")?;
    
    // 获取坚果云配置
    let webdav_user = env.secret("JIANGUOYUN_USER")?.to_string();
    let webdav_pass = env.secret("JIANGUOYUN_PASS")?.to_string();
    let webdav_path = env.var("JIANGUOYUN_PATH")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "/dav/warden-backup".to_string());
    
    // 导出所有表数据为 JSON
    let backup_data = export_database(&db).await?;
    
    // 生成带时间戳的文件名
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("warden_backup_{}.json", timestamp);
    
    // 上传到坚果云
    let url = format!("https://dav.jianguoyun.com{}/{}", webdav_path, filename);
    let auth = BASE64.encode(format!("{}:{}", webdav_user, webdav_pass));
    
    let headers = Headers::new();
    headers.set("Authorization", &format!("Basic {}", auth))?;
    headers.set("Content-Type", "application/json")?;
    
    let request = Request::new_with_init(
        &url,
        RequestInit::new()
            .with_method(Method::Put)
            .with_headers(headers)
            .with_body(Some(wasm_bindgen::JsValue::from_str(&backup_data))),
    )?;
    
    let response = Fetch::Request(request).send().await?;
    
    if response.status_code() >= 200 && response.status_code() < 300 {
        console_log!("Backup successful: {}", filename);
        Ok(format!("Backup successful: {}", filename))
    } else {
        let error_msg = format!("Backup failed with status: {}", response.status_code());
        console_error!("{}", error_msg);
        Err(Error::from(error_msg))
    }
}

/// 导出数据库所有表为 JSON
async fn export_database(db: &D1Database) -> Result<String> {
    // 导出 users 表
    let users: Vec<serde_json::Value> = db
        .prepare("SELECT * FROM users")
        .all()
        .await?
        .results()?;
    
    // 导出 ciphers 表
    let ciphers: Vec<serde_json::Value> = db
        .prepare("SELECT * FROM ciphers")
        .all()
        .await?
        .results()?;
    
    // 导出 folders 表
    let folders: Vec<serde_json::Value> = db
        .prepare("SELECT * FROM folders")
        .all()
        .await?
        .results()?;
    
    // 组合成完整备份
    let backup = serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "tables": {
            "users": users,
            "ciphers": ciphers,
            "folders": folders
        }
    });
    
    serde_json::to_string_pretty(&backup)
        .map_err(|e| Error::from(format!("JSON serialization error: {}", e)))
}
