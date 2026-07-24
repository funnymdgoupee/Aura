//! 二维码配对 — 根据连接模式生成不同格式的 URL
//!
//! 局域网：aura://pair?mode=lan&ip=192.168.1.5&port=8765&session=xxx
//! 中继：aura://pair?mode=relay&url=wss://server.com/relay&room=xxx&key=yyy
//!
//! iPhone 扫码后自动识别模式并解析参数

use base64::Engine;
use qrcode::QrCode;
use tauri::AppHandle;

use crate::config::{AppConfig, ConnectionMode};

#[derive(Debug)]
pub struct PairingInfo {
    pub qr_data_url: String, // data:image/png;base64,...
    pub raw_url: String,     // aura://pair?...
}

pub fn generate_pairing_qr(app: &AppHandle, config: &AppConfig) -> Result<PairingInfo, String> {
    let session_id = uuid::Uuid::new_v4().to_string();

    let raw_url = match config.connection_mode {
        ConnectionMode::Lan => {
            let ip = get_local_ip().ok_or("无法获取本机 IP")?;
            format!(
                "aura://pair?mode=lan&ip={}&port={}&session={}",
                ip, config.server_port, session_id
            )
        }
        ConnectionMode::Relay => {
            if config.relay_server_url.is_empty() {
                return Err("未配置中继服务器地址".into());
            }
            format!(
                "aura://pair?mode=relay&url={}&room={}&key={}",
                urlencoding::encode(&config.relay_server_url),
                session_id,
                config.relay_secret_key,
            )
        }
    };

    let code = QrCode::new(&raw_url).map_err(|e| e.to_string())?;
    let image = code.render::<image::Luma<u8>>().min_dimensions(256, 256).build();

    // 编码成 PNG base64
    let mut png_buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageLuma8(image)
        .to_rgb8()
        .write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_buf.into_inner());
    let qr_data_url = format!("data:image/png;base64,{}", b64);

    Ok(PairingInfo { qr_data_url, raw_url })
}

/// 获取本机 IPv4 地址（局域网地址，非 loopback）
fn get_local_ip() -> Option<String> {
    use local_ip_address::local_ip;
    match local_ip() {
        Ok(ip) => Some(ip.to_string()),
        Err(_) => None,
    }
}
