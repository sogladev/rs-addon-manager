<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const THEME_INTERVAL_IN_MILLISECONDS = 5000

// Disable some due to too many choices
const themeList = [
    'default',
    // 'abyss',
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
    // 'emerald',
    // 'fantasy',
    'forest',
    'garden',
    'halloween',
    'lemonade',
    'light',
    // 'lofi',
    'luxury',
    'night',
    'nord',
    'pastel',
    'retro',
    // 'silk',
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

watch(currentTheme, async (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme)
    try {
        await invoke('save_theme', newTheme)
    } catch (e) {
        console.error('Failed to save theme:', e)
    }
})

onMounted(async () => {
    try {
        const theme = (await invoke) < String > 'load_theme'
        if (theme) {
            const idx = themeList.indexOf(theme)
            if (idx >= 0) currentThemeIndex.value = idx
        }
    } catch (e) {
        console.warning('Failed to load saved theme:', e)
    }
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
            <button class="btn mx-1" @click="prevTheme">&lt;</button>
            <button class="btn mx-1" @click="nextTheme">></button>
        </div>
    </div>
</template>
