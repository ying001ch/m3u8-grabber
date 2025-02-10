<template>
  <div>
    <el-button type="primary" @click="newTask()">新建</el-button>
    <el-button type="info" @click="pause">暂停</el-button>
    <el-button type="success" @click="resumeTask">继续</el-button>
    <el-button type="danger" @click="deleteTask">删除</el-button>
    <el-divider />
    <div class="grid-content ep-bg-purple progress-container" >
        <!-- 进度条 -->
        <div v-for="(task) in tasks" :class="{progress_item:true, sel:(task.task_id == sel_id)}" 
            @click="sel(task.task_id)" :key="task.task_id">
            <span>{{task.file_name }}</span> 
            <span style="float: right;">{{task.finished }}/{{task.total }}</span>
            <el-progress 
                :text-inside="true" :stroke-width="30" :percentage="task.progress*100" 
                :status="status_transfer(task.status)">
                <span>已完成 {{(task.progress*100).toFixed(2)}}%</span>
                <span v-if="task.err_msg">Error: {{task.err_msg}}</span>
            </el-progress>
        </div>
    </div>
    <TaskNew ref="taskNewRef" @submit="refresh_list"></TaskNew>
  </div>
</template>

<script setup>
import { ref,reactive } from 'vue';
import TaskNew from './TaskNew.vue';
import { invoke } from '@tauri-apps/api'
import { ElMessageBox } from 'element-plus'

//进度条状态
const success = 'success' 
const exception = 'exception' 
const warning = 'warning' 
//任务状态
const Normal = 'Normal'
const Pause = 'Pause'
const End = 'End'
const Exception = 'Exception'
const map = {
  Normal: '',
  Pause: warning,
  PartFinish: warning,
  End: success,
  Exception: exception,
}
const tasks = ref([
    {
        task_id: 1,
        status: 'Normal',
        progress: 0.3,
        err_msg: '',
        file_name: '12wewwe',
        finished: 15,
        total: 45,
    },
    {
        task_id:2,
        status: 'Exception',
        progress: 0.5,
        err_msg: '',
        file_name: 'dfghsdf',
        finished: 15,
        total: 30,
    }
]);

const status_transfer = (status) => {
  return map[status];
};

const taskNewRef = ref(null);
const newTask = () => {
  taskNewRef.value.open();
};
const refresh_flag = ref(false);
const refresh_list = (form) => {
    if (!refresh_flag.value) {
        refresh_flag.value = true
        setTimeout(get_progress, 1000) 
    }
    console.log("refresh_list:" + form)
};

const get_progress = ()=>{  
    invoke('get_progress', {})
    .then((resp) => {
        console.log('resp='+ JSON.stringify(resp))
        //触发获取进度通知
        listen_progress(resp)
    }).catch((err) => {
        msgBox(err)
    })
};
const listen_progress = (tasks_)=>{
    tasks.value = tasks_;

    let normal_num = tasks_.filter(t=>t.status== Normal).length
    if (normal_num > 0) {
        setTimeout(get_progress, 1000) 
    }else{
        refresh_flag.value = false
    }
};
const sel_id = ref(-1);
const sel = (task_id)=>{
  if(sel_id.value == task_id){
    sel_id.value = -1
    return;
  }
  sel_id.value = task_id
};
function msgBox(msg){
  ElMessageBox.alert(msg, {
          // if you want to disable its autofocus
          // autofocus: false,
          confirmButtonText: 'OK',
          callback: (action) => {
            console.log('点击确认')
          },
        })
}
const signal = ref('');
const pause = ()=>{
  if(tasks.value.length <= 0){
    msgBox('没有正在运行的任务')
    return;
  }
  if(sel_id.value < 0){
    msgBox('请先选中任务')
    return;
  }
  console.log('暂停下载：task_id: ' + sel_id.value)
  //传参数要传成驼峰格式
  invoke('pause', {taskHash: sel_id.value})
    .then((response) => {
      msgBox(response)
    })
    signal.value = 'pause'
};
const deleteTask = ()=>{
  if(sel_id.value < 0){
    msgBox('请先选中任务')
    return;
  }
  const indexToRemove = tasks.value.findIndex(item => item.task_id === sel_id.value);

  if (indexToRemove !== -1) {
    tasks.value.splice(indexToRemove, 1);
  }
}
const resumeTask = ()=>{
  if(sel_id.value < 0){
    msgBox('请先选中任务')
    return;
  }
  //--------------------
  console.log('继续下载：task_id: ' + sel_id.value)
  invoke('resume', {taskHash: sel_id.value})
    .then((response) => {
      msgBox(response)
    })
    signal.value = 'resume'
}
</script>


<style>
.labal-col{
  text-align: right;
  padding-right: 10px;
  padding-top: 5px;
}
.header {
  font-size: 20px;
  display: inline-block; /* 将 span 元素转换为块级元素 */
  text-align: center; /* 实现水平居中 */
  line-height: 60px;/* 等于父元素高度 */
  vertical-align: middle; /* 实现垂直居中 */
  width: -webkit-fill-available; /* 自动填充宽度 */
  padding-top: 10px;
}
.el-row {
  margin-bottom: 20px;
}
.el-row:last-child {
  margin-bottom: 0;
}
.el-col {
  border-radius: 4px;
}

.grid-content {
  border-radius: 4px;
  min-height: 36px;
}
.progress-container{
  /* padding-top: 85px; */
  position: relative;
  height: 100%;
}
.progress_item{
  padding-bottom: 5px;
  padding-top: 5px;
  border-bottom: 1px solid rgb(179, 170, 170);
}
.progress_item:hover{
  background-color:rgba(195, 232, 254, 0.869);
}
.sel{
  background-color:rgba(195, 232, 254, 0.869);
}
</style>