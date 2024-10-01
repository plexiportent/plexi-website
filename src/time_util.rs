use std::time::{SystemTime, UNIX_EPOCH};


pub fn to_unix_timestamp(tm: SystemTime) -> i64 {
    if tm < UNIX_EPOCH {
        - (UNIX_EPOCH.duration_since(tm).unwrap().as_secs() as i64)
    } else {
        tm.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }
}

pub fn current_unix_timestamp() -> i64 {
    to_unix_timestamp(SystemTime::now())
}