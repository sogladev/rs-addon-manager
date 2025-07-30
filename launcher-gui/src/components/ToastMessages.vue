<script setup lang="ts">
import { ref } from 'vue'

interface Message {
    type: 'success' | 'error'
    message: string
}

const messages = ref<Message[]>([])

/**
 * Show a toast message and remove it after a delay.
 */
function showToast(
    type: 'success' | 'error',
    message: string,
    duration = 3000
) {
    messages.value.push({ type, message })
    setTimeout(() => {
        messages.value.shift()
    }, duration)
}

// Expose the showToast function to parent
defineExpose({ showToast })
</script>

<template>
    <div class="toast toast-center toast-middle">
        <div
            v-for="(msg, index) in messages"
            :key="index"
            :class="[
                'alert',
                msg.type === 'error' ? 'alert-error' : 'alert-success',
            ]"
        >
            <span>{{ msg.message }}</span>
        </div>
    </div>
</template>
