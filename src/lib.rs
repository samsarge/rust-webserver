use std::thread;
use std::sync::mpsc; // multiple producer, single consumer
use std::sync::Arc;
use std::sync::Mutex;


// alias trait object containing a one use closure to Job
type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    NewJob(Job),
    Terminate
}
pub struct ThreadPool {
    workers: Vec<Worker>, // dont need closures to return anything
    sender: mpsc::Sender<Message>
}

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

        self.sender.send(Message::NewJob(job)).unwrap(); // send that job down the channel
    }
}

// call join on all the worker threads when shutting down / dropping a threadpool
// note to self: join takes ownership so cant be working with references.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            // they stop their infinite loops if they receive this.
            // otherwise the loop would continue and join would wait for it to finish (it never would)
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        // call join in a second loop to prevent deadlocks, aka once every worker
        // has already received the terminate message.
        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);
            // use take to take ownership of Option<thread::JoinHandle<()>> and change variant to None.
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // loop forever constantly ask the receiving end of the channel for a job and running it when it gets one
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate. Terminating.", id);
                    break;
                }
            }

        });
        Worker { id, thread: Some(thread) }
    }
}
