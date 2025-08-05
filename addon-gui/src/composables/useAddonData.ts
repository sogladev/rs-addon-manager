import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import { useOperationTracker } from './useOperationTracker'

export function useAddonData() {
    const addonFolders = ref<AddOnsFolder[]>([])
    const folderPaths = computed(() => addonFolders.value.map((f) => f.path))

    const { operations, hasActiveOperations, activeOperationCount } =
        useOperationTracker()

    let refreshPending = false
    let lastRefreshTime = 0
    let scheduledRefresh: ReturnType<typeof setTimeout> | null = null
    let diskRefreshPending = false
    let lastDiskRefreshTime = 0
    let scheduledDiskRefresh: ReturnType<typeof setTimeout> | null = null

    async function refreshAddonData(force = false) {
        const now = Date.now()
        const timeSinceLast = now - lastRefreshTime
        const refreshInterval = 5000

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
        const refreshInterval = 3000

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
        listen('addon-data-updated', () => refreshAddonData())

        listen('addon-disk-updated', () => refreshDiskData())

        await refreshAddonData()
    })

    return {
        addonFolders,
        folderPaths,
        refreshAddonData,
        refreshDiskData,
        operations,
        hasActiveOperations,
        activeOperationCount,
    }
}
