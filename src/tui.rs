use std::sync::Arc;

use chorus::{
    instance::UserMeta,
    types::{MessageSendSchema, Snowflake},
};
use tokio::sync::Mutex;

use crate::e2e_data::{
    E2EMessage, MessageEncryptor, ENCRYPTED_MESSAGE, HELLO_ACCEPT, HELLO_REQUEST, PUBKEY_SHARE,
    PUBKEY_SHARE_REQUEST,
};

/// Lisens for send requests
pub async fn listen_for_user_input(
    usermeta: &mut UserMeta,
    keys: Arc<Mutex<MessageEncryptor>>,
    channel_id: Snowflake,
) {
    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let split = input.split_ascii_whitespace().collect::<Vec<&str>>();

    for i in 0..split.len() {
        if i == 0 {
            let arg = split[0].to_lowercase();
            match arg.as_str() {
                "init" | "start" | "initialize" => {
                    let mut message_to_send = MessageSendSchema::default();
                    message_to_send.content = Some(
                        E2EMessage {
                            opcode: HELLO_REQUEST,
                            data: None,
                        }
                        .encode(),
                    );
                    message_to_send.tts = Some(false);
                    usermeta
                        .send_message(message_to_send, channel_id.into())
                        .await
                        .unwrap();

                    println!("Requesting handshake..");
                }
                "accept" | "ok" => {
                    let mut message_to_send = MessageSendSchema::default();
                    message_to_send.content = Some(
                        E2EMessage {
                            opcode: HELLO_ACCEPT,
                            data: None,
                        }
                        .encode(),
                    );
                    usermeta
                        .send_message(message_to_send, channel_id.into())
                        .await
                        .unwrap();
                    println!("Accepting handshake..");
                }
                "send_pubkey" | "share" => {
                    let mut message_to_send = MessageSendSchema::default();
                    message_to_send.content = Some(
                        E2EMessage {
                            opcode: PUBKEY_SHARE,
                            data: Some(MessageEncryptor::encode_pubkey(
                                keys.lock().await.self_pubkey.clone(),
                            )),
                        }
                        .encode(),
                    );
                    usermeta
                        .send_message(message_to_send, channel_id.into())
                        .await
                        .unwrap();
                    println!("Sharing pubkey..");
                }
                "request_pubkey" | "request_share" => {
                    let mut message_to_send = MessageSendSchema::default();
                    message_to_send.content = Some(
                        E2EMessage {
                            opcode: PUBKEY_SHARE_REQUEST,
                            data: None,
                        }
                        .encode(),
                    );
                    usermeta
                        .send_message(message_to_send, channel_id.into())
                        .await
                        .unwrap();
                    println!("Sent pubkey share request..");
                }
                "message" | "send" => {
                    let message_content = inquire::Text::new(">").prompt().unwrap();

                    let encrypted = keys
                        .lock()
                        .await
                        .encrypt_message_for_peer(message_content.as_bytes().to_vec());

                    let mut message_to_send = MessageSendSchema::default();
                    message_to_send.content = Some(
                        E2EMessage {
                            opcode: ENCRYPTED_MESSAGE,
                            data: Some(encrypted),
                        }
                        .encode(),
                    );
                    usermeta
                        .send_message(message_to_send, channel_id.into())
                        .await
                        .unwrap();
                    println!("you: {} (🔒)", message_content);
                }
                _ => {
                    println!("Invalid keyword");
                }
            }
        }
    }
}
