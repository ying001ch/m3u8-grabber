<template>
  <div>
    <el-dropdown split-button type="primary" @click="newTask(1)" @command="newTask(2)">
        新建
      <template #dropdown>
          <el-dropdown-menu>
          <el-dropdown-item command="2">合并</el-dropdown-item>
          </el-dropdown-menu>
      </template>
    </el-dropdown>

    <el-button type="info" @click="pause">暂停</el-button>
    <el-button type="success" @click="resumeTask">继续</el-button>
    <el-button type="danger" @click="deleteTask">删除</el-button>
    <!-- 移除分割线 -->
    <!-- <el-divider /> -->

    <!-- 正在进行的任务 -->
    <el-collapse v-model="activeCollapse">
      <el-collapse-item title="进行中" name="1">
        <div class="grid-content ep-bg-purple progress-container" >
          <div v-for="(task) in ongoingTasks" :class="{progress_item:true, sel:(sel_ids.includes(task.task_id))}" 
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
      </el-collapse-item>
      <!-- 已完成的任务 -->
      <el-collapse-item title="已完成" name="2">
        <div class="grid-content ep-bg-purple progress-container" >
          <div v-for="(task) in completedTasks" :class="{progress_item:true, sel:sel_ids.includes(task.task_id)}" 
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
      </el-collapse-item>
    </el-collapse>

    <TaskNew ref="taskNewRef" @submit="refresh_list"></TaskNew>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import TaskNew from './TaskNew.vue';
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessageBox } from 'element-plus'

// 新增响应式变量，控制折叠项展开状态
const activeCollapse = ref(['1']);

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
    // {
    //     task_id: 1,
    //     status: 'Normal',
    //     progress: 0.3,
    //     err_msg: '',
    //     file_name: '12wewwe',
    //     finished: 15,
    //     total: 45,
    // },
]);

// 计算属性：正在进行的任务
const ongoingTasks = computed(() => {
  return tasks.value.filter(task => task.status !== 'End');
});

// 计算属性：已完成的任务
const completedTasks = computed(() => {
  return tasks.value.filter(task => task.status === 'End');
});

const status_transfer = (status) => {
  return map[status];
};

const taskNewRef = ref(null);
const newTask = (task_type) => {
  taskNewRef.value.open(task_type);
};
const refresh_flag = ref(false);
const refresh_list = (form) => {
    if (!refresh_flag.value) {
        refresh_flag.value = true
        setTimeout(get_progress, 1000) 
    }
    console.log("refresh_list:" + form)
};
// 获取进度
const get_progress = (load_db)=>{  
  load_db = load_db || false
  console.log('load_db='+ load_db)

    invoke('get_progress', {loadDb: load_db})
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
const sel_ids = ref([]);
const sel = (task_id)=>{
  const index = sel_ids.value.indexOf(task_id);
  if(index > -1){
    sel_ids.value.splice(index, 1);
  } else {
    sel_ids.value.push(task_id);
  }
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
    msgBox('没有正在运行的任务');
    return;
  }
  if(sel_ids.value.length === 0){
    msgBox('请先选中任务');
    return;
  }
  sel_ids.value.forEach(task_id => {
    console.log('暂停下载：task_id: ' + task_id);
    //传参数要传成驼峰格式
    invoke('pause', {taskHash: task_id})
      .then((response) => {
        msgBox(response);
      }).catch((error) => {
            msgBox(error);
          });
    signal.value = 'pause';
  });
};
const deleteTask = ()=>{
  if(sel_ids.value.length === 0){
    msgBox('请先选中任务');
    return;
  }
  let sel_copy = sel_ids.value;
  invoke('delete_task', {taskHash: sel_ids.value})
      .then((response) => {
        console.log("delete task resp:"+ JSON.stringify(response));
        if (Array.isArray(response)) {
          msgBox("操作成功");
        }else{
          msgBox(response);
        }
        sel_copy.forEach(task_id => {
          const indexToRemove = tasks.value.findIndex(item => item.task_id === task_id);
          tasks.value.splice(indexToRemove, 1);
        })
      }).catch((error) => {
            msgBox(error);
          });
  sel_ids.value = [];
};
const resumeTask = ()=>{
  if(sel_ids.value.length === 0){
    msgBox('请先选中任务');
    return;
  }
  sel_ids.value.forEach(task_id => {
    console.log('继续下载：task_id: ' + task_id);
    invoke('resume', {taskHash: task_id})
      .then((response) => {
        get_progress();
        msgBox(response);
      }).catch((error) => {
            msgBox(error);
          });
    signal.value = 'resume';
  });
};

let unlisten;

onMounted(async () => {
  unlisten = await listen('new-download-task', (event) => {
    console.log('Received new-download-task event:', event.payload);
    taskNewRef.value.open(1, event.payload);
  });
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});

function init(){
  get_progress(true);
}
init()
</script>

<style>
.labal-col{
  text-align: right;
  padding-right: 10px;
  padding-top: 5px;
}
.header {
  font-size: 20px;
  display: inline-block; 
  text-align: center; 
  line-height: 60px;
  vertical-align: middle; 
  width: -webkit-fill-available; 
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

/* 添加新样式，增加折叠组件与上面按钮的距离 */
.el-collapse {
  margin-top: 20px; /* 可以根据实际情况调整这个值 */
}
</style>
