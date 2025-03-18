use std::{sync::mpsc::Receiver, time::Duration};

use nostr_sdk::prelude::*;

use crate::{CNC_PRIVATE_KEY, CNC_PUB_KEY};

pub struct SessionProps {
    pub name: String,
    pub display_name: String,
    pub private_key: Option<String>,
    pub relays: Vec<String>,
}

pub struct Session {
    keys: Keys,
    client: Client,
    name: String,
    display_name: String,
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
            .about("I'm enabled");

        client.set_metadata(&metadata).await?;

        Ok(Self {
            keys,
            client,
            display_name: props.display_name,
            name: props.name,
        })
    }

    pub async fn subscribe(&self, pubkey: PublicKey) -> Result<ReceiverStream<Event>> {
        let subscription = Filter::new()
            .kind(Kind::EncryptedDirectMessage)
            .author(pubkey)
            .pubkey(self.keys.public_key());

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
        // let metadata = self
        //     .client
        //     .fetch_metadata(self.keys.public_key, Duration::from_secs(10))
        //     .await
        //     .unwrap();
        let metadata = Metadata::new()
            .name(self.name.as_str())
            .display_name(self.display_name.as_str())
            .about(about);
        self.client.set_metadata(&metadata).await?;
        Ok(())
    }

    pub async fn send_msg(&self, content: &str, pubkey: PublicKey) -> Result<()> {
        let encrypted =
            nip04::encrypt(self.keys.secret_key(), &pubkey, content.as_bytes()).unwrap();
        let tag = Tag::public_key(pubkey);
        let event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted).tag(tag);

        // let decrypted = nip04::decrypt(
        //     &SecretKey::parse(CNC_PRIVATE_KEY).unwrap(),
        //     &self.keys.public_key(),
        //     encrypted.clone(),
        // );

        // println!("sending event: {:?}", encrypted);
        // println!("sending event: {:?}", decrypted);
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
            let decrypted = nip04::decrypt(&self.keys.secret_key(), &pubkey, event.content.clone());
            if let Ok(decrypted) = decrypted {
                decrypted
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
