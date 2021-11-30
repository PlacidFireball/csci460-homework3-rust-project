use std::fs::File;
use std::io::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
struct Buffer {
    buf: String,
    owner: String,
}
impl Buffer {
    fn init(buf: String, owner: String) -> Buffer {
        Buffer { buf, owner }
    }
    fn write_buffer(&mut self, append: &str, credentials: &String) -> bool {
        if credentials.to_string().ne(&self.owner) {
            false
        } else {
            let _ = &self.buf.push_str(append);
            true
        }
    }
    fn set_owner(&mut self, new_owner: &String) {
        self.owner = new_owner.to_string();
    }
    fn job_does_own(&self, credentials: &String) -> bool {
        if credentials.to_string().eq(&self.owner) {
            return true;
        }
        false
    }
}

#[derive(Clone, Debug)]
struct Job {
    priority: usize,
    id: String,
    progress: usize,
    total_required: usize,
    arrival: usize,
    has_mutex: bool,
}
impl Job {
    fn init(priority: usize, id: String, total_required: usize, arrival: usize) -> Job {
        Job {
            priority,
            id,
            progress: 0,
            total_required,
            arrival,
            has_mutex: false,
        }
    }
    fn progress(&mut self) {
        self.progress += 1;
    }

    fn elevate_priority(&mut self) {
        self.priority = 4;
    }
    fn lower_priority(&mut self) {
        self.priority = 1;
    }
}

fn main() {
    // Job Array
    let mut jobs: Vec<Job> = vec![]; // the job vector for handling input stream
    let mut ts: Vec<Job> = vec![]; // template jobs that we can clone from
    ts.push(Job::init(3, String::from("t1"), 3, 0)); // T1 - shared buffer
    ts.push(Job::init(2, String::from("t2"), 10, 0)); // T2
    ts.push(Job::init(1, String::from("t3"), 3, 0)); // T3 - shared buffer
    let mut buffer: Buffer = Buffer::init(String::new(), String::from("")); // the shared buffer
    let mut t2_buffer: Buffer = Buffer::init(String::new(), String::from("")); // t2's buffer
                                                                               // println!("{:?}", ts);

    /*
    This program expects that the job stream will come from a
    file named input.txt where each job is on a new line and is in the form:
    [unsigned integer > 0] [0 < unsigned integer < 4]\n
    I have included a sample input.txt file in the project submission
    */

    let path = Path::new("input.txt"); // specify the path
    let display = path.display(); // for error reporting purposes
    let mut file = match File::open(&path) {
        // attempt to open the file, if it fails tell us why
        Err(why_fail) => panic!("Could not open {}: {}", display, why_fail),
        Ok(file) => file,
    };
    let mut buf = String::new(); // buffer for reading the lines of the file
    let _ = file.read_to_string(&mut buf); // read them c-style
                                           //println!("text:\n{}", buf);
    let toks: Vec<&str> = buf.split("\n").collect(); // split on the newline
    for tok in &toks {
        // for each of those tokens
        let subtok: Vec<&str> = tok.split_ascii_whitespace().collect(); // split once again on all whitespace
                                                                        //println!("{:?}", subtok);     // for debugging
        let arg1 = subtok[0].parse::<usize>().unwrap(); // convert the strings into unsigned integers
        let arg2 = subtok[1].parse::<usize>().unwrap();
        let mut tmp: Job = ts[arg2 - 1].clone(); // create our new job
        tmp.arrival = arg1; // set its arrival time
        jobs.push(tmp); // push the new job
    }
    jobs.sort_by_key(|j| j.arrival); // sort the jobs based on their arrival time (just in case if you didn't)
                                     //println!("{:?}", jobs); // debugging purposes

    // Main program execution
    let mut most_recent_time: usize = 0;
    let mut prev_time: usize = 0;
    let mut active_job_queue: Vec<Job> = vec![];
    while !jobs.is_empty() {
        most_recent_time += 1; // update the timer
        if most_recent_time != prev_time {
            // check for new tick
            for i in 0..jobs.len() {
                // for each job
                if most_recent_time == jobs[i].arrival {
                    // check if it has arrived yet
                    active_job_queue.push(jobs[i].clone()); // if it has push it into the active queue
                }
            }
            if !active_job_queue.is_empty() {
                let mut highest_priority: usize = 0;
                let mut jobs_attempting: Vec<usize> = vec![];
                for i in 0..active_job_queue.len() {
                    // retrieve the highest priority task in the queue
                    if active_job_queue[highest_priority].priority < active_job_queue[i].priority {
                        highest_priority = i;
                    }
                    if active_job_queue[i].has_mutex {
                        jobs_attempting.push(i);
                    }
                }
                // check if we are dealing with t1 or t3
                if active_job_queue[highest_priority].id.ne("t2") {
                    active_job_queue[highest_priority].has_mutex = true;
                    jobs_attempting.push(highest_priority);
                }
                // check for t3 and t1 being highest priority (no contest)
                if jobs_attempting.len() == 1 && active_job_queue[highest_priority].id.ne("t2") {
                    let owner_id = active_job_queue[highest_priority].id.clone(); // retrieve owner id
                    buffer.set_owner(&owner_id); // set the new owner of the buffer
                    let append = match owner_id.clone().as_str() {
                        "t1" => "1",
                        "t2" => unreachable!(),
                        "t3" => "3",
                        _ => unreachable!(),
                    };
                    buffer.write_buffer(append, &owner_id);
                    active_job_queue[highest_priority].progress(); // progress the job
                                                                   // check for finish
                    if active_job_queue[highest_priority].progress
                        == active_job_queue[highest_priority].total_required
                    {
                        println!(
                            "time {}: {} {} {}",
                            active_job_queue[highest_priority].arrival,
                            owner_id,
                            buffer.buf,
                            owner_id
                        );
                        active_job_queue.remove(highest_priority);
                    }
                }
                // otherwise we are dealing with t2
                else if jobs_attempting.len() == 1 {
                    let owner_id = active_job_queue[highest_priority].id.clone();
                    active_job_queue[highest_priority].progress();
                    t2_buffer.set_owner(&owner_id);
                    t2_buffer.write_buffer("2", &owner_id);
                    if active_job_queue[highest_priority].progress
                        == active_job_queue[highest_priority].total_required
                    {
                        println!("{}{}{}", owner_id, buffer.buf, owner_id);
                        active_job_queue.remove(highest_priority);
                    }
                }
                // the buffer is contested and t3 has the buffer
                else if jobs_attempting.len() > 1 && buffer.job_does_own(&"t3".to_string()) {
                    let mut low_priority = 0;
                    for i in 0..active_job_queue.len() {
                        if active_job_queue[i].priority < active_job_queue[low_priority].priority {
                            low_priority = i;
                        }
                    }
                    active_job_queue[low_priority].elevate_priority();
                    active_job_queue[highest_priority].has_mutex = false;
                }
            }
        }
        prev_time = most_recent_time;
    }
}
