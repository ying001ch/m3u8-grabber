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
              <el-input placeholder="请输入M3u8地址 http://*.*/*.m3u8" v-model="address"  clearable />
              <el-input placeholder="请输入下载路径" v-model="savePath" clearable></el-input>
              <el-input placeholder="请输入http代理" v-model="proxy" clearable></el-input>
              <el-input placeholder="请输入http header,多条使用分号隔开" v-model="headers" clearable></el-input>
            </div>
          </el-main>
          <el-footer><el-button type="primary" @click="say">开始下载</el-button></el-footer>
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
      count: 0,
      address: '',
      savePath: './',
      proxy: '',
      headers: ''
    }
  },
  methods:  {
    say : function(event) {
      ElMessageBox.alert('This is a message', 'Title', {
        // if you want to disable its autofocus
        // autofocus: false,
        confirmButtonText: 'OK',
        callback: (action) => {
          console.log('点击确认')
        },
      })
        invoke('greet', { name: 'World' })
          // `invoke` returns a Promise
          .then((response) => console.log(response))
    }
  }
}
export let email = '17@qq.com'
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
