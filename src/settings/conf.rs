

#[derive(Debug, Clone)]
pub enum AppMode {
    Server,
    Client
}

#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub is_debug: bool,
    pub mode: AppMode,
    pub listen_address: String,
    pub remote_address: Option<String>,
    pub is_encrypt_channel: bool,
}
