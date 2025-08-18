//! Priority message queue implementation

use crate::{GossipMessage, MessagePriority};
use priority_queue::PriorityQueue;

#[derive(Debug, Clone)]
pub struct MessageItem {
    pub message: GossipMessage,
    pub priority: MessagePriority,
}

pub struct PriorityMessageQueue {
    queue: PriorityQueue<GossipMessage, MessagePriority>,
}

impl PriorityMessageQueue {
    pub fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
        }
    }

    pub fn push(&mut self, message: GossipMessage, priority: MessagePriority) {
        self.queue.push(message, priority);
    }

    pub fn pop(&mut self) -> Option<(GossipMessage, MessagePriority)> {
        self.queue.pop()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
