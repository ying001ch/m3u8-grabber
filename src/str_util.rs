pub fn index_of(ch: char, str: &str) -> i32 {
    let mut idx = 0;
    while idx < str.len() {
        if str.chars().nth(idx).unwrap() == ch {
            return idx as i32;
        }
        idx += 1;
    }
    return -1;
}
pub fn last_index(ch: char, str: &str) -> i32 {
    let mut idx:i32 = (str.len() - 1) as i32;
    let chstr = ch.to_string();

    while idx >= 0 {
        let c = str.get(idx as usize..(idx + 1) as usize).unwrap();
        if c == chstr {
            // println!("c={}",c);
            return idx;
        }
        idx -= 1;
    }
    return -1;
}