<template>
    <el-row :gutter="20">
      <el-col :span="6">
        <div class="grid-content ep-bg-purple"> &nbsp;</div>
      </el-col>
      <el-col :span="12">
        <el-container>
          <el-header>M3u8下载器<el-alert title="success alert" type="success" /></el-header>
          <el-main>
            <div class="grid-content ep-bg-purple">
              <el-input placeholder="请输入M3u8地址 http://*.*/*.m3u8" v-model="param.address"  clearable />
              <el-input placeholder="请输入下载路径" v-model="param.savePath" clearable></el-input>
              <el-input placeholder="请输入http代理" v-model="param.proxy" clearable></el-input>
              <el-input placeholder="请输入http header,多条使用分号隔开" v-model="param.headers" clearable></el-input>
              
              <el-input placeholder="输入视频片段目录" v-model="param.clipDir" clearable></el-input>
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
        savePath: './',
        proxy: '',
        headers: '',
        clipDir: '',
      }
    }
  },
  methods:  {
    sayHello : function(event) {
      msgBox('欢迎使用！')
        invoke('greet', { name: 'World' })
          // `invoke` returns a Promise
          .then((response) => console.log(response))
    },
    submitTask: function (event) {
      if(!this.param.address || !this.param.savePath){
        msgBox('地址和保存路径必填')
        return
      }

      let pam = JSON.stringify(this.param)
      invoke('submit_task', { paramStr: pam })
        // `invoke` returns a Promise
        .then((response) => console.log(response))
    },
    combine : function(event) {
      if(!this.param.clipDir || !this.param.savePath){
        msgBox('片段目录和保存路径必填')
        return
      }
      let pam = JSON.stringify(this.param)
        invoke('combine', { paramStr: pam })
          // `invoke` returns a Promise
          .then((response) => console.log(response))
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
