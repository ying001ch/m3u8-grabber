use std::{future::Future, thread};

use std::sync::OnceLock;
use tokio::{runtime::{Handle, Runtime}, task::JoinHandle};

static RUNTIME: OnceLock<GlobalRuntime> = OnceLock::new();

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
    if let Ok(h) = Handle::try_current(){
        log::warn!("当前线程是 tokio 运行时线程，不能直接执行任务");
        return h.block_on(task);
    }

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
    thread::spawn(|| {
        let runtime = Runtime::new().unwrap();
        let handle = runtime.handle().clone();
        log::info!("tokio 异步运行时已创建");
        GlobalRuntime {
            runtime: runtime,
            handle,
        }
    }).join().unwrap()
}  