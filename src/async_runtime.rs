use std::future::Future;

use once_cell::sync::OnceCell;
use tokio::{runtime::{Handle, Runtime}, task::JoinHandle};

static RUNTIME: OnceCell<GlobalRuntime> = OnceCell::new();

struct GlobalRuntime {
    runtime: Runtime,
    handle: Handle,
}

impl GlobalRuntime {
    fn block_on<F: Future>(&self, task: F) -> F::Output {
        self.runtime.block_on(task)
    }
}

pub fn block_on<F: Future>(task: F) -> F::Output {
    let runtime = RUNTIME.get_or_init(default_runtime);
    runtime.block_on(task)
}
pub fn spawn<F: Future>(task: F) -> JoinHandle<F::Output> 
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let runtime = RUNTIME.get_or_init(default_runtime);
    runtime.runtime.spawn(task)
}

fn default_runtime() -> GlobalRuntime {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    println!("运行时已经创建");
    GlobalRuntime {
        runtime: runtime,
        handle,
    }
}  