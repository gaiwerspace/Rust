use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Represents a unit of work to be processed by worker threads
pub enum WorkItem {
    /// Traverse a directory to discover subdirectories
    TraverseDirectory(std::path::PathBuf),
    /// Organize files in a directory by extension
    OrganizeDirectory(std::path::PathBuf),
    /// Signal to terminate the worker thread
    Terminate,
}

/// A worker thread that processes work items from a shared channel
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new worker thread that processes work items from the receiver
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<WorkItem>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let work_item = {
                let receiver = receiver.lock().unwrap();
                receiver.recv()
            };

            match work_item {
                Ok(WorkItem::Terminate) => {
                    break;
                }
                Ok(_work) => {
                    // Work processing will be implemented in later tasks
                    // For now, workers just receive and acknowledge work items
                }
                Err(_) => {
                    // Channel disconnected, terminate worker
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/// A thread pool that manages worker threads and distributes work
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<WorkItem>>,
}

impl ThreadPool {
    /// Create a new thread pool with the specified number of worker threads
    ///
    /// # Arguments
    /// * `size` - Number of worker threads to create (must be > 0)
    ///
    /// # Returns
    /// * `Ok(ThreadPool)` - Successfully created thread pool
    /// * `Err(String)` - Error message if size is invalid
    pub fn new(size: usize) -> Result<ThreadPool, String> {
        if size == 0 {
            return Err("Thread pool size must be greater than 0".to_string());
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Execute a work item by sending it to the worker thread pool
    ///
    /// # Arguments
    /// * `work_item` - The work item to be processed
    ///
    /// # Returns
    /// * `Ok(())` - Work item successfully queued
    /// * `Err(String)` - Error if the channel is disconnected
    pub fn execute(&self, work_item: WorkItem) -> Result<(), String> {
        self.sender
            .as_ref()
            .ok_or_else(|| "Thread pool sender is not available".to_string())?
            .send(work_item)
            .map_err(|e| format!("Failed to send work item: {}", e))
    }

    /// Gracefully shut down the thread pool by sending terminate signals
    /// to all workers and waiting for them to finish
    pub fn shutdown(&mut self) {
        // Drop the sender to signal no more work will be sent
        if let Some(sender) = self.sender.take() {
            // Send terminate signal to each worker
            for _ in &self.workers {
                let _ = sender.send(WorkItem::Terminate);
            }
        }

        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(4);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_thread_pool_zero_size() {
        let pool = ThreadPool::new(0);
        assert!(pool.is_err());
        if let Err(e) = pool {
            assert_eq!(e, "Thread pool size must be greater than 0");
        }
    }

    #[test]
    fn test_thread_pool_single_thread() {
        let pool = ThreadPool::new(1);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_thread_pool_large_size() {
        let pool = ThreadPool::new(1024);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_thread_pool_execute() {
        let pool = ThreadPool::new(2).unwrap();
        let work_item = WorkItem::TraverseDirectory(std::path::PathBuf::from("/tmp"));
        let result = pool.execute(work_item);
        assert!(result.is_ok());
    }

    #[test]
    fn test_thread_pool_shutdown() {
        let mut pool = ThreadPool::new(4).unwrap();
        pool.shutdown();
        // After shutdown, the sender should be None
        // Attempting to execute should fail
        let work_item = WorkItem::TraverseDirectory(std::path::PathBuf::from("/tmp"));
        let result = pool.execute(work_item);
        assert!(result.is_err());
    }

    #[test]
    fn test_thread_pool_drop() {
        // Test that Drop implementation properly cleans up
        {
            let _pool = ThreadPool::new(4).unwrap();
            // Pool will be dropped here
        }
        // If we reach this point without hanging, the test passes
    }

    #[test]
    fn test_thread_pool_multiple_work_items() {
        let pool = ThreadPool::new(4).unwrap();
        
        for i in 0..10 {
            let path = std::path::PathBuf::from(format!("/tmp/dir{}", i));
            let work_item = WorkItem::TraverseDirectory(path);
            let result = pool.execute(work_item);
            assert!(result.is_ok());
        }
        
        // Give workers time to process
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_thread_pool_work_item_types() {
        let pool = ThreadPool::new(2).unwrap();
        
        // Test TraverseDirectory
        let result = pool.execute(WorkItem::TraverseDirectory(
            std::path::PathBuf::from("/tmp/test1")
        ));
        assert!(result.is_ok());
        
        // Test OrganizeDirectory
        let result = pool.execute(WorkItem::OrganizeDirectory(
            std::path::PathBuf::from("/tmp/test2")
        ));
        assert!(result.is_ok());
        
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn test_thread_pool_concurrent_execution() {
        let pool = Arc::new(Mutex::new(ThreadPool::new(4).unwrap()));
        let mut handles = vec![];
        
        for i in 0..8 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let pool = pool_clone.lock().unwrap();
                let path = std::path::PathBuf::from(format!("/tmp/concurrent{}", i));
                pool.execute(WorkItem::TraverseDirectory(path))
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_thread_pool_graceful_shutdown_with_pending_work() {
        let mut pool = ThreadPool::new(2).unwrap();
        
        // Queue up some work
        for i in 0..5 {
            let path = std::path::PathBuf::from(format!("/tmp/pending{}", i));
            let _ = pool.execute(WorkItem::TraverseDirectory(path));
        }
        
        // Shutdown should wait for workers to finish
        pool.shutdown();
        
        // After shutdown, execute should fail
        let result = pool.execute(WorkItem::TraverseDirectory(
            std::path::PathBuf::from("/tmp/after_shutdown")
        ));
        assert!(result.is_err());
    }
}
