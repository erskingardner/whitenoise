use derive_more::From;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Keyring(keyring::Error),
    #[from]
    Sled(sled::Error),
    #[from]
    JsonSerde(serde_json::Error),
    #[from]
    Utf8(std::str::Utf8Error),
    #[from]
    TauriInvoke(tauri::ipc::InvokeError),
    #[from]
    NostrClient(nostr_sdk::client::Error),
    #[from]
    NostrKeys(nostr_sdk::key::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), std::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
