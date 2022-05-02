use std::thread;
use std::sync::mpsc; // multiple producer, single consumer
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    workers: Vec<Worker>, // dont need closures to return anything
    sender: mpsc::Sender<Job>
}

// alias trait object containing a one use closure to Job
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        // just assert and panic, dont bother with results cause there should be
        // no handling for 0 threads, the software just wont work.
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        // is a little bit more efficient to pre-allocate the memory here with #with_capacity
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    // take a closure arg thats called once, remember closures are defined as trait like this
    // Send to transfer closure from one thread to another and 'static
    // because we dont know how long the thread will take to execute
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f); // create a Job instance (our alias)

        self.sender.send(job).unwrap(); // send that job down the channel
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // loop forever constantly ask the receiving end of the channel for a job and running it when it gets one
        let thread = thread::spawn(move || loop {
            let job = receiver.lock()
                .expect("Poisoned state, another thread panicked.")
                .recv() // sleep thread until a job becomes available. Mutex ensures only 1 thread at a time receives a job
                .unwrap();
            println!("Worker {} got a job; executing.", id);
            job(); // execute closure / job
        });
        Worker { id, thread }
    }
}
