# **RustGPT-Discord**
RustGPT-Discord is a feature-rich bot built using the Rust programming language and the Serenity library. This bot provides various commands and functionalities, including AI chat integration and privacy settings for messages.

## **Prerequisites**
Before setting up the bot, ensure you have the following:

[Rust](https://www.rust-lang.org/tools/install) installed on your system.
A Discord account and [Discord Developer Portal](https://discord.com/developers/applications) access.
[OpenAI](https://beta.openai.com/signup/) account for API access.

## **Setup**
Follow the steps below to set up the bot:

### **1**. **Create a new Discord application**
Go to the [Discord Developer Portal](https://discord.com/developers/applications) and log in with your Discord account.
Click "New Application" and provide a name for your bot.

In the "General Information" tab of your Discord application settings, you'll find the "Application ID". Save this value as you will need it later.

Navigate to the "Bot" tab on the left sidebar, and click "Add Bot".

Copy the bot token by clicking "Copy" under the "TOKEN" section. You will need this token later.

Scroll down to the "Privileged Gateway Intents" section, and enable the following intents:

- `MESSAGE CONTENT INTENT`

This allows the bot to receive messages.

Click the "Save Changes" button.
### **2**. **Clone the repository and install dependencies**
Copy code:
```sh
git clone https://github.com/MasterHarun/RustGPT.git
cd RustGPT
```

### **3**. **Set up environment variables**
Create a .env file in the root directory of the project and add the following environment variables:

Copy code:
```sh
DISCORD_TOKEN=<your_discord_bot_token>
DISCORD_APPLICATION_ID=<your_discord_application_id>
OPENAI_API_KEY=<your_openAI_api_key>
```

Replace `<your_discord_bot_token>` with the bot token you copied earlier, and `<your_openAI_api_key>` with your OpenAI API key.

### **4**. **Build and run the bot**
In the root directory of the project, run the following commands:

```sh
cargo build
cargo run
```
The bot should now be running, and it will display a "Connected" message in the terminal.

### **5**. **Invite the bot to your Discord server**
Go to the [Discord Developer Portal](https://discord.com/developers/applications) and select your bot.
Navigate to the "OAuth2" tab on the left sidebar.
Under the "Scopes" section, select the "bot" checkbox.
Under the "Bot Permissions" section, select the desired permissions for your bot.
Copy the generated URL from the "Scopes" section.
Open the URL in a new browser tab, and follow the prompts to invite the bot to your server.
The bot should now appear in your server, and you can start using its commands and features.

## **Commands**
Here is a list of available commands:

**/chat**: Chat with the AI using OpenAI's GPT.

**/reset**: Reset the chat context with the AI.

**/private**: Set chat privacy mode to "Private," making the AI responses visible only to the command issuer.

**/public**: Set chat privacy mode to "Public," making the AI responses visible to all server members.

## **Features**
AI chat integration using OpenAI's GPT.
Chat privacy settings allowing for private or public AI responses.
Extensible command system for adding new commands easily.

## **Contributing**
Contributions are welcome! If you'd like to contribute, please follow these steps:

Fork the repository.
Create a new branch for your changes.
Make your changes and commit them with a descriptive commit message.
Push your changes to your fork.
Create a pull request.

## **License**
This project is licensed under the **MIT License**.