pub const INT_STREAM_EOF: i32 = -1;
pub trait IntStream {
    fn consume() -> ();
    fn la(i: i32) -> i32;
    fn mark() -> i32;
    fn release(marker: i32) -> ();
    fn index() -> i32;
    fn seek(index: i32);
    fn size() -> i32;
    fn get_source_name() -> String;
}
