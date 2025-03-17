# Botnet

## Dropper

The dropper is a simple program that will download the payload from our github repo.
It checks the current architecture of the OS and will try to download from REPO/payloads/{payload}-{architecture}.
The dropper moves the payload to a temporary file and will spawn a new process trying to execute the payload.

## C2C

TODO

## Client

The client is the payload executed and will connect to a set of nostr relays. The client filters for C2C commands and executes these.

**TODO**

- Commands
- 

## Lib

Lib contains all the other mechanisms we will be using to obscure payloads and serialization of custom commands.
