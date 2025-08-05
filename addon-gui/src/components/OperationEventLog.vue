<script setup lang="ts">
import { OperationKey } from '@bindings/OperationKey'
import { Activity, CheckCircle, XCircle, Clock, X } from 'lucide-vue-next'
import { computed } from 'vue'

interface OperationEvent {
    keyString: string
    type: string
    time: number
    repoName: string
}

interface OperationState {
    type?: string
    progress?: { current: number; total: number }
    status?: string
    warning?: string
    error?: string
    isActive: boolean
}

const { activeOperations, recentlyCompleted, activeCount } = defineProps<{
    activeOperations: Map<string, OperationState>
    recentlyCompleted: OperationEvent[]
    activeCount: number
}>()

const activeOperationsList = computed(() => {
    const operations: Array<{
        keyString: string
        type: string
        repoName: string
        status?: string
        progress?: { current: number; total: number }
    }> = []

    activeOperations.forEach((state, keyString) => {
        if (state.isActive) {
            // Parse the keyString back to OperationKey to get repoUrl
            try {
                const key: OperationKey = JSON.parse(keyString)
                const repoUrl = key.repoUrl || ''
                operations.push({
                    keyString,
                    type: state.type || 'unknown',
                    repoName: repoUrl,
                    status: state.status,
                    progress: state.progress,
                })
            } catch (e) {
                console.error('Failed to parse operation key:', keyString, e)
            }
        }
    })

    return operations.slice(0, 5) // Limit for compact display
})

const recentEvents = computed(() => {
    return recentlyCompleted.slice(-5).reverse() // Most recent first
})

function formatTimeSince(timestamp: number): string {
    const seconds = Math.floor((Date.now() - timestamp) / 1000)
    if (seconds < 60) return `${seconds}s`
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
    return `${Math.floor(seconds / 3600)}h`
}

function getEventIcon(type: string) {
    const lower = type.toLowerCase()
    if (lower.startsWith('failed') || lower.includes('error')) {
        return XCircle
    }
    switch (lower) {
        case 'install':
        case 'clone':
            return CheckCircle
        case 'update':
        case 'pull':
            return Activity
        default:
            return CheckCircle
    }
}

function getEventColor(type: string) {
    const lower = type.toLowerCase()
    if (
        lower.startsWith('failed') ||
        lower.includes('error') ||
        lower === 'delete' ||
        lower === 'remove'
    ) {
        return 'text-error'
    }
    switch (lower) {
        case 'install':
        case 'clone':
            return 'text-success'
        case 'update':
        case 'pull':
            return 'text-primary'
        default:
            return 'text-accent'
    }
}

function formatProgressPercent(progress?: {
    current: number
    total: number
}): string {
    if (!progress || progress.total === 0) return ''
    const percent = Math.floor((progress.current / progress.total) * 100)
    return `${percent}%`
}

function clearActivity() {
    recentlyCompleted.splice(0, recentlyCompleted.length)
}
</script>

<template>
    <div class="dropdown dropdown-end">
        <div
            tabindex="0"
            role="button"
            class="btn btn-ghost btn-square relative"
        >
            <Activity class="w-4 h-4" />
            <!-- Activity indicator -->
            <div v-if="activeCount > 0" class="absolute -top-1 -right-1">
                <div
                    class="w-3 h-3 bg-primary rounded-full flex items-center justify-center"
                >
                    <span class="text-xs text-primary-content font-bold">{{
                        activeCount > 9 ? '9+' : activeCount
                    }}</span>
                </div>
            </div>
            <!-- Recent events indicator -->
            <div
                v-else-if="recentEvents.length > 0"
                class="absolute -top-1 -right-1"
            >
                <div class="w-2 h-2 bg-success rounded-full"></div>
            </div>
        </div>

        <div
            tabindex="0"
            class="dropdown-content card card-compact w-80 shadow bg-base-100 z-[100]"
        >
            <div class="card-body">
                <div class="flex items-center justify-between">
                    <h3 class="card-title text-sm flex items-center gap-2">
                        <Activity class="w-4 h-4" />
                        Operation Log
                    </h3>
                    <button
                        class="btn btn-xs btn-ghost"
                        @click="clearActivity"
                        title="Clear notifications"
                    >
                        <X class="w-4 h-4" />
                    </button>
                </div>

                <!-- Active Operations -->
                <div v-if="activeOperationsList.length > 0" class="space-y-2">
                    <div
                        class="text-xs font-medium text-base-content flex items-center gap-1"
                    >
                        <Clock class="w-3 h-3" />
                        Active ({{ activeCount }})
                    </div>
                    <div class="space-y-1">
                        <div
                            v-for="op in activeOperationsList"
                            :key="op.keyString"
                            class="flex items-center gap-2 p-2 bg-base-200 rounded text-xs"
                        >
                            <div
                                class="loading loading-spinner loading-xs text-primary"
                            ></div>
                            <div class="flex-1 min-w-0">
                                <div class="font-medium truncate">
                                    {{ op.repoName }}
                                </div>
                                <div class="text-base-content/60 capitalize">
                                    {{ op.type }}
                                </div>
                                <div
                                    v-if="op.status"
                                    class="text-base-content/60"
                                >
                                    {{ op.status }}
                                </div>
                                <div
                                    v-if="op.progress"
                                    class="text-base-content/60"
                                >
                                    {{ formatProgressPercent(op.progress) }}
                                </div>
                            </div>
                        </div>
                    </div>
                    <div
                        v-if="activeCount > 3"
                        class="text-xs text-base-content/60 text-center"
                    >
                        +{{ activeCount - 3 }} more operations
                    </div>
                </div>

                <!-- Divider -->
                <div
                    v-if="
                        activeOperationsList.length > 0 &&
                        recentEvents.length > 0
                    "
                    class="divider my-2"
                ></div>

                <!-- Recent Events -->
                <div v-if="recentEvents.length > 0" class="space-y-2">
                    <div
                        class="text-xs font-medium text-base-content/70 flex items-center gap-1"
                    >
                        <CheckCircle class="w-3 h-3" />
                        Recent
                    </div>
                    <div class="space-y-1">
                        <div
                            v-for="event in recentEvents"
                            :key="event.keyString"
                            class="flex items-center gap-2 p-2 bg-base-200 rounded text-xs"
                        >
                            <component
                                :is="getEventIcon(event.type)"
                                class="w-3 h-3 flex-shrink-0"
                                :class="getEventColor(event.type)"
                            />
                            <div class="flex-1 min-w-0">
                                <div class="font-medium truncate">
                                    {{ event.repoName }}
                                </div>
                                <div class="text-base-content/60 capitalize">
                                    {{ event.type }}
                                </div>
                            </div>
                            <div class="text-base-content/50 text-xs">
                                {{ formatTimeSince(event.time) }}
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Empty state -->
                <div
                    v-if="
                        activeOperationsList.length === 0 &&
                        recentEvents.length === 0
                    "
                    class="text-center text-base-content/60 text-xs py-4"
                >
                    No recent activity
                </div>
            </div>
        </div>
    </div>
</template>
