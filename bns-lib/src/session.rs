use std::time::Duration;

use nostr_sdk::prelude::*;

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
        let subscription = Filter::new().author(pubkey);
        // .since(Timestamp::now());

        let output = self.client.subscribe(subscription.clone(), None).await?;
        println!("Subscription ID: {}", output.val);

        let stream = self
            .client
            .stream_events(subscription, Duration::MAX)
            .await?;
        Ok(stream)
    }

    pub async fn update_about(&self, about: &str) -> Result<()> {
        let metadata = Metadata::new().about(about);
        self.client.set_metadata(&metadata).await?;
        Ok(())
    }

    pub async fn send_msg(&self, content: &str, pubkey: PublicKey) -> Result<()> {
        let nip = nip04::encrypt(self.keys.secret_key(), &pubkey, content.as_bytes()).unwrap();
        let tag = Tag::public_key(pubkey);
        let event = EventBuilder::new(Kind::EncryptedDirectMessage, nip).tag(tag);

        println!("sending event...");
        self.client.send_event_builder(event).await?;
        Ok(())
    }
}
