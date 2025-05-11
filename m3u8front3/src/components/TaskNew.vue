<template>
    <div class="task_new">
        <el-dialog v-model="dialogShow" title="任务录入" width="600px" @close="resetForm">
            <el-form :model="form" label-width="auto" v-if="task_type==1" ref="formRef" @keyup.enter="submitTask">
                <el-form-item label="地址" >
                    <el-input v-model="form.address" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item label="保存路径" >
                    <el-input v-model="form.save_path" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item label="请求头" >
                    <el-input v-model="form.headers" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item label="解密key" >
                    <el-input placeholder="请输入key的16进制字符串" v-model="form.key_str" clearable></el-input>
                </el-form-item>
                <el-form-item label="不合并" >
                  <el-switch v-model="form.no_combine" />
                </el-form-item>
                <el-form-item >
                    <el-button type="primary" @click="submitTask">提交</el-button>
                </el-form-item>
            </el-form>

            <el-form :model="form" label-width="auto" v-if="task_type==2" ref="formRef" @keyup.enter="submitTask">
              <el-form-item label="片段目录" >
                  <el-input placeholder="输入要合并的视频片段目录" v-model="form.combine_dir" clearable></el-input>
              </el-form-item>
                <el-form-item label="保存路径" >
                    <el-input v-model="form.save_path" autocomplete="off"></el-input>
                </el-form-item>
                <el-form-item >
                    <el-button type="primary" @click="submitTask">提交</el-button>
                </el-form-item>
            </el-form>
        </el-dialog>
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { reactive, ref } from 'vue';
import { ElMessageBox } from 'element-plus'

const dialogShow = ref(false);
const task_type = ref(1);
const formRef = ref(null);

const form = reactive(
    {
        address:"",
        save_path:"",
        combine_dir: null,
        headers: null,
        key_str: null,
        task_type:1,
        no_combine:false,
    }
);
const open = (task_type_) => {
    console.log('open:'+task_type);
    task_type.value = task_type_
    dialogShow.value = true;
};
const resetForm = () => {
    console.log('resetForm');
    Object.keys(form).forEach(key => {
        delete form[key];
    });
    form.address="";
};
const emit = defineEmits("submit")
const submitTask = () => {
    console.log('submit');
    console.log('submit： '+ JSON.stringify(form));

    // 让表单内的所有输入框脱离焦点
    const inputs = formRef.value.$el.querySelectorAll('input');
    inputs.forEach(input => {
        input.blur();
        console.log('input blur :'+input.value)
    });

    if(!form.save_path){
        msgBox('地址和保存路径必填')
        return
      }
    if(!form.address && task_type.value==1){
        msgBox('地址和保存路径必填')
        return
      }
    //   this.signal = ''
      form.task_type = task_type.value;
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
        .catch((error) => {
          msgBox(error)
        })
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