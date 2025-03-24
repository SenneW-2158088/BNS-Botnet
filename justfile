default:
  just --list

payload_dir := "./payload"
build_dir := "./target"

linux-x86 := "x86_64-unknown-linux-gnu"
aarch64-darwin := "aarch64-apple-darwin"
linux-x86-name := "x86_64"
aarch64-darwin-name := "aarch64"

# Ensure the payloads directory exists
_ensure-dir:
    mkdir -p {{payload_dir}}

# Build for Linux x86_64
linux-x86: _ensure-dir
    @echo "Building for Linux x86_64..."
    cargo build --bin bns-client --release --target {{linux-x86}}
    cp target/{{linux-x86}}/release/bns-client {{payload_dir}}/client-{{linux-x86-name}}
    cargo build --bin bns-payload --release --target {{linux-x86}}
    cp target/{{linux-x86}}/release/bns-payload {{payload_dir}}/payload-{{linux-x86-name}}

# Build for Macos ARM64
aarch64-darwin: _ensure-dir
    @echo "Building for macOS ARM64..."
    cargo build --bin bns-client --release --target {{aarch64-darwin}}
    cp target/{{aarch64-darwin}}/release/bns-client {{payload_dir}}/client-{{aarch64-darwin-name}}
    cargo build --bin bns-payload --release --target {{aarch64-darwin}}
    cp target/{{aarch64-darwin}}/release/bns-payload {{payload_dir}}/payload-{{aarch64-darwin-name}}
