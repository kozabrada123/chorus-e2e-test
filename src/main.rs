use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use bincode;
use chorus::{
    self,
    gateway::Gateway,
    instance::UserMeta,
    types::{GatewayIdentifyPayload, LoginSchema, Message, Snowflake, User, UserSettings},
    UrlBundle,
};
use e2e_data::{
    E2EMessage, MessageEncryptor, ENCRYPTED_MESSAGE, HELLO_ACCEPT, HELLO_REQUEST, PUBKEY_SHARE,
    PUBKEY_SHARE_REQUEST,
};
use inquire;
use observers::NewMessageObserver;
use simplelog::{Config, Level};
use tui::listen_for_user_input;

mod e2e_data;
mod observers;
mod tui;

#[tokio::main]
async fn main() {
    /*simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();*/

    let api_url = inquire::Text::new("Api url: ")
        .with_default("https://discord.com/api/v9")
        .prompt()
        .unwrap();
    let gw_url = inquire::Text::new("Gateway url: ")
        .with_default("wss://gateway.discord.gg")
        .prompt()
        .unwrap();
    let cdn_url = inquire::Text::new("Cdn url: ")
        .with_default("https://cdn.discord.com")
        .prompt()
        .unwrap();

    let token = inquire::Password::new("Token: ").prompt().unwrap();
    let channel_id: Snowflake = inquire::CustomType::<u64>::new("Channel Id: ")
        .prompt()
        .unwrap()
        .into();

    let urls = UrlBundle::new(api_url, gw_url, cdn_url);
    let instance = Rc::new(RefCell::new(
        chorus::instance::Instance::new(urls.clone(), false)
            .await
            .unwrap(),
    ));

    let mut identify = GatewayIdentifyPayload::common();
    let gateway = Gateway::new(urls.wss.clone()).await.unwrap();
    identify.token = token.clone();
    gateway.send_identify(identify).await;

    let mut user = UserMeta::new(
        instance,
        token.clone(),
        None,
        Arc::new(RwLock::new(UserSettings::default())),
        Arc::new(RwLock::new(User::default())),
        gateway,
    );

    user.object = Arc::new(RwLock::new(User::get(&mut user, None).await.unwrap()));

    let mut keys = Arc::new(tokio::sync::Mutex::new(MessageEncryptor::new_generated()));

    let message_observer = Arc::new(NewMessageObserver {
        encryptor: keys.clone(),
        self_id: user.object.read().unwrap().id.clone(),
    });

    user.gateway
        .events
        .lock()
        .await
        .message
        .create
        .subscribe(message_observer);

    println!(
        "{}",
        E2EMessage {
            opcode: HELLO_REQUEST,
            data: None
        }
        .encode()
    );

    loop {
        listen_for_user_input(&mut user, keys.clone(), channel_id.clone()).await;
    }
}
