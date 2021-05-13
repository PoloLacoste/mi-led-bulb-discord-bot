use log::{error, info};
use std::env;
use futures::lock::Mutex;
use std::collections::HashMap;
use std::net::TcpStream;

#[macro_use]
extern crate lazy_static;

use serenity::client::{Client, Context};
use serenity::model::channel::Message;
use serenity::utils::MessageBuilder;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    Args,
    macros::{
        command,
        group
    }
};

use yeelight::{Bulb, Effect};

struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

#[group]
#[commands(color, colors)]
struct General;

lazy_static! {
    static ref COLORS: HashMap<&'static str, Rgb> = {
        let mut c = HashMap::new();
        c.insert("white", Rgb { red: 255, green: 255, blue: 255 });
        c.insert("red", Rgb { red: 255, green: 0, blue: 0 });
        c.insert("green", Rgb { red: 0, green: 255, blue: 0 });
        c.insert("blue", Rgb { red: 0, green: 0, blue: 255 });
        c.insert("yellow", Rgb { red: 255, green: 255, blue: 0 });
        c.insert("magenta", Rgb { red: 255, green: 0, blue: 255 });
        c.insert("cyan", Rgb { red: 0, green: 255, blue: 255 });
        c
    };
    static ref BULBS: Mutex<Vec<Bulb>> = {

        let mut bulbs: Vec<Bulb> = vec![];

        if let Ok(ips_string) = env::var("BULBS") {
            let ips: Vec<&str> = ips_string.split(",").collect();

            for ip in ips {
                let stream = TcpStream::connect((ip, 55443))
                    .expect("Connection failed");
                let bulb = Bulb::attach(stream).unwrap();
                bulbs.push(bulb);
            }
        }

        Mutex::new(bulbs)
    };
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting...");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("&"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[command]
async fn color(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() == 1 {
        let color: String = args.single::<String>().unwrap();
        let rgb = COLORS.get(color.to_lowercase().as_str());

        let mut builder = MessageBuilder::new();

        match rgb {
            Some(rgb) => {
                
                let red: u32 = rgb.red.into();
                let green: u32 = rgb.green.into();
                let blue: u32 = rgb.blue.into();
                let mut rgb_val: u32 = (red << 8) + green;
                rgb_val = (rgb_val << 8) + blue;

                let mut map = BULBS.lock().await;
                for bulb in map.iter_mut() {
                    bulb.set_rgb(rgb_val, Effect::Sudden, 0).await?;
                    bulb.set_bright(50, Effect::Sudden, 0).await?;
                }
                return Ok(());
            },
            None => {
                builder.push("Invalid color");
            }
        };

        let response = builder.build();
        msg.reply(ctx, &response).await?;
    }
    else {
        msg.reply(ctx, "Invalid command format, color COLOR_NAME").await?;
    }

    Ok(())
}

#[command]
async fn colors(ctx: &Context, msg: &Message) -> CommandResult {
    let mut builder = MessageBuilder::new();
    let mut index: u8 = 1;

    builder.push("\n");

    for (color_name, _) in COLORS.iter() {
        builder.push(index).push(". ").push(color_name).push("\n");
        index += 1;
    }

    msg.reply(ctx, builder.build()).await?;

    Ok(())
}