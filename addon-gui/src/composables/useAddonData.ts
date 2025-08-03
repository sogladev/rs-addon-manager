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
    const installStatus = ref<{
        progress?: { current: number; total: number }
        step?: string
        error?: string
        warning?: string
        active: boolean
    }>({ active: false })

    const folderPaths = computed(() => addonFolders.value.map((f) => f.path))

    async function refreshAddonData() {
        try {
            const folders = await invoke<AddOnsFolder[]>('refresh_addon_data')
            addonFolders.value = folders
        } catch (err) {
            console.error('Failed to refresh addon data:', err)
        }
    }

    onMounted(async () => {
        // Listen for install events
        listen<InstallEventPayload>('install-event', ({ payload }) => {
            console.debug('[install-event]', payload)
            installStatus.value.active = true

            const event = payload.event
            if ('Progress' in event) {
                const { current, total } = event.Progress
                installStatus.value.progress = { current, total }
                installStatus.value.step = undefined
                installStatus.value.error = undefined
                installStatus.value.warning = undefined
            } else if ('Status' in event) {
                installStatus.value.step = event.Status
                installStatus.value.progress = undefined
                installStatus.value.error = undefined
                installStatus.value.warning = undefined
            } else if ('Warning' in event) {
                installStatus.value.warning = event.Warning
                installStatus.value.error = undefined
            } else if ('Error' in event) {
                installStatus.value.error = event.Error
                installStatus.value.warning = undefined
            } else {
                console.warn('[install-event] Unknown event type:', payload)
            }
        })

        // Listen for addon data updates
        listen('addon-data-updated', refreshAddonData)

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
        installStatus,
        refreshAddonData,
    }
}
