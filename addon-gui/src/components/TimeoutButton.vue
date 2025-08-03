<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
    /** Timeout duration in milliseconds before button can be clicked again */
    timeout?: number
    /** Whether the button is disabled */
    disabled?: boolean
}>()
const emit = defineEmits<{
    click: []
}>()

const isWaiting = ref(false)

function handleClick() {
    if (isWaiting.value || props.disabled) return

    isWaiting.value = true
    emit('click')

    const timeoutDuration = props.timeout ?? 500
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
