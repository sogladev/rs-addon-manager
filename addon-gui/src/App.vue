<script setup lang="ts">
import HomeView from '@/views/HomeView.vue'
import { onMounted } from 'vue'

onMounted(() => {
    document.documentElement.setAttribute('data-theme', 'default')
})
</script>

<template>
    <HomeView />
</template>
