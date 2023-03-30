  //! Entry point of the application
  //!
  //! - Set up the Discord client and event handlers
  //! - Register application commands
  //!

use sensible_env_logger::try_init_custom_env_and_builder;
use std::{env, sync::Arc};
use clap::{Arg, Command};

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

let matches = Command::new("RustGPT-Discord Bot")
	.version("1.0")
	.author("MasterHarun")
	.about("A Discord bot with AI chat functionality")
	.arg(
		Arg::new("discord_token")
		.short('t')
		.long("discord-token")
		.value_name("DISCORD_TOKEN")
		.help("Sets the Discord bot token"),
	)
	.arg(
		Arg::new("discord_app_id")
		.short('a')
		.long("discord-app-id")
		.value_name("DISCORD_APP_ID")
		.help("Sets the Discord app ID"),
	)
	.arg(
		Arg::new("openai_api_key")
		.short('o')
		.long("openai-api-key")
		.value_name("OPENAI_API_KEY")
		.help("Sets the OPENAI API key"),
	)
	.get_matches();

	let discord_token = if let Some(token) = matches.get_one::<String>("discord_token") {
		token.to_string()
	} else {
		env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
			dotenvy::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set")
		})
	};

	let application_id = if let Some(id) = matches.get_one::<String>("discord_app_id") {
		id.to_string()
	} else {
		env::var("DISCORD_APP_ID").unwrap_or_else(|_| {
			dotenvy::var("DISCORD_APP_ID").expect("DISCORD_APP_ID must be set")
		})
	};

	let _api_key = if let Some(key) = matches.get_one::<String>("openai_api_key") {
		key.to_string()
	} else {
		std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
			dotenvy::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
		})
	};

  // let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
  // let application_id =
    // env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID not found");

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
    &discord_token,
    application_id.parse::<u64>().unwrap(),
  ));
  let mut client = serenity::Client::builder(&discord_token, intents)
    .intents(intents)
    .event_handler(Handler::new(Arc::clone(&http)))
    .await
    .expect("Error creating client");

  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
}
