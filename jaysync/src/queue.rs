use std::{
    ops::Add,
    sync::{
        Arc, Mutex, RwLock,
        atomic::AtomicBool,
        mpsc::{self, Receiver, Sender},
    },
    task::Wake,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use polling::{Events, Poller};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub enum ScheduledEventMode {
    #[default]
    Instant,
    Repeat,
}

pub struct ScheduledEvent<Event> {
    event: Event,
    mode: ScheduledEventMode,
}

pub struct EventScheduler<Event> {
    sender: Arc<Sender<Event>>,
    scheduled: Arc<Mutex<Vec<Event>>>,
    poller: Arc<Poller>,
    checking_thread: JoinHandle<()>,
    run_check_thread: Arc<AtomicBool>,
}

impl<Event: Sync + Send + 'static> EventScheduler<Event> {
    pub fn new(sender: &Arc<Sender<Event>>) -> Self {
        let checking_sender = Arc::clone(sender);

        let poller = Arc::new(Poller::new().unwrap());
        let scheduled_list = Arc::new(Mutex::new(Vec::new()));
        let scheduled_list_check = scheduled_list.clone();
        let check_polled = poller.clone();

        let check_thread = std::thread::spawn(move || {
            let mut events = Events::new();
            loop {
                check_polled
                    .wait_deadline(&mut events, Instant::now().add(Duration::from_millis(200)))
                    .unwrap();
                let mut lock = scheduled_list_check.lock().unwrap();
                while let Some(event) = lock.pop() {
                    checking_sender.send(event).unwrap();
                }
            }
        });
        Self {
            run_check_thread: Arc::new(AtomicBool::new(true)),
            sender: sender.clone(),
            poller: poller,
            scheduled: scheduled_list,
            checking_thread: check_thread,
        }
    }

    pub fn push(&mut self, event: Event) {}

    pub fn notify(&self) -> () {
        self.poller.notify().unwrap();
    }
}

impl<Event> Wake for EventScheduler<Event> {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {}
}
