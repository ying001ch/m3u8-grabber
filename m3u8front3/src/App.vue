<template>
  <div class="main-page">
    <el-menu
      :default-active="$route.path"
      mode="vertical"
      router
      class="side-menu"
    >
      <el-menu-item index="/download">
        <span>download</span>
      </el-menu-item>
      <el-menu-item index="/settings">
        <span>settings</span>
      </el-menu-item>
    </el-menu>

    <div class="content-area">
      <router-view />
    </div>
  </div>
</template>

<script setup>
import { ElMessageBox } from 'element-plus'
import { ref, computed,onMounted } from 'vue';
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

<style scoped>
.main-page {
  height: 100vh;
  display: flex;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

.side-menu {
  width: 120px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-right: 1px solid rgba(0, 0, 0, 0.08);
  box-shadow: 2px 0 20px rgba(0, 0, 0, 0.05);
}

.side-menu :deep(.el-menu-item) {
  margin: 8px 12px;
  border-radius: 8px;
  transition: all 0.3s ease;
  color: #606266;
  font-weight: 500;
}

.side-menu :deep(.el-menu-item:hover) {
  background: linear-gradient(135deg, #f0f2ff 0%, #e8ebff 100%);
  color: #667eea;
  transform: translateX(3px);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.15);
  border-left: 3px solid #667eea;
}

.side-menu :deep(.el-menu-item.is-active) {
  background: linear-gradient(135deg, #8b9aff 0%, #7c8fff 100%);
  color: white;
  transform: translateX(3px);
  box-shadow: 0 4px 20px rgba(139, 154, 255, 0.4);
  border-left: 3px solid #5a67ff;
}

.side-menu :deep(.el-menu-item span) {
  font-size: 14px;
}

.content-area {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  background: rgba(255, 255, 255, 0.9);
  margin: 16px;
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
}
</style>

<style>
/* 全局样式：确保弹窗在最上层 */
.el-overlay {
  z-index: 9998 !important;
}

.el-dialog {
  z-index: 9999 !important;
}

.el-dialog__wrapper {
  z-index: 9999 !important;
}

/* 确保弹窗不会被其他元素遮挡 */
.el-dialog__header, .el-dialog__body, .el-dialog__footer {
  z-index: 10000 !important;
}
</style>

