<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

// Reset theme to default
async function resetTheme() {
    currentThemeIndex.value = 0
    document.documentElement.setAttribute('data-theme', themeList[0])
    try {
        await invoke('save_theme', { theme: themeList[0] })
    } catch (e) {
        console.error('Failed to reset theme:', e)
    }
}

watch(currentTheme, async (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme)
    try {
        await invoke('save_theme', { theme: newTheme })
    } catch (e) {
        console.error('Failed to save theme:', e)
    }
})

onMounted(async () => {
    try {
        const theme = await invoke('load_theme')
        if (theme) {
            document.documentElement.setAttribute('data-theme', theme)
            const idx = themeList.indexOf(theme)
            if (idx >= 0) currentThemeIndex.value = idx
        }
    } catch (e) {
        console.warn('Failed to load saved theme:', e)
    }
})
</script>

<template>
    <div class="p-2 flex flex-col items-center font-mono text-sm gap-2">
        <div>
            {{ currentTheme }} {{ currentThemeIndex + 1 }}/{{
                themeList.length
            }}
        </div>
        <div class="flex justify-center gap-2">
            <button class="btn btn-sm" @click="prevTheme">&lt;</button>
            <button class="btn btn-sm" @click="nextTheme">&gt;</button>
        </div>
        <button
            class="btn btn-sm btn-outline mt-2"
            @click="resetTheme"
            aria-label="Reset to default theme"
        >
            Default
        </button>
    </div>
</template>
