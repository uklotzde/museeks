use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use aoide::backend_embedded::storage::DatabaseConfig;

const POOL_SIZE: u32 = 8;

const POOL_ACQUIRE_READ_TIMEOUT_MILLIS: u64 = 5_000;

const POOL_ACQUIRE_WRITE_TIMEOUT_MILLIS: u64 = 10_000;

const FILE_NAME: &str = "aoide.sqlite";

pub(super) fn default_config(dir_path: &Path) -> DatabaseConfig {
    DatabaseConfig {
        connection: aoide::storage_sqlite::connection::Config {
            storage: aoide::storage_sqlite::connection::Storage::File {
                path: dir_path.join(FILE_NAME),
            },
            pool: aoide::storage_sqlite::connection::pool::Config {
                max_size: NonZeroU32::MIN.saturating_add(POOL_SIZE - NonZeroU32::MIN.get()),
                gatekeeper: aoide::storage_sqlite::connection::pool::gatekeeper::Config {
                    acquire_read_timeout_millis: NonZeroU64::MIN
                        .saturating_add(POOL_ACQUIRE_READ_TIMEOUT_MILLIS - NonZeroU64::MIN.get()),
                    acquire_write_timeout_millis: NonZeroU64::MIN
                        .saturating_add(POOL_ACQUIRE_WRITE_TIMEOUT_MILLIS - NonZeroU64::MIN.get()),
                },
            },
        },
        migrate_schema: None,
    }
}
