import { reactive, computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { OperationKey } from '@bindings/OperationKey'
// import type { OperationEvent } from '@bindings/OperationEvent'
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
    // Map of operation ID to operation state
    const operations = reactive<Map<string, OperationState>>(new Map())

    // Helper function to generate operation ID
    function getOperationId(key: OperationKey): string {
        return `${key.folderPath}:${key.repoUrl}`
    }

    // Get operation state for a specific repo
    function getOperationState(
        repoUrl: string,
        folderPath: string
    ): OperationState {
        const id = getOperationId({ repoUrl, folderPath })
        return operations.get(id) || { isActive: false }
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

    onMounted(async () => {
        // Listen for operation events
        listen<OperationEventPayload>('operation-event', ({ payload }) => {
            const key = payload.key
            const id = getOperationId(key)
            const event = payload.event

            if (event === 'completed') {
                // Completed event
                const current = operations.get(id) || { isActive: false }
                current.isActive = false
                current.progress = undefined
                current.status = 'Completed'
                current.warning = undefined
                current.error = undefined
                operations.set(id, current)
                setTimeout(() => {
                    operations.delete(id)
                }, 2000)
            } else if ('started' in event) {
                operations.set(id, {
                    type: event.started.operation,
                    isActive: true,
                })
            } else if ('progress' in event) {
                const current = operations.get(id) || { isActive: true }
                current.progress = event.progress
                current.status = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(id, current)
            } else if ('status' in event) {
                const current = operations.get(id) || { isActive: true }
                current.status = event.status
                current.progress = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(id, current)
            } else if ('warning' in event) {
                const current = operations.get(id) || { isActive: true }
                current.warning = event.warning
                current.error = undefined
                operations.set(id, current)
            } else if ('error' in event) {
                const current = operations.get(id) || { isActive: true }
                current.error = event.error
                current.warning = undefined
                operations.set(id, current)
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
    }
}
