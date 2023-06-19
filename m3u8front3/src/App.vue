<template>
    <el-row :gutter="5">
      <el-col :span="4">
        <div class="grid-content ep-bg-purple"> &nbsp;</div>
      </el-col>
      <el-col :span="16">
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
                  <span class=""> 线程数量</span>
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
            <!-- 进度条 -->
            <!-- <el-progress :text-inside="true" :stroke-width="30" :percentage="percentage" :status="progress_status">
              <span>title mp4  {{percentage}}%</span>
            </el-progress>
            <el-progress type="circle" :percentage="percentage" :status="progress_status"/> -->
          </el-main>
          <el-footer>
            <el-button type="primary" @click="submitTask">开始下载</el-button>
            <el-button type="primary" @click="combine">合并片段</el-button>
            <el-button type="primary" @click="pause">暂停下载</el-button>
          </el-footer>
        </el-container>
      </el-col>
      <el-col :span="4">
        <div class="grid-content ep-bg-purple"></div>
      </el-col>
    </el-row>
</template>

<script>
import { invoke } from '@tauri-apps/api'
import { ElMessageBox } from 'element-plus'
export default {
  name: 'App',
  data () {
    return {
      percentage: 30,
      progress_status: '',
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
      }
    }
  },
  methods:  {
    pause: function(){
      console.log('暂停下载')
      invoke('pause', {})
        .then((response) => {
          msgBox(response)
        })
    },
    submitUpload: function (evn){
      console.log('文件提交 evn:' + evn)
    },
    handleExceed: function(files){
      console.log('获取文件 files: ' + files);
      files.forEach(f=>console.log('f='+ f))
    },
    decrease: function (){
      this.percentage -= 10
      if(this.percentage <= 0){
        this.percentage = 0
      }
      if(this.percentage < 100){
        this.progress_status = ''
      }
    },
    increase: function(){
      this.percentage += 10
      if(this.percentage >=100){
        this.percentage = 100
        this.progress_status = 'success'
      }
    },
    submitTask: function (event) {
      if(!this.param.address || !this.param.save_path){
        msgBox('地址和保存路径必填')
        return
      }
      let that = this.param
      that.task_type = 1
      let pam = JSON.stringify(this.param)
      console.log('sub pam: '+pam)
      invoke('submit_task', { paramStr: pam })
        .then((response) => {
          msgBox(response)
        })
    },
    combine : function(event) {
      if(!this.param.combine_dir || !this.param.save_path){
        msgBox('片段目录和保存路径必填')
        return
      }
      let that = this.param
      that.task_type = 2
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
</style>
