<script setup lang="ts">
import { ref } from 'vue'

const { timeout, disabled } = defineProps<{
    timeout?: number
    disabled?: boolean
}>()

const emit = defineEmits<{
    click: []
}>()

const isWaiting = ref(false)

const handleClick = () => {
    if (isWaiting.value || disabled) return

    isWaiting.value = true
    emit('click')

    const timeoutDuration = timeout ?? 500
    setTimeout(() => {
        isWaiting.value = false
    }, timeoutDuration)
}
</script>

<template>
    <button :disabled="isWaiting || disabled" @click="handleClick" class="btn">
        <slot />
    </button>
</template>
