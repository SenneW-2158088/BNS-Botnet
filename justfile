default:
  just --list

payload_dir := "./payload"
build_dir := "./target"

linux-x86 := "x86_64-unknown-linux-gnu"
aarch64-darwin := "aarch64-apple-darwin"

# Ensure the payloads directory exists
_ensure-dir:
    mkdir -p {{payload_dir}}

# Build for Linux x86_64
linux-x86: _ensure-dir
    @echo "Building for Linux x86_64..."
    cargo build --bin bns-client --release --target {{linux-x86}}
    cp target/{{linux-x86}}/release/bns-client {{payload_dir}}/client-{{linux-x86}}

# Build for Macos ARM64
aarch64-darwin: _ensure-dir
    @echo "Building for macOS ARM64..."
    cargo build --bin bns-client --release --target {{aarch64-darwin}}
    cp target/{{aarch64-darwin}}/release/bns-client {{payload_dir}}/client-{{aarch64-darwin}}
