<template>
    <div id="settins">
        <el-form :model="settings" label-width="auto">
            <el-form-item label="代理" >
                <el-input v-model="settings.proxy"></el-input>
            </el-form-item>
            <el-form-item label="并发数" >
                <el-input-number  v-model="settings.work_num" :min="1" :max="100"/>
            </el-form-item>
            <el-form-item label="合并方式" >
                <el-select v-model="settings.combine_type" placeholder="Select"
                      size="large"
                      style="width: 240px"
                    >
                      <el-option  v-for="item in options"
                          :key="item.value"
                          :label="item.label"
                          :value="item.value"/>
                  </el-select>
            </el-form-item>
            <el-form-item >
                <el-button type="primary" @click="saveSettings">保存</el-button>
            </el-form-item>
        </el-form>
    </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api'
import { ElMessageBox } from 'element-plus'

const settings = reactive({
    proxy: '',
    work_num: 16,
    combine_type: 1,
})
const options = [
    {
        value: 1,
        label: '二进制合并',
    },
    {
        value: 2,
        label: 'FFMPEG合并',
    },
]

const saveSettings = () => {
    console.log("saveSettings: "+ JSON.stringify(settings))

    invoke('save_settings', {"config":{...settings}})
        .then((response) => {
            msgBox(response)
        })
}


//-----
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