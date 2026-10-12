use core::Message;

/// A source of messages that is already ordered within its own channel.
pub trait MessageSource {
    fn next_message(&mut self) -> Option<Message>;

    fn source_name(&self) -> &str {
        "SOURCE"
    }
}
