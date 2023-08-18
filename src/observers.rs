//! Defines chorus observers

use std::sync::Arc;

use async_trait::async_trait;
use chorus::{
    gateway::Observer,
    types::{MessageCreate, Snowflake},
};
use tokio::sync::Mutex;

use crate::e2e_data::{
    E2EMessage, MessageEncryptor, ENCRYPTED_MESSAGE, HELLO_ACCEPT, HELLO_REQUEST, PUBKEY_SHARE,
    PUBKEY_SHARE_REQUEST,
};

/// Observes when new messages are sent
#[derive(Debug, Clone)]
pub struct NewMessageObserver {
    pub self_id: Snowflake,
    pub encryptor: Arc<Mutex<MessageEncryptor>>,
}

#[async_trait]
impl Observer<MessageCreate> for NewMessageObserver {
    async fn update(&self, data: &MessageCreate) {
        if data.message.author.id == self.self_id {
            return;
        }

        let message_content = data.message.content.clone().unwrap();
        if let Ok(decoded) = E2EMessage::decode(message_content.clone()) {
            match decoded.opcode {
                HELLO_REQUEST => {
                    println!(
                        "{} requested encryption in channel {}",
                        data.message.author.username.clone().unwrap(),
                        data.message.channel_id
                    );
                }
                HELLO_ACCEPT => {
                    println!(
                        "{} accepted encryption in channel {}",
                        data.message.author.username.clone().unwrap(),
                        data.message.channel_id
                    );
                }
                PUBKEY_SHARE => {
                    println!(
                        "{} shared their pubkey in channel {}",
                        data.message.author.username.clone().unwrap(),
                        data.message.channel_id
                    );
                    self.encryptor.lock().await.peer_pubkey =
                        Some(MessageEncryptor::decode_pubkey(decoded.data.unwrap()).unwrap());
                }
                PUBKEY_SHARE_REQUEST => {
                    println!(
                        "{} requested pubkey in channel {}",
                        data.message.author.username.clone().unwrap(),
                        data.message.channel_id
                    );
                }
                ENCRYPTED_MESSAGE => {
                    let decryptor = self.encryptor.lock().await;
                    let decrypted = String::from_utf8(
                        decryptor
                            .decrypt_message_from_peer(decoded.data.unwrap())
                            .unwrap(),
                    )
                    .unwrap();
                    println!(
                        "{}: {:?} (🔒)",
                        data.message.author.username.clone().unwrap(),
                        decrypted
                    );
                }
                _ => {}
            }
        } else {
            println!(
                "{}-{}: {}",
                data.message.author.username.clone().unwrap(),
                data.message.channel_id.clone(),
                message_content.clone()
            );
        }
    }
}
