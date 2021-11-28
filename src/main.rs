use std::io;
use std::io::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::path::Path; 

macro_rules! update_timer {
    ( $receiver:expr ) => {
        *$receiver
            .try_iter()
            .collect::<Vec<usize>>()
            .last()
            .expect("Called last with None Value")
    };
}

#[derive(Clone, Debug)]
struct Job {
    priority: usize,
    id: String,
    progress: usize,
    total_required: usize,
    arrival: usize,
    active: bool,
}
impl Job {
    fn init(priority: usize, 
        id: String, 
        total_required: usize, 
        arrival: usize) -> Job {
        Job {
            priority,
            id,
            progress: 0,
            total_required,
            arrival,
            active: false
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
    let mut jobs: Vec<Job> = vec![];    // the job vector for handling input stream
    let mut ts: Vec<Job> = vec![];      // template jobs that we can clone from
    ts.push(Job::init(3, String::from("t1"), 3, 0)); // T1 - shared buffer
    ts.push(Job::init(2, String::from("t2"), 10, 0)); // T2
    ts.push(Job::init(1, String::from("t3"), 3, 0)); // T3 - shared buffer
    // println!("{:?}", ts);

    /* 
    This program expects that the job stream will come from a 
    file named input.txt where each job is on a new line and is in the form:
    [unsigned integer > 0] [0 < unsigned integer < 4]\n
    I have included a sample input.txt file in the project submission
    */

    let path = Path::new("input.txt");          // specify the path
    let display = path.display();               // for error reporting purposes
    let mut file = match File::open(&path) {    // attempt to open the file, if it fails tell us why
        Err(why_fail) => panic!("Could not open {}: {}", display, why_fail),
        Ok(file) => file, 
    };
    let mut buf = String::new();            // buffer for reading the lines of the file
    let _ = file.read_to_string(&mut buf);  // read them c-style
    //println!("text:\n{}", buf); 
    let toks: Vec<&str> = buf.split("\n").collect(); // split on the newline
    for tok in &toks {      // for each of those tokens
        let subtok: Vec<&str> = tok.split_ascii_whitespace().collect(); // split once again on all whitespace
        //println!("{:?}", subtok);     // for debugging
        let arg1 = subtok[0].parse::<usize>().unwrap(); // convert the strings into unsigned integers
        let arg2 = subtok[1].parse::<usize>().unwrap();
        let mut tmp: Job = ts[arg2-1].clone();          // create our new job
        tmp.arrival = arg1;                             // set its arrival time
        jobs.push(tmp);                                 // push the new job
    }
    jobs.sort_by_key(|j| j.arrival); // sort the jobs based on their arrival time (just in case if you didn't)
    //println!("{:?}", jobs); // debugging purposes

    // Timer stuff
    let (tx, rx) = mpsc::channel(); // create a new transmiter (tx) and receiver (rx)
    let _counter = thread::spawn(move || {
        let mut x: usize = 0;
        loop {
            x += 1; // increment our counter
            tx.send(x.clone()).unwrap(); // send the data off to the main thread
            thread::sleep(Duration::from_secs(1)); // sleep to hand off execution just in case
        }
    });
    thread::sleep(Duration::from_nanos(1000)); // wait a sec for the counter thread to spawn some values
    
    // Main program execution
    let mut most_recent_time: usize = 0;
    let mut active_job_queue: Vec<Job> = vec![];
    let mut prev_time: usize = 0;
    while !jobs.is_empty() {
        most_recent_time = update_timer!(rx);   // update the timer
        if most_recent_time != prev_time {      // check for new tick
            for i in 0..jobs.len() {                  // for each job
                if most_recent_time == jobs[i].arrival { // check if it has arrived yet
                    active_job_queue.push(jobs[i].clone()); // if it has push it into the active queue
                }    
            }
            let mut highest_priority: usize = 0;
            for i in 0..active_job_queue.len() {
               if active_job_queue[highest_priority].priority < active_job_queue[i].priority {
                   highest_priority = i;
               }
            }
            active_job_queue[highest_priority].active = true;
            
        }
        prev_time = most_recent_time;
    }

}
