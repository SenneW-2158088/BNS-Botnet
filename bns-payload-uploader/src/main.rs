use bns_lib::FILE_STORAGE_SERVER;
use nostr_sdk::Client;
use nostr_sdk::prelude::*;
use std::env;
use std::fs::File;
use std::io::Read;

const RELAYS: [&str; 1] = [bns_lib::RELAY];

async fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            std::process::exit(1);
        }
    };

    let mut contents = Vec::new();
    if let Err(e) = file.read_to_end(&mut contents) {
        eprintln!("Failed to read file: {}", e);
        std::process::exit(1);
    }

    let keys: Keys = Keys::parse(bns_lib::CNC_PRIVATE_KEY)?;

    let connection: Connection = Connection::new();
    let opts = Options::new().connection(connection);

    let client = Client::builder().signer(keys.clone()).opts(opts).build();

    // uploading file
    println!("[+] server_config");

    let server_config = nip96::get_server_config(Url::parse(bns_lib::FILE_STORAGE_SERVER)?, None).await?;

    println!("[+] allowed mimetypes: {:?}", server_config.content_types);
    
    println!("[+] uploading data...");

    let url = nip96::upload_data(&client.signer().await?, &server_config, contents, Some("text/plain"), None).await?;

    println!("url: {}", url);
    
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
