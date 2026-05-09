use std::{
    sync::{Arc, RwLock, mpsc::Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub mod capture;
pub mod io;
pub mod mpsc;
pub mod notifier;
pub mod queue;
pub mod wake;

pub trait AsyncBooleanTrigger<Output: 'static + Send>: Send + 'static {
    fn triggered(&self) -> bool;
    fn output(&self) -> Output;
}

pub struct UnNamedAsyncBooleanTrigger<
    TriggerGetter: Fn() -> bool + 'static,
    Output: 'static + Send,
    OutputGetter: Fn() -> Output + 'static,
> {
    triggered: TriggerGetter,
    output: OutputGetter,
}

impl<
    Triggered: Fn() -> bool + 'static,
    Output: 'static + Send,
    OutputGetter: Fn() -> Output + 'static,
> UnNamedAsyncBooleanTrigger<Triggered, Output, OutputGetter>
{
    pub fn new(triggered: Triggered, output: OutputGetter) -> Self {
        Self {
            triggered: triggered,
            output: output,
        }
    }
}

unsafe impl<Trigger: Fn() -> bool, Output: 'static + Send, OutputGetter: Fn() -> Output + 'static>
    Send for UnNamedAsyncBooleanTrigger<Trigger, Output, OutputGetter>
{
}

impl<
    Trigger: Fn() -> bool + 'static,
    Output: 'static + Send,
    OutputGetter: Fn() -> Output + 'static,
> AsyncBooleanTrigger<Output> for UnNamedAsyncBooleanTrigger<Trigger, Output, OutputGetter>
{
    fn triggered(&self) -> bool {
        (self.triggered)()
    }

    fn output(&self) -> Output {
        (self.output)()
    }
}

pub fn spawn_send_on_triggered<
    TriggerOutput: 'static + Send,
    Trigger: AsyncBooleanTrigger<TriggerOutput>,
>(
    trigger_input: Trigger,
    sender: Sender<TriggerOutput>,
    interval_duration: Duration,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(interval_duration);
            if trigger_input.triggered() {
                let output = trigger_input.output();
                sender.send(output).unwrap();
                break;
            }
        }
        ()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
