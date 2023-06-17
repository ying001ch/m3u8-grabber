<template>
    <el-row :gutter="20">
      <el-col :span="6">
        <div class="grid-content ep-bg-purple"> &nbsp;</div>
      </el-col>
      <el-col :span="12">
        <el-container>
          <el-header>M3u8下载器</el-header>
          <el-main>
            <div class="grid-content ep-bg-purple">
              M3u8地址<el-input placeholder="请输入M3u8地址 http://*.*/*.m3u8" v-model="param.address"  clearable />
              保存路径<el-input placeholder="请输入下载路径" v-model="param.save_path" clearable></el-input>
              Http代理<el-input placeholder="请输入http代理" v-model="param.proxy" clearable></el-input>
              请求头<el-input placeholder="请输入http header,多条使用分号隔开" v-model="param.headers" clearable></el-input>
              
              片段目录<el-input placeholder="输入视频片段目录" v-model="param.combine_dir" clearable></el-input>
              M3u8文件目录<el-input placeholder="本地m3u8文件路径" v-model="param.m3u8_file" clearable></el-input>
              临时目录<el-input placeholder="下载临时目录，为空时使用时间戳生成，下载完删除" v-model="param.temp_path" clearable></el-input>
              解密Key<el-input placeholder="请输入解密Key" v-model="param.key_str" clearable></el-input>
              线程数量<el-input placeholder="下载线程数" v-model.number="param.worker_num" type="number" clearable></el-input>
              只下载不合并<el-switch v-model="param.no_combine" />
            </div>
          </el-main>
          <el-footer>
            <el-button type="primary" @click="sayHello">sayHello</el-button>
            <el-button type="primary" @click="submitTask">开始下载</el-button>
            <el-button type="primary" @click="combine">合并片段</el-button>
          </el-footer>
        </el-container>
      </el-col>
      <el-col :span="6">
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
      param: {
        address: '',
        save_path: './',
        proxy: null,
        headers: null,
        combine_dir: null,
        m3u8_file: null,
        temp_path: null,
        key_str: null,
        worker_num: 2,
        task_type: 1,
        no_combine: false,
      }
    }
  },
  methods:  {
    sayHello : function(event) {
      msgBox('欢迎使用！')
        invoke('greet', { name: 'World' })
          .then((response) => console.log(response))
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
