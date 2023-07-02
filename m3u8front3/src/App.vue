<template>
    <el-row :gutter="5">
      <!-- <el-col :span="4">
        <div class="grid-content ep-bg-purple"> &nbsp;</div>
      </el-col> -->
      <el-col :span="12">
        <el-container>
          <el-header><span class="header">M3u8下载器</span></el-header>
          <el-main>
            <div class="grid-content ep-bg-purple">
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> M3u8地址</span>
                </el-col>
                <el-col :span="19" class="labal-col">
                  <el-input placeholder="请输入M3u8地址 http://*.*/*.m3u8" v-model="param.address"  clearable />
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> 保存路径</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="请输入下载路径" v-model="param.save_path" clearable></el-input>
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> 临时目录</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="视频片段的临时存放目录，为空时使用时间戳生成" v-model="param.temp_path" clearable></el-input>
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> 请求头</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="请输入http header,多条使用分号隔开" v-model="param.headers" clearable></el-input>
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> M3u8文件目录</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="本地m3u8文件路径" v-model="param.m3u8_file" clearable/>
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> 解密Key</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="请输入解密Key" v-model="param.key_str" clearable></el-input>
                </el-col>
              </el-row>
              <!-- 全局设置 -->
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> Http代理</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="请输入http代理" v-model="param.proxy" clearable></el-input>
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> 并行任务数</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="下载线程数" v-model.number="param.worker_num" type="number" clearable></el-input>
                </el-col>
              </el-row>
              <el-row :gutter="3">
                <el-col :span="5" class="labal-col">
                  <span class=""> 片段目录</span>
                </el-col>
                <el-col :span="19">
                  <el-input placeholder="输入要合并的视频片段目录" v-model="param.combine_dir" clearable></el-input>
                </el-col>
              </el-row>
              <!-- 合并设置 -->
              只下载不合并<el-switch v-model="param.no_combine" />
            </div>
          </el-main>
          <el-footer>
            <el-button type="primary" @click="submitTask">开始下载</el-button>
            <el-button type="primary" @click="combine">合并片段</el-button>
            <el-button type="primary" @click="pause">暂停下载</el-button>
          </el-footer>
        </el-container>
      </el-col>
      <el-col :span="12">
        <div class="grid-content ep-bg-purple progress-container" >
            <!-- 进度条 -->
            <div v-for="(task,index) in tasks" :class="{progress_item:true, sel:(task.task_id == sel_id)}" 
                @click="sel(task.task_id)">
                <span>{{task.file_name }}</span> 
                <span style="float: right;">{{task.finished }}/{{task.total }}</span>
                <el-progress 
                    :text-inside="true" :stroke-width="30" :percentage="task.progress*100" 
                    :status="status_transfer(task.status)">
                  <span>已完成 {{(task.progress*100).toFixed(2)}}%</span>
                </el-progress>
            </div>
        </div>
      </el-col>
    </el-row>
</template>

<script>
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
        // {
        //   task_id: '12',
        //   err_msg: '',
        //   status: 'Exception',
        //   progress: 0.35,
        //   file_name: '葫芦兄弟.mp4',
        //   finished: '685',
        //   total: '2023',
        // },
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
      invoke('combine', { paramStr: pam })
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
