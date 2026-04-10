use teloxide::{prelude::*, Bot};

pub fn start_telegram(token: &str) {
    let bot = Bot::new(token);
    
    tokio::spawn(async move {
        println!("🤖 Telegram bot thread running!");
        
        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            if let Some(text) = msg.text() {
                println!("📱 Telegram: {}", text);
                bot.send_message(msg.chat.id, "AMOR is online!").await.ok();
            }
            respond(())
        }).await;
    });
}
