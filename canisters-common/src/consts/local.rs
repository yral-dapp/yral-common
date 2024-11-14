use std::sync::LazyLock;

use url::Url;

pub static METADATA_API_BASE: LazyLock<Url> =
    LazyLock::new(|| Url::parse("http://localhost:8001").unwrap());

pub const AGENT_URL: &str = "http://localhost:4943";
