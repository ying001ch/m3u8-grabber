use crate::M3u8Item;
use M3u8Item::DownParam;


fn test_json(){
    let mut param = M3u8Item::DownParam::default();
    
    param.address =  "String".to_string();
    param.save_path = "/opt".to_string();
    param.proxy = Some("http://127.0.0.1:1081".to_string());
    param.headers = Some("refer: http://baidu.com".to_string());
    
    
      // Convert the Point to a JSON string.
      let serialized = serde_json::to_string(&param).unwrap();
    
      // Prints serialized = {"x":1,"y":2}
      println!("serialized = {}", serialized);
    
      // Convert the JSON string back to a Point.
      let deserialized: DownParam = serde_json::from_str(&serialized).unwrap();
    
      // Prints deserialized = Point { x: 1, y: 2 }
      println!("deserialized = {:?}", deserialized);
}
#[cfg(test)]
mod Test{
    use std::mem::discriminant;
    use std::num::ParseIntError;
    use std::ops::Deref;
    use std::sync::Arc;
    use std::sync::Condvar;
    use std::sync::OnceLock;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    use crate::config::Signal;

    use super::DownParam;
    use super::M3u8Item;
    #[test]
    fn test_hex_parse() {
        let s = "0x1f58ab9c1f58ab9c1f58ab9c1f58ab98";
        //1.解析字符方式 每两位转换为一个u8
        let us = M3u8Item::hex2_byte(s);  
        
        //2.先整体转换成u128 再位运算
        let mut t = u128::from_str_radix(&s[2..], 16).unwrap();
        let mut u2 = [0u8;16];
        for i in 0..16 {
            u2[u2.len()-1-i] = (t & 0x00ffu128) as u8;
            t = t >> 8;
        }
        println!("us={:?}, len={}", us,us.len());
        println!("u2={:?}, len={}", u2,u2.len());
        assert_eq!(us, u2);
    }
    #[test]
    fn test_enum(){
        println!("discriminant(Normal) = {:?}", discriminant(&Signal::Normal));
        println!("discriminant(Pause) = {:?}", discriminant(&Signal::Pause));
        println!("discriminant(End) = {:?}", discriminant(&Signal::End));
        // assert_eq!(discriminant(&Signal::Normal), discriminant(&Signal::End));
        // assert_eq!(discriminant(&Signal::End), discriminant(&Signal::Pause));
        assert_eq!(discriminant(&Signal::Normal), discriminant(&Signal::Normal));
    }
    #[test]
    fn test_rw_lock(){
        let lock = std::sync::RwLock::new(12);
        { //读锁；获取多个读锁不会阻塞
            let r1 = lock.read().unwrap();
            let r2 = lock.read().unwrap();
            println!("r1 = {:?}", r1);
            println!("r2 = {:?}", r2);
        }
        
        let arc_lock = Arc::new(lock);
        { //写锁
            let clone_lock = arc_lock.clone();
            std::thread::spawn(move ||{
                let mut w = clone_lock.write().unwrap();
                *w = 90;
                std::thread::sleep(Duration::from_secs(5));
                println!("===> 子线程释放锁")
            });

            std::thread::sleep(Duration::from_millis(500));
            
            //写锁 未释放时获取读锁会阻塞
            let r = arc_lock.read().unwrap();
            println!("r1 = {:?}", r);
        }
        println!("r = {:?}", arc_lock.read().unwrap());
    }
    #[test]
    fn test_atomic(){
        let counter = Arc::new(AtomicU32::new(100));
        let counter_c = counter.clone();
        let joiner= thread::spawn(move ||{
            for _ in 0..1000{
                counter_c.fetch_add(1, Ordering::Relaxed);
            }
        });

        for _ in 0..1000{
            counter.fetch_add(1, Ordering::Relaxed);
        }
        joiner.join();

        println!("atomic = {:?}", counter);
        assert_eq!(counter.load(Ordering::Relaxed), 2100);
    }
    #[test]
    fn test_lazy(){
        static lock: OnceLock<i32> = std::sync::OnceLock::new();
        let lock_ref = lock.get_or_init(||{
            //初始化只会被执行一次
            println!("初始化1");
            99
        });
        let lock_ref_2 = lock.get_or_init(||{
            println!("初始化2");
            99
        });
        println!("lock_ref = {:?}", lock);
    }

}