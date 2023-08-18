# chorus-e2e-test
A tech demo for dm end to end encryption with Chorus

https://github.com/kozabrada123/chorus-e2e-test/assets/59031733/935f239c-a933-4a22-a9f6-e34dc367b5d4

## Disclaimer

This is mostly a tech demo, use at your own risk.

Also, I cannot guarantee cryptographic security

## Usage

You'll need a token on any spacebar compatible server and the dm channel id.

Afterwards clone the project and run it with `cargo run --release`.

You'll be prompted for all needed info.

After entering all your details, you can use the following keywords to send messages:

| keyword       | message                         |
|---------------|---------------------------------|
| start         | Handshake init                  |
| accept        | Handshake accept                |
| share         | Share pubkey                    |
| request_share | Request peer share their pubkey |
| message       | Send an encrypted message       |

You must exchange pubkeys before you can encrypt messages.
