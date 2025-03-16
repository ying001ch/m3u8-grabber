
pub fn encode_headers(headers: &[(String,String)])-> String{
    headers.iter()
       .map(|(k,v)|format!("{}:{}",k,v))
       .collect::<Vec<String>>()
       .join("\n") 
}
pub fn decode_headers(cnt: String) -> Vec<(String, String)> {
    cnt.lines()
      .map(|line|{
        let mut it = line.splitn(2,':');
        let k = it.next().unwrap().to_string();
        let v = it.next().unwrap().to_string();
        (k,v)
      })
      .collect()
}