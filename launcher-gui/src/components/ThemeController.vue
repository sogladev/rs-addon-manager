<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AddOnsUserConfig } from '@bindings/AddOnsUserConfig'

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

// persist whenever theme changes
watch(currentTheme, async (newTheme) => {
  document.documentElement.setAttribute('data-theme', newTheme)
  try {
    await invoke('save_theme', newTheme)
  } catch (e) {
    console.error('Failed to save theme:', e)
  }
})

// load saved theme on mount
onMounted(async () => {
  try {
    const config = await invoke<AddOnsUserConfig>('load_user_config')
    if (config.theme) {
      const idx = themeList.indexOf(config.theme)
      if (idx >= 0) currentThemeIndex.value = idx
    }
  } catch (e) {
    console.error('Failed to load saved theme:', e)
  }
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
