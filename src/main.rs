use dotenv::dotenv;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use chrono::prelude::*;

mod bot;
mod message;
mod modemd;
use bot::SisBot;

async fn check_sms_task(slots: &[u8], should_stop: Arc<AtomicBool>, bot: Arc<SisBot>) {
    let modemd = modemd::get_modemd();
    let mut last_check = Utc::now();
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        for slot in slots {
            match modemd.get_sms_list(*slot).await {
                Ok(sms_list) => {
                    for msg in sms_list {
                        let datetime = match msg.datetime() {
                            Ok(dt) => dt,
                            Err(e) => {
                                log::error!("Error parsing datetime: {}", e);
                                continue;
                            }
                        };
                        if (datetime.with_timezone(&Utc) - last_check).num_seconds() < 0 {
                            continue;
                        }
                        let _ = bot.send_message_owner(&msg.to_html()).await;
                    }
                }
                Err(e) => {
                    log::error!("Error getting SMS list: {}", e);
                }
            };
        }
        last_check = Utc::now();
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok(); // Load .env file
    env_logger::init(); // Set log level through RUST_LOG

    // Get token variables
    let owner_id = std::env::var("TELEGRAM_OWNER_ID")
        .expect("Owner ID should be set")
        .parse::<i64>()
        .expect("Owner ID should be a number");
    let token = std::env::var("TELEGRAM_API_TOKEN")
        .expect("Bot token should be set");
    if let Ok(url) = std::env::var("MODEMD_URL") {
        modemd::set_base_url(url);
    };

    // Create Sistematics Bot
    let bot = match SisBot::new(token.as_str(), owner_id).await {
        Ok(bot) => Arc::new(bot),
        Err(e) => {
            panic!("Error creating Telegram bot: {}", e)
        }
    };

    // Create a thread to check new SMS messages
    let should_stop = Arc::new(AtomicBool::new(false));
    let shared_should_stop = should_stop.clone();
    let shared_bot = bot.clone();
    let messages_handler = tokio::spawn(async move {
        check_sms_task(&[2, 3], shared_should_stop, shared_bot).await;
    });

    // Run the bot indefinitely
    bot.run().await;
   
    // Shutdown program
    should_stop.store(true, Ordering::Relaxed);
    messages_handler.await?;
    Ok(())
}
