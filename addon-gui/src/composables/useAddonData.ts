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

    let refreshPending = false
    let lastRefreshTime = 0
    let scheduledRefresh: ReturnType<typeof setTimeout> | null = null
    let diskRefreshPending = false
    let lastDiskRefreshTime = 0
    let scheduledDiskRefresh: ReturnType<typeof setTimeout> | null = null

    async function refreshAddonData(force = false) {
        const now = Date.now()
        const timeSinceLast = now - lastRefreshTime
        const refreshInterval = 10000 // 10 seconds

        if (refreshPending) {
            return
        }

        if (!force && timeSinceLast < refreshInterval) {
            if (scheduledRefresh) clearTimeout(scheduledRefresh)
            scheduledRefresh = setTimeout(() => {
                scheduledRefresh = null
                refreshAddonData()
            }, refreshInterval - timeSinceLast)
            return
        }

        refreshPending = true
        lastRefreshTime = now
        try {
            const folders = await invoke<AddOnsFolder[]>('refresh_addon_data')
            addonFolders.value = folders
        } catch (err) {
            console.error('Failed to refresh addon data:', err)
        } finally {
            refreshPending = false
        }
    }

    async function refreshDiskData() {
        const now = Date.now()
        const timeSinceLast = now - lastDiskRefreshTime
        const refreshInterval = 10000 // 10 seconds

        if (diskRefreshPending) {
            return
        }

        if (timeSinceLast < refreshInterval) {
            if (scheduledDiskRefresh) clearTimeout(scheduledDiskRefresh)
            scheduledDiskRefresh = setTimeout(() => {
                scheduledDiskRefresh = null
                refreshDiskData()
            }, refreshInterval - timeSinceLast)
            return
        }

        diskRefreshPending = true
        lastDiskRefreshTime = now
        try {
            const folders = await invoke<AddOnsFolder[]>('refresh_disk_data')
            addonFolders.value = folders
        } catch (err) {
            console.error('Failed to refresh disk data:', err)
        } finally {
            diskRefreshPending = false
        }
    }

    onMounted(async () => {
        listen<InstallEventPayload>('install-event', ({ payload }) => {
            console.debug('[install-event]', payload)
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
