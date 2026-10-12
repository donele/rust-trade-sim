use core::Message;

use crate::MessageSource;

#[derive(Debug, Clone)]
pub struct VecMessageSource {
    name: String,
    messages: Vec<Message>,
    next_index: usize,
}

impl VecMessageSource {
    pub fn new(messages: Vec<Message>) -> Self {
        Self::named("UNNAMED", messages)
    }

    pub fn named(name: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            name: name.into(),
            messages,
            next_index: 0,
        }
    }
}

impl MessageSource for VecMessageSource {
    fn next_message(&mut self) -> Option<Message> {
        let message = self.messages.get(self.next_index).copied();
        self.next_index += usize::from(message.is_some());
        message
    }

    fn source_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{BookTicker, SymbolId};

    #[test]
    fn source_name_is_available_for_diagnostics() {
        let source = VecMessageSource::named(
            "MARKET_DATA",
            vec![Message::BookTicker(BookTicker {
                ts_nanos: 1,
                symbol: SymbolId(1),
                best_bid_price: 1.0,
                best_bid_qty: 1.0,
                best_ask_price: 2.0,
                best_ask_qty: 1.0,
                update_id: 1,
            })],
        );
        assert_eq!(source.source_name(), "MARKET_DATA");
    }
}
