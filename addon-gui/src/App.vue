<script setup lang="ts">
import AddonManager from '@/views/AddonManager.vue'
import { onMounted } from 'vue'

onMounted(() => {
    document.documentElement.setAttribute('data-theme', 'default')
})
</script>

<template>
    <AddonManager />
</template>
