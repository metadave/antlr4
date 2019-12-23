pub use crate::antlr4::runtime::int_stream::IntStream;
pub use crate::antlr4::runtime::misc::interval::Interval;

pub trait CharStream: IntStream {
    fn get_text(interval: Interval) -> String;
}
