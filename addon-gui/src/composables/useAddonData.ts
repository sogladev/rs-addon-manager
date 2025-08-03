import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { AddOnsFolder } from '@bindings/AddOnsFolder'

type InstallKey = { path: string; url: string }
type InstallEventPayload = {
    key: InstallKey
    event:
        | { Progress: { current: number; total: number } }
        | { Status: string }
        | { Warning: string }
        | { Error: string }
}

export function useAddonData() {
    const addonFolders = ref<AddOnsFolder[]>([])

    const folderPaths = computed(() => addonFolders.value.map((f) => f.path))

    async function refreshAddonData() {
        try {
            const folders = await invoke<AddOnsFolder[]>('refresh_addon_data')
            addonFolders.value = folders
        } catch (err) {
            console.error('Failed to refresh addon data:', err)
        }
    }

    async function refreshDiskData() {
        try {
            const folders = await invoke<AddOnsFolder[]>('refresh_disk_data')
            addonFolders.value = folders
        } catch (err) {
            console.error('Failed to refresh disk data:', err)
        }
    }

    onMounted(async () => {
        // Listen for install events (for potential future use)
        listen<InstallEventPayload>('install-event', ({ payload }) => {
            console.debug('[install-event]', payload)
            // Install events are currently logged for debugging
            // Future enhancement: Could display progress notifications
        })

        // Listen for addon data updates
        listen('addon-data-updated', refreshAddonData)

        // Listen for addon disk updates (fast, disk-only refresh)
        listen('addon-disk-updated', refreshDiskData)

        // Listen for update-all completion
        listen<string>('update-all-complete', ({ payload }) => {
            console.log('Update all completed:', payload)
            // You can add toast notifications here later
        })

        // Load initial addon data from backend
        await refreshAddonData()
    })

    return {
        addonFolders,
        folderPaths,
        refreshAddonData,
        refreshDiskData,
    }
}
