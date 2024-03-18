use crate::modemd::get_modemd;
use std::error::Error;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::ParseMode,
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Display all SMS")]
    List(u8),
    #[command(description = "Delete SMS", parse_with = "split")]
    Delete { slot: u8, index: u32 },
}

pub struct SisBot {
    owner_id: i64,
    tg: Arc<Bot>,
}

impl SisBot {
    pub async fn new(token: &str, owner_id: i64) -> Result<SisBot, Box<dyn Error>> {
        log::debug!("Starting telegram bot with token {}", token);
        let tg = Arc::new(Bot::new(token));
        tg.set_my_commands(Command::bot_commands()).await?;

        let bot = SisBot {
            owner_id,
            tg: tg.clone(),
        };
        Ok(bot)
    }

    pub async fn run(&self) -> () {
        let owner_id = self.owner_id as u64;
        let handler = dptree::entry().branch(
            Update::filter_message()
                .filter(move |msg: Message| {
                    msg.from()
                        .map(|user| user.id.0 == owner_id)
                        .unwrap_or_default()
                })
                .filter_command::<Command>()
                .endpoint(answer),
        );
        let mut dispatcher = Dispatcher::builder(self.tg.clone(), handler)
            .default_handler(|upd| async move {
                log::warn!("unhandled update: {:?}", upd);
            })
            .error_handler(LoggingErrorHandler::with_custom_text(
                "an error has occurred in the dispatcher",
            ))
            .enable_ctrlc_handler()
            .build();

        dispatcher.dispatch().await;
    }

    pub async fn send_message_owner(&self, text: &str) -> Result<(), Box<dyn Error>> {
        self.tg
            .send_message(ChatId(self.owner_id), text)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }
}

async fn answer(msg: Message, tg: Arc<Bot>, cmd: Command) -> ResponseResult<()> {
    let modemd = get_modemd();
    match cmd {
        Command::List(slot) => {
            match modemd.get_sms_list(slot).await {
                Ok(messages) => {
                    if messages.len() == 0 {
                        tg.send_message(msg.chat.id, "No messages").await?;
                    } else {
                        for m in messages {
                            tg.send_message(msg.chat.id, m.to_html())
                                .parse_mode(ParseMode::Html)
                                .await?;
                        }
                    }
                }
                Err(e) => {
                    tg.send_message(msg.chat.id, format!("Error getting SMS list: {}", e))
                        .await?;
                }
            };
        }
        Command::Delete { slot, index } => {
            match modemd.delete_sms(slot, index).await {
                Ok(_) => {
                    tg.send_message(msg.chat.id, format!("SMS {}:{} deleted", slot, index))
                        .await?;
                }
                Err(e) => {
                    tg.send_message(msg.chat.id, format!("Error deleting SMS: {}", e))
                        .await?;
                }
            }
            log::info!(
                "Got delete command from {:?} on slot {}, index {}",
                msg.chat.id,
                slot,
                index
            );
        }
    };
    Ok(())
}
