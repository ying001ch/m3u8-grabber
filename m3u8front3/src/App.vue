<template>
  <div class="main-page">
    <el-tabs :tab-position="'left'" style="" class="demo-tabs">
      <el-tab-pane label="download">
          <Download/>
      </el-tab-pane>
      <el-tab-pane label="settings">
          <Settings/>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api'
import { ElMessageBox } from 'element-plus'
import { ref } from 'vue';
import Download from './components/Download.vue';
import Settings from './components/Settings.vue';


const dialogRef = ref(null);
const addressList = ref([]);

// 打开弹窗
const openDialog = () => {
  dialogRef.value.open();
};

// 处理提交数据
const handleSubmit = (formData) => {
  addressList.value.push(formData);
};

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
let refresh_flag = false // 刷新标识
export default {
  name: 'App',
  data () {
    return {
      sel_id: -1, //选中的索引
      param: {
        address: '',
        save_path: './',
        proxy: null,
        headers: null,
        combine_dir: null,
        m3u8_file: null,
        temp_path: null,
        key_str: null,
        worker_num: 80,
        task_type: 1,
        no_combine: false,
        combine_type: 1,
        signal: ''
      },
      tasks: [
        // {
        //   task_id: '123',
        //   err_msg: '',
        //   status: 'Exception',
        //   progress: 0.35,
        //   file_name: 'huluwa.mp4',
        //   finished: '685',
        //   total: '2023',
        // },
      ],
      options: [
        {
          value: 1,
          label: '二进制合并',
        },
        {
          value: 2,
          label: 'FFMPEG合并',
        },
      ]
    }
  },
  methods:  {
    sel: function(task_id){
      if(this.sel_id == task_id){
        this.sel_id = -1
        return;
      }
      this.sel_id = task_id
    },
    status_transfer: function(task_status){
      return map[task_status]
    },
    pause: function(){
      if(this.tasks.length <= 0){
        msgBox('没有正在运行的任务')
        return;
      }
      if(this.sel_id < 0){
        msgBox('请先选中任务')
        return;
      }
      console.log('暂停下载：task_id: ' + this.sel_id)
      //传参数要传成驼峰格式
      invoke('pause', {taskHash: this.sel_id})
        .then((response) => {
          msgBox(response)
        })
        this.signal = 'pause'
    },
    listen_progress: function(tasks_){
      this.tasks = tasks_;

      let normal_num = tasks_.filter(t=>t.status== Normal).length
      if (normal_num > 0) {
        setTimeout(this.get_progress, 1000) 
      }else{
        refresh_flag = false
      }
    },
    submitTask: function (event) {
      if(!this.param.address || !this.param.save_path){
        msgBox('地址和保存路径必填')
        return
      }
      this.signal = ''
      let that = this
      this.param.task_type = 1;
      let pam = JSON.stringify(this.param)
      console.log('sub pam: '+pam)
      invoke('submit_task', { paramStr: pam })
        .then((response) => {
          msgBox(response)
           //触发获取进度通知
           if (!refresh_flag) {
              refresh_flag = true
              setTimeout(that.get_progress, 1000) 
           }
        }).catch((error) => {
          msgBox(error)
        })
    },
    // 获取进度通知
    get_progress: function(){  
      let that = this
      invoke('get_progress', {})
        .then((resp) => {
          console.log('resp='+ JSON.stringify(resp))
          //触发获取进度通知
          that.listen_progress(resp)
        }).catch((err) => {
          msgBox(err)
        })
    },
    combine : function(event) {
      if(!this.param.combine_dir || !this.param.save_path){
        msgBox('片段目录和保存路径必填')
        return
      }
      this.param.task_type = 2;
      let pam = JSON.stringify(this.param)
      console.log('sub pam: '+pam)
      invoke('combine_cmd', { paramStr: pam })
        .then((response) => msgBox(response))
    }
  }
}
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
  padding-top: 85px;
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
