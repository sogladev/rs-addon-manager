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
    // Map of OperationKey to operation state
    const operations = reactive<Map<OperationKey, OperationState>>(new Map())

    // Recently completed operations tracking
    const recentlyCompleted = ref<
        { id: string; type: string; time: number; repoName: string }[]
    >([])

    // Get operation state for a specific repo
    function getOperationState(
        repoUrl: string,
        folderPath: string
    ): OperationState {
        const key: OperationKey = { repoUrl, folderPath }
        return operations.get(key) || { isActive: false }
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

    // Helper function to extract repo name from URL
    function extractRepoName(repoUrl: string): string {
        try {
            const url = new URL(repoUrl)
            const pathParts = url.pathname.split('/').filter(Boolean)
            return pathParts[pathParts.length - 1] || 'Unknown'
        } catch {
            return 'Unknown'
        }
    }

    onMounted(async () => {
        // Listen for operation events
        listen<OperationEventPayload>('operation-event', ({ payload }) => {
            const key = payload.key
            const event = payload.event

            if (event === 'completed') {
                // Completed event
                const current = operations.get(key) || { isActive: false }
                current.isActive = false
                current.progress = undefined
                current.status = 'Completed'
                current.warning = undefined
                current.error = undefined
                operations.set(key, current)

                // Add to recently completed
                const repoName = extractRepoName(key.repoUrl)
                recentlyCompleted.value.push({
                    id: JSON.stringify(key),
                    type: current.type || 'unknown',
                    time: Date.now(),
                    repoName,
                })

                // Clean up old completed operations after 2 minutes
                setTimeout(() => {
                    recentlyCompleted.value = recentlyCompleted.value.filter(
                        (op) => op.id !== JSON.stringify(key)
                    )
                }, 120000)

                setTimeout(() => {
                    operations.delete(key)
                }, 2000)
            } else if ('started' in event) {
                operations.set(key, {
                    type: event.started.operation,
                    isActive: true,
                })
            } else if ('progress' in event) {
                const current = operations.get(key) || { isActive: true }
                current.progress = event.progress
                current.status = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(key, current)
            } else if ('status' in event) {
                const current = operations.get(key) || { isActive: true }
                current.status = event.status
                current.progress = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(key, current)
            } else if ('warning' in event) {
                const current = operations.get(key) || { isActive: true }
                current.warning = event.warning
                current.error = undefined
                operations.set(key, current)
            } else if ('error' in event) {
                const current = operations.get(key) || { isActive: true }
                current.error = event.error
                current.warning = undefined
                operations.set(key, current)
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
