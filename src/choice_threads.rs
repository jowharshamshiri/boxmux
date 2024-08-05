use crossbeam::channel::{unbounded, Receiver, Sender};
use rayon::prelude::*;
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

pub struct ChoiceResultPacket<T, E> {
    pub choice_id: String,
    pub panel_id: String,
    pub result: Result<T, E>,
}

#[derive(Debug)]
pub struct ChoiceResult<T> {
    pub success: bool,
    pub result: T,
}

impl<T> ChoiceResult<T> {
    pub fn new(success: bool, result: T) -> Self {
        ChoiceResult { success, result }
    }
}

#[derive(Debug)]
pub struct ChoiceThreadManager {
    sender: Sender<Box<dyn FnOnce(Sender<Box<dyn Any + Send>>) + Send>>,
    receiver: Arc<Mutex<Receiver<Box<dyn FnOnce(Sender<Box<dyn Any + Send>>) + Send>>>>,
    result_sender: Sender<Box<dyn Any + Send>>,
    result_receiver: Arc<Mutex<Receiver<Box<dyn Any + Send>>>>,
}

impl ChoiceThreadManager {
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver): (
            Sender<Box<dyn FnOnce(Sender<Box<dyn Any + Send>>) + Send>>,
            Receiver<Box<dyn FnOnce(Sender<Box<dyn Any + Send>>) + Send>>,
        ) = unbounded();
        let receiver = Arc::new(Mutex::new(receiver));
        let (result_sender, result_receiver) = unbounded();
        let result_receiver = Arc::new(Mutex::new(result_receiver));

        // Create the thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .panic_handler(|panic_info| {
                log::error!("Panic in thread pool: {:?}", panic_info);
            })
            .exit_handler(|exit_info| {
                log::info!("Thread pool exited: {:?}", exit_info);
            })
            .start_handler(|thread_index| {
                log::info!("Thread {} started", thread_index);
            })
            .build()
            .unwrap();

        // Start the worker threads
        for _ in 0..num_threads {
            let receiver_clone = Arc::clone(&receiver);
            let result_sender_clone = result_sender.clone();
            pool.spawn(move || loop {
                match receiver_clone.lock().unwrap().try_recv() {
                    Ok(task) => {
                        task(result_sender_clone.clone());
                    }
                    Err(_) => {
                        // std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            });
        }

        ChoiceThreadManager {
            sender,
            receiver,
            result_sender,
            result_receiver,
        }
    }

    pub fn execute<F, T, E>(&self, choice_id: String, panel_id: String, job: F)
    where
        F: FnOnce(Sender<Result<T, E>>) + Send + 'static,
        T: Any + Send + 'static + std::fmt::Debug,
        E: Any + Send + 'static + std::fmt::Debug,
    {
        let job_boxed = Box::new(move |sender: Sender<Box<dyn Any + Send>>| {
            let (res_sender, res_receiver) = unbounded();
            job(res_sender);
            let result = res_receiver.recv().unwrap();
            let packet: Box<dyn Any + Send> = Box::new(ChoiceResultPacket {
                choice_id,
                panel_id,
                result,
            });
            sender.send(packet).unwrap();
        }) as Box<dyn FnOnce(Sender<Box<dyn Any + Send>>) + Send>;

        log::info!("Sending job to thread pool");
        self.sender.send(job_boxed).unwrap();
    }

    pub fn get_results<T, E>(&self) -> Vec<ChoiceResultPacket<T, E>>
    where
        T: Any + Send + 'static + std::fmt::Debug,
        E: Any + Send + 'static + std::fmt::Debug,
    {
        let mut results = Vec::new();
        let expected_type_id = TypeId::of::<ChoiceResultPacket<T, E>>();

        log::info!("Fetching results from result_receiver");
        while let Ok(res) = self.result_receiver.lock().unwrap().try_recv() {
            log::info!("Result received");
            match res.downcast::<ChoiceResultPacket<T, E>>() {
                Ok(res) => {
                    results.push(*res);
                }
                Err(res) => {
                    log::error!(
                        "Result type mismatch: expected {:?}, got {:?}",
                        expected_type_id,
                        res.type_id()
                    );
                }
            }
        }
        log::info!("Total results fetched: {}", results.len());
        results
    }
}

// pub fn main() {
//     // Initialize the ChoiceThreadManager with 4 threads
//     let manager = ChoiceThreadManager::new(4);

//     // Define a simple job
//     let job = |sender: Sender<Result<i32, String>>| {
//         let result = 2 + 2;
//         sender.send(Ok(result)).unwrap();
//     };

//     // Execute the job with choice_id and panel_id
//     manager.execute("choice_1".to_string(), "panel_1".to_string(), job);

//     // Execute more jobs to test concurrency
//     for i in 2..6 {
//         let job = |sender: Sender<Result<i32, String>>| {
//             let result = i * 2;
//             sender.send(Ok(result)).unwrap();
//         };
//         manager.execute(format!("choice_{}", i), "panel_1".to_string(), job);
//     }

//     // Simulate other work while the jobs are processed
//     std::thread::sleep(std::time::Duration::from_secs(1));

//     // Retrieve and print the results
//     let results: Vec<ChoiceResultPacket<i32, String>> = manager.get_results();
//     for result in results {
//         match result.result {
//             Ok(value) => println!(
//                 "Result for {}-{}: {:?}",
//                 result.choice_id, result.panel_id, value
//             ),
//             Err(e) => println!(
//                 "Error for {}-{}: {:?}",
//                 result.choice_id, result.panel_id, e
//             ),
//         }
//     }
// }
