use std::{
    sync::Arc,
    collections::VecDeque,
};

use tokio::sync::Mutex;


type InternalQueue<T> = Arc<Mutex<VecDeque<T>>>;


#[derive(Clone)]
pub struct MessageQueue<T> {
    internal: InternalQueue<T>,
}


impl<T> MessageQueue<T> {
    pub fn new() -> Self {
        Self {
            internal: Arc::new(
                Mutex::new(
                    VecDeque::new()
                )
            )
        }
    }

    pub async fn push(&self, item: T) {
        let mut internal = self.internal
            .lock()
            .await;

        internal.push_back(item);
    }

    pub async fn push_multiple(&self, items: impl IntoIterator<Item = T>) {
        let mut internal = self.internal
            .lock()
            .await;

        for item in items {
            internal.push_back(item);
        }
    }

    // TODO: Rename this function
    pub fn drain(&self) -> Vec<T> {
        let mut internal = self.internal
            .blocking_lock();

        internal
            .drain(..)
            .collect()
    }
}
