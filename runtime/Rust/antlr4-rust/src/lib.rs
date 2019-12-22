pub mod interval;
pub mod int_stream;
pub mod char_stream;
pub mod token;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_interval() {
        let x = crate::interval::Interval::new(1, 5).to_string();
        assert_eq!("1..5".to_string(), x.to_string());
    }
}
