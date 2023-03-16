use serenity::{
  async_trait,
  model::{gateway::Ready, prelude::{command::{CommandOptionType, Command}, interaction::{InteractionResponseType, Interaction}}},
  prelude::*,
};
use std::env;
use reqwest::Client;
use serde_json::{Value, json};
use dotenv::dotenv;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
      if let Interaction::ApplicationCommand(command) = interaction {
          let content = command.data.options[0].value.as_ref().unwrap().as_str().unwrap();
          let model = "gpt-3.5-turbo";
          let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
          let response = call_openai_api(&openai_api_key, model, content).await;

          let response_text = if let Some(reply) = response {
              reply
          } else {
              "Failed to get a response from OpenAI API.".to_string()
          };

          let _ = command
              .create_interaction_response(&ctx.http, |response| {
                  response
                      .kind(InteractionResponseType::ChannelMessageWithSource)
                      .interaction_response_data(|message| message.content(response_text))
              })
              .await;
      }
  }

  async fn ready(&self, ctx: Context, ready: Ready) {
      println!("{} is connected!", ready.user.name);

      let commands = Command::set_global_application_commands(&ctx.http, |commands| {
          commands.create_application_command(|command| {
              command
                  .name("chat")
                  .description("Chat with the AI model")
                  .create_option(|option| {
                      option
                          .name("text")
                          .description("The text you want to send")
                          .kind(CommandOptionType::String)
                          .required(true)
                  })
          })
      })
      .await;

      // println!("Slash commands set: {:?}", commands);
  }
}

// Call the OpenAI API with the specified API key, model, and input
async fn call_openai_api(api_key: &str, model: &str, input: &str) -> Option<String> {
  let client = Client::new();
  let url = format!("https://api.openai.com/v1/engines/{}/completions", model);
  let prompt = format!("User: {}\nAssistant:", input);

  // Change the model endpoint to "davinci-codex"
  let model_endpoint = if model == "gpt-3.5-turbo" {
      "davinci-codex"
  } else {
      model
  };

  let url = format!("https://api.openai.com/v1/engines/{}/completions", model_endpoint);

  // Send a POST request to the OpenAI API
  let response = client
      .post(&url)
      .header("Content-Type", "application/json")
      .header("Authorization", format!("Bearer {}", api_key))
      .json(&json!({
          "prompt": prompt,
          "max_tokens": 50,
          "n": 1,
          "stop": ["\n"]
      }))
      .send()
      .await;

  match response {
      Ok(res) => {
          // If the API request is successful, parse the JSON response
          if res.status().is_success() {
              let res_json: Value = res.json().await.unwrap();
              res_json["choices"][0]["text"].as_str().map(str::to_string)
          } else {
              eprintln!("OpenAI API error: {:?}", res);
              None
          }
      }
      Err(err) => {
          eprintln!("Failed to send request to OpenAI API: {:?}", err);
          None
      }
  }
}



#[tokio::main]
async fn main() {
  dotenv().ok();
  let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
  let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

  let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

  if let Err(why) = client.start().await {
      println!("Client error: {:?}", why);
  }
}
