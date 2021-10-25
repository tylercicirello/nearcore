use std::default::Default;
use std::sync::{Arc, Mutex};
use std::time;

use arc_swap::ArcSwap;
use chrono;
use once_cell::sync::Lazy;

pub use chrono::Utc;
pub use std::time::Instant;

use std::collections::VecDeque;

pub struct MockTimeSingleton {
    utc: VecDeque<chrono::DateTime<chrono::Utc>>,
    instant: VecDeque<time::Instant>,
    utc_call_count: u64,
    instant_call_count: u64,
}

impl Default for MockTimeSingleton {
    fn default() -> Self {
        Self {
            utc: VecDeque::with_capacity(16),
            instant: VecDeque::with_capacity(16),
            utc_call_count: 0,
            instant_call_count: 0,
        }
    }
}

static SINGLETON: Lazy<ArcSwap<Mutex<MockTimeSingleton>>> =
    Lazy::new(|| ArcSwap::from_pointee(Mutex::new(MockTimeSingleton::new())));

impl MockTimeSingleton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get() -> Arc<Mutex<MockTimeSingleton>> {
        SINGLETON.load_full()
    }

    pub fn set(value: MockTimeSingleton) {
        SINGLETON.store(Arc::new(Mutex::new(value)))
    }

    pub fn add_utc(&mut self, mock_date: chrono::DateTime<chrono::Utc>) {
        self.utc.push_back(mock_date);
    }

    pub fn pop_utc(&mut self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.utc_call_count += 1;
        self.utc.pop_front()
    }

    pub fn pop_instant(&mut self) -> Option<time::Instant> {
        self.instant_call_count += 1;
        self.instant.pop_front()
    }

    pub fn reset(&mut self) {
        self.instant_call_count = 0;
        self.utc_call_count = 0;
        self.instant.clear();
        self.utc.clear();
    }

    pub fn add_instant(&mut self, mock_instant: time::Instant) {
        self.instant.push_back(mock_instant);
    }

    pub fn get_instant_call_count(&mut self) -> u64 {
        self.instant_call_count
    }

    pub fn get_utc_call_count(&mut self) -> u64 {
        self.utc_call_count
    }

    pub fn count_instant(self) -> usize {
        self.instant.len()
    }
}

pub trait MockTime {
    type Value;

    fn now_or_mock() -> Self::Value;

    fn system_time() -> Self::Value;
}

impl MockTime for Utc {
    type Value = chrono::DateTime<chrono::Utc>;

    fn now_or_mock() -> chrono::DateTime<chrono::Utc> {
        let time_singleton = MockTimeSingleton::get();
        let x = time_singleton.lock().unwrap().pop_utc();
        match x {
            Some(t) => t,
            None => chrono::Utc::now(),
        }
    }

    fn system_time() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

impl MockTime for Instant {
    type Value = time::Instant;

    fn now_or_mock() -> time::Instant {
        let time_singleton = MockTimeSingleton::get();
        let x = time_singleton.lock().unwrap().pop_instant();
        match x {
            Some(t) => t,
            None => time::Instant::now(),
        }
    }

    fn system_time() -> time::Instant {
        time::Instant::now()
    }
}
