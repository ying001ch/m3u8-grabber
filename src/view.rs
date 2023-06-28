use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct TaskView{
    pub task_id: String, // task id
    pub err_msg: String, // 错误信息
    pub status: String, // 状态信息
    pub progress: f64, // 进度信息
}