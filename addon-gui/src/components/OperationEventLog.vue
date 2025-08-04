<script setup lang="ts">
import { computed } from 'vue'
import {
    Activity,
    CheckCircle,
    AlertTriangle,
    XCircle,
    Clock,
} from 'lucide-vue-next'

interface OperationEvent {
    id: string
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

const props = defineProps<{
    activeOperations: Map<string, OperationState>
    recentlyCompleted: OperationEvent[]
    activeCount: number
}>()

// Extract active operations with repo names
const activeOperationsList = computed(() => {
    const operations: Array<{
        id: string
        type: string
        repoName: string
        status?: string
        progress?: { current: number; total: number }
    }> = []

    props.activeOperations.forEach((state, id) => {
        if (state.isActive) {
            const [, repoUrl] = id.split(':')
            const repoName = extractRepoName(repoUrl)
            operations.push({
                id,
                type: state.type || 'unknown',
                repoName,
                status: state.status,
                progress: state.progress,
            })
        }
    })

    return operations.slice(0, 3) // Limit to 3 for compact display
})

// Recent events (completed operations)
const recentEvents = computed(() => {
    return props.recentlyCompleted
        .slice(-5) // Show last 5 events
        .reverse() // Most recent first
})

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

// Format time since event
function formatTimeSince(timestamp: number): string {
    const seconds = Math.floor((Date.now() - timestamp) / 1000)
    if (seconds < 60) return `${seconds}s`
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
    return `${Math.floor(seconds / 3600)}h`
}

// Get status icon and color
function getEventIcon(type: string) {
    switch (type.toLowerCase()) {
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
    switch (type.toLowerCase()) {
        case 'install':
        case 'clone':
            return 'text-accent'
        case 'update':
        case 'pull':
            return 'text-primary'
        default:
            return 'text-success'
    }
}

// Determine if there's anything to show
const hasContent = computed(() => {
    return (
        activeOperationsList.value.length > 0 || recentEvents.value.length > 0
    )
})
</script>

<template>
    <div v-if="hasContent" class="dropdown dropdown-end">
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
            class="dropdown-content card card-compact w-80 p-2 shadow bg-base-100 z-[1]"
        >
            <div class="card-body">
                <h3 class="card-title text-sm">
                    <Activity class="w-4 h-4" />
                    Operation Log
                </h3>

                <!-- Active Operations -->
                <div v-if="activeOperationsList.length > 0" class="space-y-2">
                    <div
                        class="text-xs font-medium text-base-content/70 flex items-center gap-1"
                    >
                        <Clock class="w-3 h-3" />
                        Active ({{ activeCount }})
                    </div>
                    <div class="space-y-1">
                        <div
                            v-for="op in activeOperationsList"
                            :key="op.id"
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
                                    {{ op.progress.current }}/{{
                                        op.progress.total
                                    }}
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
                            :key="event.id"
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
                    v-if="!hasContent"
                    class="text-center text-base-content/60 text-xs py-4"
                >
                    No recent activity
                </div>
            </div>
        </div>
    </div>
</template>
