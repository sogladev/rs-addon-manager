import { reactive, computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'

// Types matching the Rust backend
export type OperationType = 'Install' | 'Update'

export interface OperationKey {
    repo_url: string
    folder_path: string
}

export interface OperationEvent {
    Started?: { operation: OperationType }
    Progress?: { current: number; total: number }
    Status?: string
    Warning?: string
    Error?: string
    Completed?: null
}

export interface OperationEventPayload {
    key: OperationKey
    event: OperationEvent
}

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
        return `${key.folder_path}:${key.repo_url}`
    }

    // Get operation state for a specific repo
    function getOperationState(
        repo_url: string,
        folder_path: string
    ): OperationState {
        const id = getOperationId({ repo_url, folder_path })
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
    function isOperationActive(repo_url: string, folder_path: string): boolean {
        return getOperationState(repo_url, folder_path).isActive
    }

    // Get operation type for a specific repo
    function getOperationType(
        repo_url: string,
        folder_path: string
    ): OperationType | undefined {
        return getOperationState(repo_url, folder_path).type
    }

    // Get progress for a specific repo
    function getProgress(
        repo_url: string,
        folder_path: string
    ): { current: number; total: number } | undefined {
        return getOperationState(repo_url, folder_path).progress
    }

    onMounted(async () => {
        // Listen for operation events
        listen<OperationEventPayload>('operation-event', ({ payload }) => {
            const id = getOperationId(payload.key)
            const event = payload.event

            if (event.Started) {
                operations.set(id, {
                    type: event.Started.operation,
                    isActive: true,
                    progress: undefined,
                    status: undefined,
                    warning: undefined,
                    error: undefined,
                })
            } else if (event.Progress) {
                const current = operations.get(id) || { isActive: true }
                current.progress = event.Progress
                current.status = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(id, current)
            } else if (event.Status) {
                const current = operations.get(id) || { isActive: true }
                current.status = event.Status
                current.progress = undefined
                current.warning = undefined
                current.error = undefined
                operations.set(id, current)
            } else if (event.Warning) {
                const current = operations.get(id) || { isActive: true }
                current.warning = event.Warning
                current.error = undefined
                operations.set(id, current)
            } else if (event.Error) {
                const current = operations.get(id) || { isActive: true }
                current.error = event.Error
                current.warning = undefined
                operations.set(id, current)
            } else if ('Completed' in event) {
                // Keep the operation for a short time to show completion state
                const current = operations.get(id) || { isActive: false }
                current.isActive = false
                current.progress = undefined
                current.status = 'Completed'
                current.warning = undefined
                current.error = undefined
                operations.set(id, current)

                // Remove after 2 seconds
                setTimeout(() => {
                    operations.delete(id)
                }, 2000)
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
