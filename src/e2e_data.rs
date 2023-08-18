//! Contains main spec for e2e
// Each message starts with ッ to filter out messages
// Each message shall be encoded with bincode

use bincode;
use rand;
use rsa::{
    self,
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use z85;

/// First message sent, sent basically to ask if we can encrypt in this channel
pub const HELLO_REQUEST: u8 = 1;
/// Reply to [HELLO_REQUEST], ackowledges this protocol and accepts attempted encryption
pub const HELLO_ACCEPT: u8 = 2;
/// Sent to share a public key with the other peer.
pub const PUBKEY_SHARE: u8 = 3;
/// Requests the other peer to send its pubkey
pub const PUBKEY_SHARE_REQUEST: u8 = 4;
/// Sent after both clients should have each other's keys, contains encrypted content
pub const ENCRYPTED_MESSAGE: u8 = 5;

/// Character used to identify encoded messages
pub const CONTROL_CHARACTER: char = '⌬';

pub const ENCRYPTION_BITS: usize = 4096;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct E2EMessage {
    /// Operation code, what this message means
    pub opcode: u8,
    /// Optional data, such as encrypted message content or our public key
    pub data: Option<Vec<u8>>,
}

impl E2EMessage {
    /// Converts self into an encoded message.
    pub fn encode(&self) -> String {
        let as_bytes = bincode::serialize(self).unwrap();
        let mut as_string = z85::encode(as_bytes);
        as_string.insert(0, CONTROL_CHARACTER);
        as_string
    }

    /// Decodes self from an encoded message.
    pub fn decode(item: String) -> Result<Self, String> {
        if !Self::is_encoded(item.clone()) {
            return Err("Invalid start sequence!".to_string());
        }

        let mut string = item.clone();
        string.remove(0);

        let as_bytes = z85::decode(string);

        if let Err(e) = as_bytes {
            return Err(e.to_string());
        }

        let decode_result: Result<E2EMessage, _> = bincode::deserialize(&as_bytes.unwrap());

        if decode_result.is_ok() {
            return Ok(decode_result.unwrap());
        } else {
            return Err(decode_result.err().unwrap().to_string());
        }
    }

    /// Checks if a message could be an encoded message.
    /// Checks if the first character is the right one.
    pub fn is_encoded(item: String) -> bool {
        item.starts_with(CONTROL_CHARACTER)
    }
}

/// Encrypts messages and generates keys.
#[derive(Clone, Debug)]
pub struct MessageEncryptor {
    /// Our own privkey
    pub privkey: RsaPrivateKey,
    /// Our own pubkey
    pub self_pubkey: RsaPublicKey,
    /// Our peer's pubkey
    pub peer_pubkey: Option<RsaPublicKey>,
}

impl MessageEncryptor {
    /// Generates a new keypair for Self and sets it
    pub fn generate_keypair(&mut self) {
        let mut rng = rand::thread_rng();
        let privkey =
            RsaPrivateKey::new(&mut rng, ENCRYPTION_BITS).expect("Failed to generate key");
        let pubkey = RsaPublicKey::from(&privkey);
        self.privkey = privkey;
        self.self_pubkey = pubkey;
    }

    /// Creates a new [MessageEncryptor], generating a random pubkey and privkey
    pub fn new_generated() -> Self {
        let mut rng = rand::thread_rng();
        let privkey =
            RsaPrivateKey::new(&mut rng, ENCRYPTION_BITS).expect("Failed to generate key");
        let pubkey = RsaPublicKey::from(&privkey);

        MessageEncryptor {
            privkey,
            self_pubkey: pubkey,
            peer_pubkey: None,
        }
    }

    /// Encodes a pubkey.
    /// Inverse of [decode_pubkey].
    pub fn encode_pubkey(key: RsaPublicKey) -> Vec<u8> {
        key.to_pkcs1_der().unwrap().to_vec()
    }

    /// Decodes a pubkey.
    /// Inverse of [encode_pubkey].
    pub fn decode_pubkey(key: Vec<u8>) -> Result<RsaPublicKey, rsa::pkcs1::Error> {
        RsaPublicKey::from_pkcs1_der(&key)
    }

    /// Encrypts a message for a peer, (using our peer pubkey) returning an array of encrypted bytes
    pub fn encrypt_message_for_peer(&self, message: Vec<u8>) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        self.peer_pubkey
            .as_ref()
            .unwrap()
            .encrypt(&mut rng, Pkcs1v15Encrypt, &message)
            .unwrap()
    }

    /// Decrypts a message from our peer, (using our privkey) returning an array of decrypted bytes
    pub fn decrypt_message_from_peer(&self, message: Vec<u8>) -> Result<Vec<u8>, rsa::Error> {
        self.privkey.decrypt(Pkcs1v15Encrypt, &message)
    }
}

#[test]
/// Tests encoding and decoding [E2EMessage]s.
fn encode_decode_test() {
    let test_message = E2EMessage {
        opcode: HELLO_REQUEST,
        data: Some("Testing stuff".to_string().into_bytes()),
    };

    let encoded = test_message.encode();

    println!(r#"Encoded: "{}" ({:?})"#, encoded, encoded);

    let decoded = E2EMessage::decode(encoded).unwrap();

    assert_eq!(test_message, decoded);
}
