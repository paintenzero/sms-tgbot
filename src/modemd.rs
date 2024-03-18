use crate::message::Message;
use serde::Deserialize;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref G_MODEMD_BASE_URL: Mutex<String> = Mutex::new("http://127.0.0.1:222".to_string());
}

pub fn get_modemd() -> ModemdInfo {
    let base_url = G_MODEMD_BASE_URL.lock().unwrap().clone();
    ModemdInfo::new(base_url)
}

pub fn set_base_url(url: String) {
    let mut base_url = G_MODEMD_BASE_URL.lock().unwrap();
    base_url.clear();
    base_url.push_str(url.as_str());
}
pub struct ModemdInfo {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct ResultModemd {
    result_code: String,
    result: String
}

impl ModemdInfo {
    pub fn new(base_url: String) -> ModemdInfo {
        ModemdInfo {
            base_url: base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_sms_list(&self, slot: u8) -> Result<Vec<Message>, String> {
        return match self.client.get(format!("{}/sms_list?M{}", self.base_url, slot)).send().await {
            Ok(resp) => {
                if resp.status() != 200 {
                    return Err(format!("Error getting SMS list: Status is not 200 (Got: {})", resp.status()));
                }
                match resp.json::<Vec<Message>>().await {
                    Ok(messages) => {
                        Ok(messages)
                    },
                    Err(e) => Err(e.to_string()),
                }
            },
            Err(e) => Err(e.to_string()),
        };
    }

    pub async fn delete_sms(&self, slot: u8, message_index: u32) -> Result<(), String> {
        match self.client.get(format!("{}/sms_delete?M{}&index={}", self.base_url, slot, message_index)).send().await {
            Ok(resp) => {
                if resp.status() != 200 {
                    return Err(format!("Status is not 200 (Got: {})", resp.status()));
                }
                match resp.json::<ResultModemd>().await {
                    Ok(body) => {
                        if body.result != "OK" {
                            return Err(format!("Got result {} ({})", body.result, body.result_code));
                        }
                    },
                    Err(e) => return Err(e.to_string()),
                }
            }
            Err(e) => return Err(e.to_string()),
        };
        Ok(())
    }
}
