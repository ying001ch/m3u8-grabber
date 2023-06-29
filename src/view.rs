use serde::{Deserialize, Serialize};

use crate::config::Signal;

#[derive(Serialize, Default)]
pub struct TaskView{
    pub task_id: String, // task id
    pub err_msg: String, // 错误信息
    pub status: Signal, // 状态信息
    pub progress: f64, // 进度信息
}