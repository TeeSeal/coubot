mod coub;

use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    http::AttachmentType,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tempfile::Builder;
use std::env;

// This is the amount of seconds of mp4 video I found to be under 8 MB in most cases.
// Not super reliable but works for me.
const MAX_SECONDS: f64 = 21_f64;
lazy_static! {
    static ref COUB_REGEX: Regex = Regex::new(r"(https?://)?(www\.)?coub\.com/[\w/]+").unwrap();
}

struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if let Some(url_match) = COUB_REGEX.find(&msg.content) {
            if let Ok(c) = coub::fetch_coub(url_match.as_str()) {
                let loops = (MAX_SECONDS / c.duration) as usize;

                let path = Builder::new()
                    .prefix(&c.id)
                    .suffix(".mp4")
                    .rand_bytes(0)
                    .tempfile()
                    .unwrap()
                    .into_temp_path();

                match c.download_loops(&path, loops) {
                    Ok(()) => {
                        let _ = msg.channel_id.send_message(&ctx.http, |m| {
                            m.add_file(AttachmentType::Path(&path))
                        });
                    },
                    Err(why) => {
                        println!("Error converting coub {}\n{:?}", c.id, why);
                    }
                }

                path.close().unwrap();
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let token = env::var("TOKEN").expect("TOKEN not found in environment.");
    let mut client = Client::new(&token, Handler).expect("Error creating client");
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
