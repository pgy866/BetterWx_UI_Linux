<template>
    <div class="flex row justify-between items-center">
        <div class="flex row justify-center">
            <b class="m-r">{{ isMain ? '主程序' : '共存-' + data.id }}</b>
            <div v-if="!revoke_unsupport" class="flex items-center gap  m-x">
                <label for="revoke">撤回</label>
                <ToggleSwitch inputId="revoke" v-model="revoke" @change="switch_change" :disables="revoke_unsupport"/>
            </div>
            <div v-if="isMain && !unlock_unsupport" class=" flex items-center gap m-x">
                <label for="unlock">双开</label>
                <ToggleSwitch inputId="unlock" v-model="unlock" @change="switch_change"  :disables="unlock_unsupport"/>
            </div>
        </div>
        <div class="flex items-center gap">
            <Button v-if="isMain" label="刷新" @click="refresh" size="small" />
            <Button v-if="isMain" label="目录" @click="loc" size="small" />
            <Button label="打开" @click="open_app" size="small" />
            <Button v-if="isMain && !unlock_unsupport" label="共存" @click="add" size="small" />
            <Button v-if="!isMain" label="删除" @click="del" size="small" severity="danger" />
        </div>
    </div>
</template>

<script setup>
import { ref, watchEffect, defineEmits } from 'vue'

const props = defineProps({
    isMain: { type: Boolean, default: false },
    data: { type: Object, default: {} },
    index: { type: [String, Number], default: 0, required: true },
});

const unlock = ref(false)
const revoke = ref(false)
const unlock_unsupport  = ref(false)
const revoke_unsupport  = ref(false)

watchEffect(() => {
    let patch_status = props.data.patch_status
    console.log("patch_status",patch_status);
    patch_status.forEach(item => {
        if(item.name == "unlock"){
            unlock.value = item.status
            unlock_unsupport.value = item.unsupport
        }
        if(item.name == "revoke"){
            revoke.value = item.status
            revoke_unsupport.value = item.unsupport
        }
    });
})

const emit = defineEmits(["switch_change", "open_app", "add", "loc", "del", "refresh"])


function switch_change() {
    emit('switch_change', { unlock: unlock.value, revoke: revoke.value, index: props.index })
}

function open_app() {
    emit('open_app', props.data.exe_file)
}
function refresh() {
    emit('refresh')
}
function add() {
    emit('add')
}
function loc() {
    emit('loc')
}
function del() {
    if (props.num !== -1) {
        emit('del', { index: props.index })
    }
}
</script>