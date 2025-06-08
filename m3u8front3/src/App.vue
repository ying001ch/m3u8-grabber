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
import { ElMessageBox } from 'element-plus'
import { ref, computed,onMounted } from 'vue';
import Download from './components/Download.vue';
import Settings from './components/Settings.vue';
import { TrayIcon } from '@tauri-apps/api/tray';
import { defaultWindowIcon } from '@tauri-apps/api/app';
import { Menu } from '@tauri-apps/api/menu';
import { getCurrentWindow } from '@tauri-apps/api/window';

async function addTrayIcon(){
  const menu = await Menu.new({
    items: [
      {
        id: 'show',
        text: 'Show',
        action: async() => {
          await getCurrentWindow().show();
          console.log('show pressed');
        },
      },
      {
        id: 'quit',
        text: 'Quit',
        action: async() => {
          await getCurrentWindow().destroy()
          console.log('quit pressed');
        },
      },
    ],
  });

  const icon = await defaultWindowIcon();
  console.log('icon=',icon)
  const options = {
    icon: icon || undefined,  // Convert null to undefined if icon is null
    menu,
    menuOnLeftClick: true,
    tooltip: "m3u8-grabber",
  };

  const trayId = sessionStorage.getItem('tray')
  console.log('old trayId=',trayId)
  if(!trayId){
      const tray = await TrayIcon.new(options)
      sessionStorage.setItem('tray', tray.id);

    }
    const currentWindow = getCurrentWindow();
    currentWindow.listen('tauri://close-requested',()=>{
      console.log('close pressed')
      currentWindow.hide();
    })
}


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

onMounted(async () => {
  addTrayIcon().then(()=>{
    console.log('add tray icon success')
  });
})

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

