pub mod aggregation;
pub mod models;

use std::{env, sync::Arc, time::Duration};

use chrono::{FixedOffset, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc};
use lazy_static::lazy_static;
use models::ScheduleBlock;
use mongodb::options::ClientOptions;
use poise::serenity_prelude::{self as serenity, Cache, ChannelId, GuildId, Http};

// Based on where the comp is from utc time (EDT for our team)
const TZ_OFFSET: i32 = 4;

#[derive(Debug, Clone)]
struct CompTZ;

impl Offset for CompTZ {
    fn fix(&self) -> FixedOffset {
        FixedOffset::west_opt(TZ_OFFSET * 60 * 60).unwrap()
    }
}

impl TimeZone for CompTZ {
    type Offset = CompTZ;

    fn from_offset(_offset: &Self::Offset) -> Self {
        CompTZ
    }

    fn offset_from_local_date(&self, _local: &NaiveDate) -> LocalResult<Self> {
        LocalResult::Single(CompTZ)
    }

    fn offset_from_local_datetime(&self, _local: &NaiveDateTime) -> LocalResult<Self> {
        LocalResult::Single(CompTZ)
    }

    fn offset_from_utc_date(&self, _utc: &NaiveDate) -> Self {
        CompTZ
    }

    fn offset_from_utc_datetime(&self, _utc: &NaiveDateTime) -> Self {
        CompTZ
    }
}

// User data, which is stored and accessible in all command invocations
#[derive(Debug, Clone)]
struct Data {
    mongo_client: mongodb::Client,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

lazy_static! {
    pub static ref DB: String = env::var("DEFAULT_DB").unwrap();
    pub static ref CHANNEL_ID: ChannelId =
        ChannelId(env::var("CHANNEL_ID").unwrap().parse().unwrap());
    pub static ref GUILD_ID: GuildId = GuildId(env::var("GUILD_ID").unwrap().parse().unwrap());
    pub static ref CLIENT_ID: u64 = env::var("CLIENT_ID").unwrap().parse().unwrap();
}

/// Displays your or another user's account creation date
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    ctx.say("pong").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_filename(".env.production").ok();
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();

    let mongo_options = ClientOptions::parse(&env::var("DB_URI").unwrap()).await?;
    let mongo_client = mongodb::Client::with_options(mongo_options)?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .token(std::env::var("TOKEN").expect("missing TOKEN in environment"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let cache = ctx.cache.clone();
                let http = ctx.http.clone();
                tokio::spawn(ping_scouters_task(cache, http, mongo_client.clone()));

                Ok(Data { mongo_client })
            })
        });

    framework.run().await.unwrap();
    Ok(())
}

async fn ping_scouters_task(
    cache: Arc<Cache>,
    http: Arc<Http>,
    mongo_client: mongodb::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = mongo_client.database(&*DB);
    let blocks = db.collection::<ScheduleBlock>("scheduleBlocks");

    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let now = Utc::now();
        let block = ScheduleBlock::find_all_populated(&blocks)
            .await?
            .into_iter()
            // Remove blocks that have passed
            .filter(|block| block.start_time.to_chrono() > now)
            // Take first block
            .next();

        if let Some(mut block) = block {
            // If no scouters, go to next iteration
            if block.blue1.is_none()
                && block.blue2.is_none()
                && block.blue3.is_none()
                && block.red1.is_none()
                && block.red2.is_none()
                && block.red3.is_none()
            {
                continue;
            }

            let time_till_block = block.start_time.to_chrono() - now;
            let scouting_in: String =
                if time_till_block <= chrono::Duration::minutes(30) && !block.min_30 {
                    // Set that min 30 warning has gone out
                    block.update_min_30(&blocks).await?;
                    "30 minutes".into()
                } else if time_till_block <= chrono::Duration::minutes(10) && !block.min_10 {
                    // Set that min 10 warning has gone out
                    block.update_min_10(&blocks).await?;
                    "10 minutes".into()
                } else {
                    // No more warnings to send, continue to next iteration
                    continue;
                };

            CHANNEL_ID
                .send_message(&http, |m| {
                    m.content(block.pings()).embed(|e| {
                        if let Some(blue1) = block.blue1 {
                            e.field(
                                format!("{} {}", blue1.first_name, blue1.last_name),
                                "You are scouting blue 1",
                                false,
                            )
                        } else {
                            e.field("No One 😔", "Is scouting blue 1", false)
                        };

                        if let Some(blue2) = block.blue2 {
                            e.field(
                                format!("{} {}", blue2.first_name, blue2.last_name),
                                "You are scouting blue 2",
                                false,
                            )
                        } else {
                            e.field("No One 😔", "Is scouting blue 2", false)
                        };

                        if let Some(blue3) = block.blue3 {
                            e.field(
                                format!("{} {}", blue3.first_name, blue3.last_name),
                                "You are scouting blue 3",
                                false,
                            )
                        } else {
                            e.field("No One 😔", "Is scouting blue 3", false)
                        };

                        if let Some(red1) = block.red1 {
                            e.field(
                                format!("{} {}", red1.first_name, red1.last_name),
                                "You are scouting red 1",
                                false,
                            )
                        } else {
                            e.field("No One 😔", "Is scouting red 1", false)
                        };

                        if let Some(red2) = block.red2 {
                            e.field(
                                format!("{} {}", red2.first_name, red2.last_name),
                                "You are scouting red 2",
                                false,
                            )
                        } else {
                            e.field("No One 😔", "Is scouting red 2", false)
                        };

                        if let Some(red3) = block.red3 {
                            e.field(
                                format!("{} {}", red3.first_name, red3.last_name),
                                "You are scouting red 3",
                                false,
                            )
                        } else {
                            e.field("No One 😔", "Is scouting red 3", false)
                        };

                        e.color((84, 182, 229))
                            .title(format!(
                                "Scouters for {} - {}, in {scouting_in}",
                                block
                                    .start_time
                                    .to_chrono()
                                    .with_timezone(&CompTZ)
                                    .time()
                                    .format("%H:%M"),
                                block
                                    .end_time
                                    .to_chrono()
                                    .with_timezone(&CompTZ)
                                    .time()
                                    .format("%H:%M")
                            ))
                            .footer(|f| f.text("Sussy scouter has been oxidized 🦀, rejoice!"))
                    })
                })
                .await?;
        }
    }
}

fn ping_str(str: impl AsRef<str>) -> String {
    format!("<@{}>", str.as_ref())
}
