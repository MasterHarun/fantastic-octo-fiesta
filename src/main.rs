  //! Entry point of the application
  //!
  //! - Set up the Discord client and event handlers
  //! - Register application commands
  //!

use sensible_env_logger::try_init_custom_env_and_builder;
use std::{env, sync::Arc};
use clap::{Arg, Command};

use serenity::prelude::GatewayIntents;

mod commands;
mod handlers;
mod structures;
mod utils;
mod users;

use dotenvy::dotenv;

use crate::handlers::{HandlerStruct};
use crate::utils::get_env_var;
use crate::structures::{Config, ConfigStruct};

extern crate sensible_env_logger;
#[macro_use]
extern crate log;


#[tokio::main]
async fn main() {
	#![warn(clippy::disallowed_types)]
	
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
	.arg(
		Arg::new("rust_log")
		.short('r')
		.long("rust-log")
		.value_name("RUST_LOG")
		.help("Sets the Log level for the app")
		.default_value("info"),
	)
	.arg(
		Arg::new("global_log_level")
		.short('g')
		.long("global-log-level")
		.value_name("GLOBAL_LOG_LEVEL")
		.help("Sets the global logs for the app")
		.default_value("off")
	)
	.get_matches();
 //todo: rework this
	let api_key = get_env_var("OPENAI_API_KEY", "openai_api_key", Some(&matches));
	let discord_token = get_env_var("DISCORD_TOKEN", "discord_token", Some(&matches));
	let app_id = get_env_var("DISCORD_APP_ID", "discord_app_id", Some(&matches));
	let rust_log = get_env_var("RUST_LOG", "rust_log", Some(&matches));
	let global_logs = get_env_var("GLOBAL_LOG_LEVEL", "global_log_level", Some(&matches));
	
	let config: ConfigStruct = Config::new(api_key, discord_token, app_id, rust_log, global_logs);
  
	// Initialize the logger
  let _ = try_init_custom_env_and_builder(
		&config.rust_log,
		&config.global_log,
    env!("CARGO_PKG_NAME"),
    module_path!(),
    sensible_env_logger::pretty::formatted_timed_builder,
  );

	// todo: add ability to load from file or database
  let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
	let handler: HandlerStruct = HandlerStruct::new(Arc::new(config.clone()));
	
  let mut client = serenity::Client::builder(&config.discord_token, intents)
    .intents(intents)
    .event_handler(handler)
    .await
    .expect("Error creating client");

  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
}
