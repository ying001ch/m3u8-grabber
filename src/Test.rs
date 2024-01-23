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
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    use tokio::runtime::Runtime;

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
    #[test]
    fn test_tuple(){
        // let (mut a, mut b) = (1, 2);
        let mut a = 1;
        let mut b = 2;
        println!("a={},b={}", a, b);
        //通过元组交换值
        (a,b) = (b,a);
        println!("a={},b={}", a, b);
    }
    #[test]
    fn test_oneshot(){
        // 只有 没有内部值的枚举才能使用as 转换成整数
        println!("signal idx = {:?}", Signal::Normal as usize);
        println!("signal idx = {:?}", Signal::Pause as usize);

        println!("discriminant(Normal) idx = {:?}", discriminant(&Signal::Normal));
        println!("discriminant(Pause) idx = {:?}", discriminant(&Signal::Pause));

        enum Testenum{
            A,
            B(i32),
            C(f32)
        }
        println!("discriminant(A) idx = {:?}", discriminant(&Testenum::A));
        println!("discriminant(B(10)) idx = {:?}", discriminant(&Testenum::B(10)));
        println!("discriminant(B(20)) idx = {:?}", discriminant(&Testenum::B(20)));
        println!("discriminant(C(20f32)) idx = {:?}", discriminant(&Testenum::C(20f32)));
        println!("discriminant(C(40f32)) idx = {:?}", discriminant(&Testenum::C(40f32)));

    }
    /// Condvar用来主动阻塞和 唤醒线程
    /// Condvar.wait(guard) 阻塞线程
    /// Condvar.notify_all(guard) 唤醒等待这个条件的线程
    /// 可以用来实现CountDownLatch门闩工具
    #[test]
    fn test_condvar(){
        println!("begin....");
        let cond = std::sync::Condvar::new();
        let b = false;

        let pair: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(b),cond));
        let p2 = pair.clone();
        std::thread::spawn(move ||{
            println!("进入子线程");
            thread::sleep(Duration::from_secs(5));
            let mut guard = p2.0.lock().unwrap();
            *guard = true;
            p2.1.notify_all();
            println!("释放锁");
        });

        
        let mut guard = pair.0.lock().unwrap();
        while !*guard {
            guard = pair.1.wait(guard).unwrap();
            println!("重新获取锁");
        }
        println!("program end... guard:{}", *guard);
    }
    #[test]
    fn test_pattern(){
        let a=(1,2);
        match a{
            (x,y) if x >=1 =>{

            },
            _=>{}
        }
        struct S{
            num: i32
        };
        /// @匹配 只能用于结构体，限定
        /// 成员变量的值，相当于简化if语句
        let s = S{num:10};
        match s{
            S{num: num2 @0..=3} =>{
                println!("num={}",num2);
            }
            _=>{}
        }

        /// 为常量绑定一个值(Rust 1.53 新增)
        match 1 {
            n @ 1=>{}
            n @ 2=>{}
            num @ (3 | 4) => {
                println!("{}", num);
            }
            _ => {}
        }
        ///前绑定后解构(Rust 1.56 新增)
        #[derive(Debug)]
        struct Point {
            x: i32,
            y: i32,
        }
        /// 绑定新变量 `p`，同时对 `Point` 进行解构
        let p @ Point {x: px, y: py } = Point {x: 10, y: 23};
        println!("x: {}, y: {}", px, py);
        println!("{:?}", p);


        let point = Point {x: 10, y: 5};
        if let p @ Point {x: 10, y} = point {
            println!("x is 10 and y is {} in {:?}", y, p);
        } else {
            println!("x was not 10 :(");
        }
    }
}