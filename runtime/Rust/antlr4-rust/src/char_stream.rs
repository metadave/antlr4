pub use crate::interval::Interval;
pub use crate::int_stream::IntStream;

pub trait CharStream: IntStream {
    fn get_text(interval: Interval) -> String;
}