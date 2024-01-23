pub fn index_of(ch: char, str: &str) -> i32 {
    str.find(ch).map(|i|i as i32).unwrap_or(-1)
}
pub fn last_index(ch: char, str: &str) -> i32 {
    str.rfind(ch).map(|i|i as i32).unwrap_or(-1)
}