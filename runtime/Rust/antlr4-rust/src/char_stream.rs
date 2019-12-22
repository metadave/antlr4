pub use crate::int_stream::IntStream;
pub use crate::interval::Interval;

pub trait CharStream: IntStream {
    fn get_text(interval: Interval) -> String;
}
