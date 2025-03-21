pub const MAX_WATCH_HISTORY_CACHE_LEN: u64 = 5000;
pub const MAX_SUCCESS_HISTORY_CACHE_LEN: u64 = 10000;
pub const MAX_GLOBAL_CACHE_LEN: u64 = 3000;
pub const MAX_USER_CACHE_LEN: u64 = 1000;

pub const GLOBAL_CACHE_CLEAN_KEY: &str = "global_cache_clean";
pub const GLOBAL_CACHE_NSFW_KEY: &str = "global_cache_nsfw";
pub const GLOBAL_CACHE_MIXED_KEY: &str = "global_cache_mixed";

pub const USER_WATCH_HISTORY_CLEAN_SUFFIX: &str = "_watch_clean";
pub const USER_SUCCESS_HISTORY_CLEAN_SUFFIX: &str = "_success_clean";
pub const USER_WATCH_HISTORY_NSFW_SUFFIX: &str = "_watch_nsfw";
pub const USER_SUCCESS_HISTORY_NSFW_SUFFIX: &str = "_success_nsfw";

pub const USER_CACHE_CLEAN_SUFFIX: &str = "_cache_clean";
pub const USER_CACHE_NSFW_SUFFIX: &str = "_cache_nsfw";
pub const USER_CACHE_MIXED_SUFFIX: &str = "_cache_mixed";
