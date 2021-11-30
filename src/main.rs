// includes
use std::fs::File;
use std::io::*;
use std::path::Path;


// Buffer class, used by t1 and t3 to write to
#[derive(Clone, Debug)]
struct Buffer {
    buf: String,
    owner: String,
    is_owned: bool
}
impl Buffer {
    // static initializer function for Buffer
    fn init(buf: String, owner: String) -> Buffer {
        Buffer { buf, owner, is_owned: false }
    }
    // writer function for the buffer, write will fail if job does not own the buffer
    fn write_buffer(&mut self, append: &str, credentials: &String) -> bool {
        if credentials.to_string().ne(&self.owner) { 
            false
        } else {
            let _ = &self.buf.push_str(append);
            true
        }
    }
    // set new owner for the buffer
    fn set_owner(&mut self, new_owner: &String) {
        self.owner = new_owner.to_string();
        self.is_owned = true;
    }

    // frees the buffer
    fn free_buffer(&mut self) {
        self.owner = "".to_string();
        self.is_owned = false;
        self.buf = "".to_string();
    }

    // checks if a job owns the buffer, returns true if it does, false if not
    fn job_does_own(&self, credentials: &String) -> bool {
        if credentials.to_string().eq(&self.owner) {
            return true;
        }
        false
    }
}


// Job Class
#[derive(Clone, Debug)]
struct Job {
    priority: usize,        // priority of the job
    id: String,             // for differentiating t1 t2 and t3
    progress: usize,        // how far along the job is
    total_required: usize,  // how much work it needs to do
    arrival: usize,         // when it arrives
}
impl Job {
    // simple static initializer function
    fn init(priority: usize, id: String, total_required: usize, arrival: usize) -> Job {
        Job {
            priority,
            id,
            progress: 0,
            total_required,
            arrival,
        }
    }
    // for doing work on the job
    fn progress(&mut self) {
        self.progress += 1;
    }
    // for checking if the job is finished or not
    fn is_finished(&self) -> bool {
        self.progress == self.total_required
    }
    // for when t3 has the buffer but t1 is waiting for it
    fn elevate_priority(&mut self) {
        self.priority = 4;
    }
}

fn main() {
    // Job Array
    let mut jobs: Vec<Job> = vec![];                    // the job vector for handling input stream
    let mut ts: Vec<Job> = vec![];                      // template jobs that we can clone from
    ts.push(Job::init(3, String::from("t1"), 3, 0));    // T1 - uses shared buffer
    ts.push(Job::init(2, String::from("t2"), 10, 0));   // T2
    ts.push(Job::init(1, String::from("t3"), 3, 0));    // T3 - uses shared buffer
    let mut buffer: Buffer = Buffer::init(String::new(), String::from("")); // the shared buffer

    /*
    This program expects that the job stream will come from a
    file named input.txt where each job is on a new line and is in the form:
    [unsigned integer > 0] [0 < unsigned integer < 4]\n
    For example:
    1 3
    4 1
    7 2
    10 1
    creates: t3 arriving at 1, t1 arriving at 4, t2 arriving at 7 and t1 arriving at 10
    I have included a sample input.txt file in the project submission
    */

    let path = Path::new("input.txt"); // specify the path
    let display = path.display(); // for error reporting purposes
    let mut file = match File::open(&path) {
        // attempt to open the file, if it fails tell us why
        Err(why_fail) => panic!("Could not open {}: {}", display, why_fail),
        Ok(file) => file,
    };
    let mut buf = String::new();                        // buffer for reading the lines of the file
    let _ = file.read_to_string(&mut buf);              // read them in c-style
    let toks: Vec<&str> = buf.split("\n").collect();    // split the string on the on the newline
    for tok in &toks {
        // for each of those tokens
        let subtok: Vec<&str> = tok.split_ascii_whitespace().collect(); // split once again on all whitespace
        let arg1 = subtok[0].parse::<usize>().unwrap();                 // convert the strings into unsigned integers
        let arg2 = subtok[1].parse::<usize>().unwrap();
        let mut tmp: Job = ts[arg2 - 1].clone();            // create our new job
        tmp.arrival = arg1;                                 // set its arrival time
        jobs.push(tmp);                                     // push the new job
    }
    jobs.sort_by_key(|j| j.arrival);    // sort the jobs based on their arrival time (just in case if you didn't)

    // Main program execution
    let mut most_recent_time: usize = 0;            // our "timer"
    let mut active_job_queue: Vec<Job> = vec![];    // vector for jobs that have arrived and need to be worked on
    // Main loop
    while most_recent_time < 10000 {
        for i in 0..jobs.len() {                        // for each job
            if most_recent_time == jobs[i].arrival {    // check if it has arrived yet
                active_job_queue.push(jobs[i].clone()); // if it has, push it into the active queue
            }
        }
        if !active_job_queue.is_empty() {           // check for empty queue
            let mut highest_priority: usize = 0;    // highest priority job index
            for i in 0..active_job_queue.len() {    // retrieve the highest priority task in the queue
                if active_job_queue[highest_priority].priority < active_job_queue[i].priority {
                    highest_priority = i;
                }
            }
            // check for t1 being the highest priority job but t3 has the buffer
            if "t1".to_string().eq(&active_job_queue[highest_priority].id.clone()) && buffer.job_does_own(&"t3".to_string()) {
                let mut low_priority = 0;   // for retrieving the most recent lowest priority job in the queue
                for i in 0..active_job_queue.len() {
                    if active_job_queue[i].priority < active_job_queue[low_priority].priority {
                        low_priority = i;
                    }
                }
                active_job_queue[low_priority].elevate_priority();
            }
            // check for t3 or t1 being the highest priority job with no contest over the buffer
            else if active_job_queue[highest_priority].id.ne("t2") 
            && (buffer.job_does_own(&active_job_queue[highest_priority].id.clone()) || !buffer.is_owned) // make sure the job owns the buffer, or the buffer is free
            {
                let owner_id = active_job_queue[highest_priority].id.clone();   // retrieve owner id
                buffer.set_owner(&owner_id);                                    // set the new owner of the buffer
                let append = match owner_id.clone().as_str() {                  // find out what we are writing into the buffer
                    "t1" => "1",
                    "t3" => "3",
                    _ => unreachable!(),
                };
                buffer.write_buffer(append, &owner_id);                         // write into the buffer
                active_job_queue[highest_priority].progress();                  // "work" on the job
                if active_job_queue[highest_priority].is_finished()             // check for finish
                {
                    // print finishing data to stdout
                    println!(
                        "time {}: {} {} {}",
                        most_recent_time,
                        owner_id,
                        buffer.buf,
                        owner_id
                    );
                    active_job_queue.remove(highest_priority);  // since the job is finished, remove it
                    buffer.free_buffer();                       // free the buffer as well
                }
            }
            // otherwise, t2 is the highest priority job, so just run it
            else {

                let owner_id = active_job_queue[highest_priority].id.clone();
                active_job_queue[highest_priority].progress();
                if active_job_queue[highest_priority].is_finished() {
                    println!("time {}: {} {} {}", most_recent_time, owner_id, "NNNNNNNNNN", owner_id);
                    active_job_queue.remove(highest_priority);
                }

            }
            
        }
        most_recent_time += 1;      // update the timer
    }
}
