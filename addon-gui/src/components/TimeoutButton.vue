<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
    timeout?: number
    disabled?: boolean
}>()
const emit = defineEmits<{
    click: []
}>()

const isWaiting = ref(false)
const timeout_in_ms = props.timeout ?? 500

function handleClick() {
    if (isWaiting.value || props.disabled) return
    isWaiting.value = true
    emit('click')
    setTimeout(() => {
        isWaiting.value = false
    }, timeout_in_ms)
}
</script>

<template>
    <button :disabled="isWaiting || disabled" @click="handleClick" class="btn">
        <slot />
    </button>
</template>
