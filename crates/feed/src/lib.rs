use core::Message;

#[derive(Debug, Default)]
pub struct SimSequencer {
    events: Vec<Message>,
}

impl SimSequencer {
    pub fn new(events: Vec<Message>) -> Self {
        Self { events }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn drain(self) -> Vec<Message> {
        self.events
    }
}
