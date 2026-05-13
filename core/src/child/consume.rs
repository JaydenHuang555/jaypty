use crate::child::killer::ConsumedChildKiller;

pub trait ConsumedChildConsumer<K: ConsumedChildKiller> {
    fn killer(self) -> K;
}
