
fn test_json(){
    let param = main::DownParam { 
        url: "String".to_string(),
        savePath: "/opt".to_string(),
        proxy: Some("http://127.0.0.1:1081".to_string()),
        headers: vec!["refer: http://baidu.com".to_string()],
       };
    
      // Convert the Point to a JSON string.
      let serialized = serde_json::to_string(&param).unwrap();
    
      // Prints serialized = {"x":1,"y":2}
      println!("serialized = {}", serialized);
    
      // Convert the JSON string back to a Point.
      let deserialized: DownParam = serde_json::from_str(&serialized).unwrap();
    
      // Prints deserialized = Point { x: 1, y: 2 }
      println!("deserialized = {:?}", deserialized);
}