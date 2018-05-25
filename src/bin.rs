extern crate spectrophoner;

use std::time::Duration;
use std::thread;

use spectrophoner::conductor;

pub fn main() {
    conductor::conduct();

    thread::sleep(Duration::from_millis(1000_000));
}
