# Botnet

## Usage

Log in with the C&C private key to any nostr client that supports nip04 messaging:

- private key: `nsec1hynhg786hrmcd8r3m5aphjk79gyjzslw0522ymjup573kyjtfnvss0nrza`

Now you can send commands to the botnet.

Possible clients:
- https://www.nostrchat.io
- https://primal.net/

## Dropper

The dropper is a simple program that will download the payload from our github repo.
It checks the current architecture of the OS and will try to download from REPO/payloads/{payload}-{architecture}.
The dropper moves the payload to a temporary file and will spawn a new process trying to execute the payload.

## Client

The client is the payload executed and will connect to a set of nostr relays. The client filters for C2C commands and executes these.

**TODO**

- Commands
-

## Lib

Lib contains all the other mechanisms we will be using to obscure payloads and serialization of custom commands.
