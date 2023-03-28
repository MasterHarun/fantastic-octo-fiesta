  //! Entry point of the application
  //!
  //! - Set up the Discord client and event handlers
  //! - Register application commands
  //!

use sensible_env_logger::try_init_custom_env_and_builder;
use std::{env, sync::Arc};

use serenity::{http::Http, prelude::GatewayIntents};

mod commands;
mod handlers;
mod structures;
mod utils;

use dotenvy::dotenv;

use crate::handlers::Handler;

extern crate sensible_env_logger;
#[macro_use]
extern crate log;


#[tokio::main]
async fn main() {

  dotenv().ok();
  info!("running");

  let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
  let application_id =
    env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID not found");

  // Initialize the logger
  let _ = try_init_custom_env_and_builder(
    &env::var("RUST_LOG").expect("RUST_LOG not found"),
    &env::var("GLOBAL_LOG_LEVEL").expect("GLOBAL_LOG_LEVEL not found"),
    env!("CARGO_PKG_NAME"),
    module_path!(),
    sensible_env_logger::pretty::formatted_timed_builder,
  );

  let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

  let http = Arc::new(Http::new_with_application_id(
    &token,
    application_id.parse::<u64>().unwrap(),
  ));
  let mut client = serenity::Client::builder(&token, intents)
    .intents(intents)
    .event_handler(Handler::new(Arc::clone(&http)))
    .await
    .expect("Error creating client");

  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
}
