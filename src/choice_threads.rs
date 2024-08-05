use crossbeam::channel::{unbounded, Receiver, Sender};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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
    queued_jobs: Arc<Mutex<HashMap<String, (String, String)>>>,
    executing_jobs: Arc<Mutex<HashMap<String, (String, String)>>>,
    finished_jobs: Arc<Mutex<HashMap<String, (String, String)>>>,
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

        let queued_jobs: Arc<Mutex<HashMap<String, (String, String)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let executing_jobs: Arc<Mutex<HashMap<String, (String, String)>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let finished_jobs: Arc<Mutex<HashMap<String, (String, String)>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Start the worker threads
        for _ in 0..num_threads {
            let receiver_clone = Arc::clone(&receiver);
            let result_sender_clone = result_sender.clone();
            let queued_jobs_clone = Arc::clone(&queued_jobs);
            let executing_jobs_clone = Arc::clone(&executing_jobs);
            let finished_jobs_clone = Arc::clone(&finished_jobs);

            pool.spawn(move || loop {
                let task = {
                    let rcv = receiver_clone.lock().unwrap();
                    rcv.recv()
                };
                if let Ok(task) = task {
                    let (job_id, choice_id, panel_id) = {
                        let queued_jobs = queued_jobs_clone.lock().unwrap();
                        let (job_id, (choice_id, panel_id)) = queued_jobs.iter().next().unwrap();
                        (job_id.clone(), choice_id.clone(), panel_id.clone())
                    };

                    {
                        let mut queued_jobs = queued_jobs_clone.lock().unwrap();
                        queued_jobs.remove(job_id.as_str());
                    }

                    {
                        let mut executing_jobs = executing_jobs_clone.lock().unwrap();
                        executing_jobs
                            .insert(job_id.clone(), (choice_id.clone(), panel_id.clone()));
                    }

                    task(result_sender_clone.clone());
                    {
                        let mut executing_jobs = executing_jobs_clone.lock().unwrap();
                        executing_jobs.remove(job_id.as_str());

                        let mut finished_jobs = finished_jobs_clone.lock().unwrap();
                        finished_jobs
                            .insert(job_id.clone(), (choice_id.clone(), panel_id.to_string()));
                    }
                }
            });
        }

        ChoiceThreadManager {
            sender,
            receiver,
            result_sender,
            result_receiver,
            queued_jobs,
            executing_jobs,
            finished_jobs,
        }
    }

    pub fn execute<F, T, E>(&self, choice_id: String, panel_id: String, job: F)
    where
        F: FnOnce(Sender<Result<T, E>>) + Send + 'static,
        T: Any + Send + 'static + std::fmt::Debug,
        E: Any + Send + 'static + std::fmt::Debug,
    {
        {
            let job_id = Uuid::new_v4().to_string();
            let mut queued_jobs = self.queued_jobs.lock().unwrap();
            queued_jobs.insert(job_id, (choice_id.clone(), panel_id.clone()));
        }

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

    pub fn get_queued_jobs(&self) -> HashMap<String, (String, String)> {
        self.queued_jobs.lock().unwrap().clone()
    }

    pub fn get_executing_jobs(&self) -> HashMap<String, (String, String)> {
        self.executing_jobs.lock().unwrap().clone()
    }

    pub fn get_finished_jobs(&self) -> HashMap<String, (String, String)> {
        self.finished_jobs.lock().unwrap().clone()
    }
}
