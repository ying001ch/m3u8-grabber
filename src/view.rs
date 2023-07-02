use serde::{Deserialize, Serialize};

use crate::config::Signal;

#[derive(Serialize, Default)]
pub struct TaskView{
    pub task_id: String, // task id
    pub err_msg: String, // 错误信息
    pub status: Signal, // 状态信息
    pub progress: f64, // 进度信息
    pub file_name: String, // 文件名
    pub finished: usize, // 已完成
    pub total: usize, // 总数量
}