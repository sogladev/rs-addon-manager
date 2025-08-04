import { reactive, computed, onMounted, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { OperationKey } from '@bindings/OperationKey'
import type { OperationEventPayload } from '@bindings/OperationEventPayload'
import type { OperationType } from '@bindings/OperationType'

export interface OperationState {
    type?: OperationType
    progress?: { current: number; total: number }
    status?: string
    warning?: string
    error?: string
    isActive: boolean
}

export function useOperationTracker() {
    // Map of stringified OperationKey to operation state
    const operations = reactive<Map<string, OperationState>>(new Map())

    // Recently completed operations tracking
    const recentlyCompleted = ref<
        { keyString: string; type: string; time: number; repoName: string }[]
    >([])

    // Helper to create string key from OperationKey
    function createKeyString(key: OperationKey): string {
        return JSON.stringify(key)
    }

    // Get operation state for a specific repo
    function getOperationState(
        repoUrl: string,
        folderPath: string
    ): OperationState {
        const key: OperationKey = { repoUrl, folderPath }
        const keyString = createKeyString(key)
        return operations.get(keyString) || { isActive: false }
    }

    // Check if any operation is active
    const hasActiveOperations = computed(() => {
        for (const state of operations.values()) {
            if (state.isActive) return true
        }
        return false
    })

    // Get count of active operations
    const activeOperationCount = computed(() => {
        let count = 0
        for (const state of operations.values()) {
            if (state.isActive) count++
        }
        return count
    })

    // Check if a specific repo has an active operation
    function isOperationActive(repoUrl: string, folderPath: string): boolean {
        return getOperationState(repoUrl, folderPath).isActive
    }

    // Get operation type for a specific repo
    function getOperationType(
        repoUrl: string,
        folderPath: string
    ): OperationType | undefined {
        return getOperationState(repoUrl, folderPath).type
    }

    // Get progress for a specific repo
    function getProgress(
        repoUrl: string,
        folderPath: string
    ): { current: number; total: number } | undefined {
        return getOperationState(repoUrl, folderPath).progress
    }

    // copy of clone.rs `extract_owner_repo_from_url(url: &str) -> Result<(String, String), String>`
    function extractOwnerRepoFromUrl(
        url: string
    ): { owner: string; repo: string } | null {
        try {
            const parsed = new URL(url)
            const segments = parsed.pathname.split('/').filter(Boolean)
            if (segments.length < 2) return null
            const owner = segments[segments.length - 2]
            const repo = segments[segments.length - 1].replace(/\.git$/, '')
            return { owner, repo }
        } catch {
            return null
        }
    }

    function extractRepoName(repoUrl: string): string {
        const { repo } = extractOwnerRepoFromUrl(repoUrl) || {}
        return repo || 'Unknown repo'
    }

    onMounted(async () => {
        listen<OperationEventPayload>('operation-event', ({ payload }) => {
            const key = payload.key
            const keyString = createKeyString(key)
            const event = payload.event

            // Handle different event types
            if (typeof event === 'object' && 'error' in event) {
                // Error event: mark inactive, update state, and push to recent events
                const current = operations.get(keyString) || { isActive: true }
                current.error = event.error
                current.warning = undefined
                current.isActive = false
                operations.set(keyString, current)
                const repoName = extractRepoName(key.repoUrl)
                recentlyCompleted.value.push({
                    keyString,
                    type: event.error,
                    time: Date.now(),
                    repoName,
                })
                setTimeout(() => {
                    recentlyCompleted.value = recentlyCompleted.value.filter(
                        (op) => op.keyString !== keyString
                    )
                }, 120000)
                setTimeout(() => {
                    operations.delete(keyString)
                }, 2000)
            } else if (event === 'completed') {
                const current = operations.get(keyString) || { isActive: true }
                current.status = 'Completed'
                current.warning = undefined
                current.error = undefined
                operations.set(keyString, current)

                // Add to recently completed
                const repoName = extractRepoName(key.repoUrl)
                recentlyCompleted.value.push({
                    keyString: keyString,
                    type: current.type || 'unknown',
                    time: Date.now(),
                    repoName,
                })

                const recentlyCompletedDelay = 180000
                setTimeout(() => {
                    recentlyCompleted.value = recentlyCompleted.value.filter(
                        (op) => op.keyString !== keyString
                    )
                }, recentlyCompletedDelay)

                const operationsCleanupDelay = 3000
                setTimeout(() => {
                    operations.delete(keyString)
                }, operationsCleanupDelay)
            } else if (typeof event === 'object' && 'started' in event) {
                operations.set(keyString, {
                    type: event.started.operation,
                    isActive: true,
                })
            } else if (typeof event === 'object' && 'progress' in event) {
                const current = operations.get(keyString) || { isActive: true }
                current.progress = event.progress
                current.status = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(keyString, current)
            } else if (typeof event === 'object' && 'status' in event) {
                const current = operations.get(keyString) || { isActive: true }
                current.status = event.status
                current.progress = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(keyString, current)
            } else if (typeof event === 'object' && 'warning' in event) {
                const current = operations.get(keyString) || { isActive: true }
                current.warning = event.warning
                current.error = undefined
                operations.set(keyString, current)
            }
        })
    })

    return {
        operations,
        hasActiveOperations,
        activeOperationCount,
        isOperationActive,
        getOperationType,
        getProgress,
        getOperationState,
        recentlyCompleted,
    }
}
