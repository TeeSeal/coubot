mod coub;

use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    async_trait,
    http::AttachmentType,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::env;

// 8MB file size limit by discord :(
const MAX_SIZE: u64 = 7_500_000;
lazy_static! {
    static ref COUB_REGEX: Regex = Regex::new(r"(https?://)?(www\.)?coub\.com/[\w/]+").unwrap();
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(url_match) = COUB_REGEX.find(&msg.content) {
            let mut status_message = msg
                .channel_id
                .say(&ctx.http, "Working on it...")
                .await
                .unwrap();

            if let Ok(c) = coub::fetch_coub(url_match.as_str()).await {
                let loops = (MAX_SIZE / c.size) as usize;

                let path = tempfile::Builder::new()
                    .prefix(&c.id)
                    .suffix(".mp4")
                    .rand_bytes(0)
                    .tempfile()
                    .unwrap()
                    .into_temp_path();

                let result = if loops < 2 {
                    c.download(&path).await
                } else {
                    c.download_loops(&path, loops).await
                };

                match result {
                    Ok(()) => {
                        let _ = msg
                            .channel_id
                            .send_message(&ctx.http, |m| m.add_file(AttachmentType::Path(&path)))
                            .await;
                        let _ = status_message.delete(&ctx).await;
                    }
                    Err(why) => {
                        println!("Error converting coub {}\n{:?}", c.id, why);
                        let _ = status_message
                            .edit(&ctx, |m| {
                                m.content(format!("Error converting coub {}\n{:?}", c.id, why))
                            })
                            .await;
                    }
                }

                path.close().unwrap();
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("TOKEN").expect("TOKEN not found in environment.");
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
