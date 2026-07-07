use std::mem;

use crate::sys_service::time_sync::get_synced_system_time;
use libc::{clock_gettime, timespec, CLOCK_BOOTTIME, CLOCK_MONOTONIC};
use tokio::time::{Duration, Instant};

pub struct LdCountdown {
    start: Instant,
    duration: Duration,
}

impl LdCountdown {
    pub fn new(duration: Duration) -> Self {
        Self { start: Instant::now(), duration }
    }

    pub fn remaining(&self) -> Duration {
        let elapsed = self.start.elapsed();
        if elapsed >= self.duration {
            Duration::from_secs(0)
        } else {
            self.duration - elapsed
        }
    }
}

pub fn get_boot_time_ns() -> Result<u64, i32> {
    let mut ts: timespec = unsafe { mem::zeroed() };

    let result = unsafe { clock_gettime(CLOCK_BOOTTIME, &mut ts) };

    if result == 0 {
        let ns = (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64);
        Ok(ns)
    } else {
        Err(unsafe { *libc::__errno_location() })
    }
}

pub fn get_current_time_ns() -> Result<u64, i32> {
    let time =
        get_synced_system_time().duration_since(std::time::UNIX_EPOCH).map_err(|_| libc::EINVAL)?;
    Ok(time.as_nanos() as u64)
}

pub fn get_current_time_ms() -> Result<u64, i32> {
    Ok(get_current_time_ns()? / 1_000_000)
}

pub fn get_relative_time_ns() -> Result<u64, i32> {
    let current_time_ns = get_current_time_ns()?;
    let mut monotonic: timespec = unsafe { std::mem::zeroed() };

    if unsafe { clock_gettime(CLOCK_MONOTONIC, &mut monotonic) } != 0 {
        return Err(unsafe { *libc::__errno_location() });
    }

    let monotonic_ns = (monotonic.tv_sec as u64) * 1_000_000_000 + (monotonic.tv_nsec as u64);
    current_time_ns.checked_sub(monotonic_ns).ok_or(libc::EINVAL)
}

pub const MILL_A_DAY: u32 = 1000 * 60 * 60 * 24;

pub fn get_f64_timestamp() -> f64 {
    const MILLIS_PER_SEC: u64 = 1_000;
    let time = get_synced_system_time()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time is before UNIX_EPOCH");

    (time.as_secs() as f64) * (MILLIS_PER_SEC as f64) + (time.subsec_millis() as f64)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::get_boot_time_ns;

    #[test]
    pub fn test() {
        let now = Instant::now();
        match get_boot_time_ns() {
            Ok(ns) => println!("system boot time (ns): {}", ns),
            Err(e) => eprintln!("failed to get boot time, error code: {}", e),
        }
        println!("{}", now.elapsed().as_nanos())
    }
}
