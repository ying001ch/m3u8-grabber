<template>
    <div class="task_new">
        <el-dialog v-model="dialogShow" title="任务录入" width="600px" @close="resetForm">
            <el-form :model="form" label-width="auto">
                <el-form-item label="地址" >
                    <el-input v-model="form.address" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item label="保存路径" >
                    <el-input v-model="form.save_path" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item label="请求头" >
                    <el-input v-model="form.headers" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item >
                    <el-button type="primary" @click="submitTask">提交</el-button>
                </el-form-item>
            </el-form>
        </el-dialog>
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api'
import { reactive, ref } from 'vue';
import { ElMessageBox } from 'element-plus'

const dialogShow = ref(false);

const form = reactive(
    {
        address:"",
        save_path:"",
        headers:"",
        worker_num:16,
        task_type:1,
        combine_type:1,
        no_combine:true,
    }
);
const open = () => {
    console.log('open');
    dialogShow.value = true;
};
const resetForm = () => {
    console.log('resetForm');
    Object.keys(form).forEach(key => {
        console.log("key:" + key)
        delete form[key];
    });
};
const emit = defineEmits("submit")
const submitTask = () => {
    console.log('submit');
    console.log('submit： '+ JSON.stringify(form));

    if(!form.address || !form.save_path){
        msgBox('地址和保存路径必填')
        return
      }
    //   this.signal = ''
      let that = this
    //   form.task_type = 1;
      let pam = JSON.stringify(form)
      console.log('sub pam: '+pam)
      invoke('submit_task', { paramStr: pam })
        .then((response) => {
          msgBox(response)
           //触发获取进度通知
           emit("submit")
           dialogShow.value = false;
        }).catch((error) => {
          msgBox(error)
        })
};

const combine = (event) => {
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
defineExpose({
    open,
});
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