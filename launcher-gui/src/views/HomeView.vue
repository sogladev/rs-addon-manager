<script setup lang="ts">
import MainLayout from '@/layouts/MainLayout.vue'
import TopBar from '@/components/TopBar.vue'
import BottomBar from '@/components/BottomBar.vue'
import MainContent from '@/components/MainContent.vue'
</script>

<template>
    <div class="background-image"></div>
    <MainLayout class="bg-gradient-to-r from-base-100/50 to-base-300/80">
        <TopBar class="bg-base-200/25" />
        <MainContent class="bg-base-100/50" />
        <BottomBar />
    </MainLayout>
</template>

<style scoped>
.background-image {
    background-image: url('@/assets/images/background-0.jpg');
    background-size: cover;
    background-position: center;
    filter: blur(1px);
    height: 100%;
    width: 100%;
    position: absolute;
    top: 0;
    left: 0;
    z-index: -1;
}
</style>
