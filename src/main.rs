use std::io;
use std::io::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

macro_rules! update_timer {
    ( $receiver:expr ) => {
        *$receiver
            .try_iter()
            .collect::<Vec<u32>>()
            .last()
            .expect("Called last with None Value")
    };
}

#[derive(Clone, Debug)]
struct Job {
    priority: usize,
    id: String,
    progress: usize,
}
impl Job {
    fn init(p: usize, id: String) -> Job {
        Job {
            priority: p,
            id,
            progress: 0,
        }
    }
    fn reset(mut self) {
        self.progress = 0;
    }
    fn progress(mut self) {
        self.progress += 1;
    }
}

fn main() {
    // Job Array
    let mut ts: Vec<Job> = vec![];
    ts.push(Job::init(3, String::from("t1")));
    ts.push(Job::init(2, String::from("t2")));
    ts.push(Job::init(1, String::from("t3")));
    // println!("{:?}", ts);

    // TODO File I/O parsing jobs

    // Timer stuff
    let (tx, rx) = mpsc::channel(); // create a new transmiter (tx) and receiver (rx)
    let _counter = thread::spawn(move || {
        let mut x: u32 = 0;
        loop {
            x += 1; // increment our counter
            tx.send(x.clone()).unwrap(); // send the data off to the main thread
            thread::sleep(Duration::from_secs(1)); // sleep to hand off execution just in case
        }
    });
    thread::sleep(Duration::from_nanos(1000)); // wait a sec for the counter thread to spawn some values
    let mut most_recent_time = update_timer!(rx);

    // TODO Run the jobs n' stuff
}
