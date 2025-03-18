use std::{sync::mpsc::Receiver, time::Duration};

use nostr_sdk::prelude::*;

use crate::CNC_PRIVATE_KEY;

pub struct SessionProps {
    pub name: String,
    pub display_name: String,
    pub private_key: Option<String>,
    pub relays: Vec<String>,
}

pub struct Session {
    keys: Keys,
    client: Client,
}

impl Session {
    pub async fn create(props: SessionProps) -> Result<Self> {
        let keys: Keys = match props.private_key {
            Some(key) => Keys::parse(key.as_str()).unwrap(),
            None => Keys::generate(),
        };
        println!("PRIVATE KEY:{:?}", keys.secret_key().to_bech32());
        println!("PUBLIC KEY: {:?}", keys.public_key().to_bech32());
        let connection: Connection = Connection::new();
        let opts = Options::new().connection(connection);

        let client = Client::builder().signer(keys.clone()).opts(opts).build();

        for relay in props.relays {
            client.add_relay(relay.as_str()).await?;
            client.connect_relay(relay.as_str()).await?;
            println!("Connected to relay: {}", relay);
        }

        let metadata = Metadata::new()
            .name(props.name.as_str())
            .display_name(props.display_name.as_str())
            .about("")
            .nip05("username@example.com");

        client.set_metadata(&metadata).await?;

        Ok(Self { keys, client })
    }

    pub async fn subscribe(&self, pubkey: PublicKey) -> Result<ReceiverStream<Event>> {
        let subscription = Filter::new()
            .kind(Kind::EncryptedDirectMessage)
            .author(pubkey);
        // .since(Timestamp::now());

        let output = self.client.subscribe(subscription.clone(), None).await?;
        println!("Subscription ID: {}", output.val);

        let stream = self
            .client
            .pool()
            .stream_events(
                subscription,
                Duration::MAX,
                ReqExitPolicy::WaitDurationAfterEOSE(Duration::MAX),
            )
            .await?;
        Ok(stream)
    }

    pub async fn update_about(&self, about: &str) -> Result<()> {
        let metadata = Metadata::new().about(about);
        self.client.set_metadata(&metadata).await?;
        Ok(())
    }

    pub async fn send_msg(&self, content: &str, pubkey: PublicKey) -> Result<()> {
        let encrypted =
            nip04::encrypt(self.keys.secret_key(), &pubkey, content.as_bytes()).unwrap();
        let tag = Tag::public_key(pubkey);
        let event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted.clone()).tag(tag);

        println!("sending event: {:?}", encrypted);
        self.client.send_event_builder(event).await?;
        Ok(())
    }

    pub async fn receive_msgs(
        &self,
        pubkey: PublicKey,
    ) -> Result<
        nostr_sdk::async_utility::futures_util::stream::Map<
            ReceiverStream<Event>,
            impl FnMut(Event) -> String,
        >,
    > {
        let stream = self.subscribe(pubkey).await?;
        let decrypted_stream = stream.map(move |event| {
            println!("received event: {:?}", event.content);

            let decrypted = nip04::decrypt_to_bytes(
                &self.keys.secret_key(),
                &event.pubkey,
                event.content.clone(),
            );
            if let Ok(decrypted) = decrypted {
                println!("DECRYPT OK");
                String::from_utf8(decrypted).unwrap_or_else(|_| {
                    println!("DECRYPT FAILED");
                    event.content.clone()
                })
            } else {
                if let Err(e) = decrypted {
                    println!("Error: {:?}", e);
                }
                event.content
            }
        });
        Ok(decrypted_stream)
    }
}
