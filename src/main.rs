use std::fs::File;
use std::io;
use std::io::*;
use std::path::Path;
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
    arrival: usize,
}
impl Job {
    fn init(p: usize, id: String, arrival: usize) -> Job {
        Job {
            priority: p,
            id,
            progress: 0,
            arrival,
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
    let mut jobs: Vec<Job> = vec![];
    let mut ts: Vec<Job> = vec![];
    ts.push(Job::init(3, String::from("t1"), 0));
    ts.push(Job::init(2, String::from("t2"), 0));
    ts.push(Job::init(1, String::from("t3"), 0));
    // println!("{:?}", ts);

    // TODO File I/O parsing jobs
    let path = Path::new("input.txt");
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why_fail) => panic!("Could not open {}: {}", display, why_fail),
        Ok(file) => file,
    };
    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf);
    //println!("text:\n{}", buf);
    let toks: Vec<&str> = buf.split("\n").collect();
    for tok in &toks {
        let subtok: Vec<&str> = tok.split_ascii_whitespace().collect();
        println!("{:?}", subtok);
        let arg1 = subtok[0].parse::<usize>().unwrap();
        let arg2 = subtok[1].parse::<usize>().unwrap();
        let mut tmp: Job = ts[arg2 - 1].clone();
        tmp.arrival = arg1;
        jobs.push(tmp);
    }
    println!("{:?}", jobs);

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
