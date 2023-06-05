use std::time::Duration;

use log::{error, info};

use crate::util::variables::{CLAMD_HOST, USE_CLAMD};

pub fn init() {
    if *USE_CLAMD {
        info!("Waiting for clamd to be ready...");

        loop {
            let clamd_available = match revolt_clamav_client::ping_tcp(CLAMD_HOST.to_string()) {
                Ok(ping_response) => ping_response == b"PONG\0",
                Err(_) => false,
            };

            if clamd_available {
                info!("clamd is ready, virus protection enabled!");
                break;
            } else {
                error!(
                    "Could not ping clamd host at {}, retrying in 10 seconds...",
                    CLAMD_HOST.to_string()
                );

                std::thread::sleep(Duration::from_secs(10));
            }
        }
    }
}
