//! The server threadpool
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::marker::Sync;

/// This implementation of `ThreadPool` is based of the one in [the
/// book](https://doc.rust-lang.org/book/second-edition/ch20-04-storing-threads.html). The
/// documentation may be better there than it is here.
pub struct ThreadPool {
    workers : Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

unsafe impl Sync for Message {}

type Job = Box<FnBox + Send + 'static>;

/// A function that takes another function to execute in the threadpool.
pub type Executor<F: FnOnce() + Send + 'static> = Box<Fn(F) -> () + Send + 'static>;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    #[cfg_attr(feature = "cargo-clippy", allow(boxed_local))]
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

impl ThreadPool {

    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    /// Execute a job in the threadpool.
    pub fn execute<F>(&self, f: F) 
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }

    /// Creates a way to send jobs to the threadpool from other threads e.g.
    /// the HTTPS thread. Similar to a partial function.
    pub fn create_executor<F>(&self) -> Executor<F>
        where
            F: FnOnce() + Send + 'static
    {

        let s = self.sender.clone();
        Box::new(move |f: F| {
            let job = Box::new(f);
            s.send(Message::NewJob(job)).unwrap();
        })
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {

    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop{
                let message = receiver.lock().unwrap().recv().unwrap();
                
                match message {
                    Message::NewJob(job) => {
                        debug!("Executing on Worker {}", id);

                        job.call_box();
                    },
                    Message::Terminate => {
                        debug!("Worker {} was told to terminate.", id);

                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {

    use threadpool;
    use std::thread;
    use std::sync::mpsc;
    
    #[test]
    fn assert_thread_pool_size_correct() {
        let pool = threadpool::ThreadPool::new(4);

        assert_eq!(pool.workers.len(), 4);
    }

    #[test]
    fn assert_threadpool_does_not_exceed_max() {
        let pool = threadpool::ThreadPool::new(2);

        for _ in 0..4 {
            pool.execute( move || { 2+2; } );
        }

        assert_eq!(pool.workers.len(), 2);
    }

    #[test]
    #[should_panic]
    fn assert_threads_must_be_at_least_1() {
        let _ = threadpool::ThreadPool::new(0);
    }
    
        
    /// Passes a boolean back to the main (test) thread ensuring the closure is 
    /// called.
    #[test]
    fn assert_execute_works() {

        let (tx, rx) = mpsc::channel();

        let pool = threadpool::ThreadPool::new(2);
        pool.execute( move || {
            tx.send(true).unwrap();
        });

        let received = rx.recv().unwrap();
        assert!(received);
    }

    #[test]
    fn assert_executor_works() {

        let (tx, rx) = mpsc::channel();

        let pool = threadpool::ThreadPool::new(2);
        let executor = pool.create_executor();
        thread::spawn( move || {
            executor( move || {
                tx.send(true).unwrap();
            });
        });

        let received = rx.recv().unwrap();
        assert!(received);

    }

}

