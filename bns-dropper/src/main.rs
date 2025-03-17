const ARCH: &'static str = std::env::consts::ARCH;

const REPO: &'static str = "https://github.com/SenneW-2158088/BNS-Botnet";

fn drop() {}

fn main() {
    // Get architecture
    println!("Architecture: {}", ARCH);

    // You can also get the OS
    let os = std::env::consts::OS;
    println!("Operating System: {}", os);

    // And the family (unix, windows, etc.)
    let family = std::env::consts::FAMILY;
    println!("OS Family: {}", family);
}
