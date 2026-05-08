use std::sync::mpsc::Sender;

pub trait HookableSource<Event> {
    fn hook(&mut self, sender: Sender<Event>);
}
