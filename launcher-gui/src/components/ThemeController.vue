<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
const THEME_INTERVAL_IN_MILLISECONDS = 5000

const themeList = [
    'default',
    'abyss',
    'acid',
    'aqua',
    'autumn',
    'black',
    'bumblebee',
    'business',
    'caramellatte',
    'cmyk',
    'coffee',
    'corporate',
    'cupcake',
    'cyberpunk',
    'dark',
    'dim',
    'dracula',
    'emerald',
    'fantasy',
    'forest',
    'garden',
    'halloween',
    'lemonade',
    'light',
    'lofi',
    'luxury',
    'night',
    'nord',
    'pastel',
    'retro',
    'silk',
    'sunset',
    'synthwave',
    'valentine',
    'winter',
    'wireframe',
]

const currentThemeIndex = ref(0)
const currentTheme = computed(() => themeList[currentThemeIndex.value])

function nextTheme() {
    currentThemeIndex.value = (currentThemeIndex.value + 1) % themeList.length
}

function prevTheme() {
    currentThemeIndex.value =
        (currentThemeIndex.value - 1 + themeList.length) % themeList.length
}

watch(currentTheme, (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme)
})

// Timer to increment the theme index
let themeTimer

onMounted(() => {
    //   themeTimer = setInterval(() => {
    // nextTheme();
    //   }, THEME_INTERVAL_IN_MILLISECONDS);
})

onUnmounted(() => {
    clearInterval(themeTimer)
})
</script>

<template>
    <div class="p-1 flex flex-col text-center font-mono text-sm">
        <div>
            {{ currentTheme }} {{ currentThemeIndex + 1 }}/{{
                themeList.length
            }}
        </div>
        <div class="flex justify-center">
            <button class="btn mx-1" @click="prevTheme"><</button>
            <button class="btn mx-1" @click="nextTheme">></button>
        </div>
    </div>
</template>
