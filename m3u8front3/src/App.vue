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

