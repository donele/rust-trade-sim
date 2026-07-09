use sim_core::ReplayEvent;

#[derive(Debug, Default)]
pub struct SimSequencer {
    events: Vec<ReplayEvent>,
}

impl SimSequencer {
    pub fn new(events: Vec<ReplayEvent>) -> Self {
        Self { events }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn drain(self) -> Vec<ReplayEvent> {
        self.events
    }
}

