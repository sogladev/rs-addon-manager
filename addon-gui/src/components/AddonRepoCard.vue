<script setup lang="ts">
import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'
import { Ellipsis } from 'lucide-vue-next'
import { FileText, Globe, Wrench, Trash2 } from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'
import { useOperationTracker } from '@/composables/useOperationTracker'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
    repo: AddonRepository & { latestRef?: string | null }
    folderPath: string
}>()
const emit = defineEmits<{
    'branch-change': [branch: string]
    delete: []
}>()

const { isOperationActive, getOperationType, getProgress } =
    useOperationTracker()

function handleToggleAddon(addon: Addon) {
    // Symlink logic
    if (addon.isSymlinked) {
        invoke('create_addon_symlink', {
            repoUrl: props.repo.repoUrl,
            folderPath: props.folderPath,
            addonName: addon.name,
        })
    } else {
        invoke('remove_addon_symlink', {
            repoUrl: props.repo.repoUrl,
            folderPath: props.folderPath,
            addonName: addon.name,
        })
    }
}

function handleButtonClick() {
    // Install or update directly
    if (!props.repo.repoRef) {
        invoke('install_addon_cmd', {
            url: props.repo.repoUrl,
            path: props.folderPath,
            branch: props.repo.currentBranch,
        }).catch((e) => console.error('Install failed:', e))
    } else {
        invoke('update_addon_cmd', {
            url: props.repo.repoUrl,
            path: props.folderPath,
            branch: props.repo.currentBranch,
        }).catch((e) => console.error('Update failed:', e))
    }
}

// Open repository readme (stub)
function handleReadme() {
    console.log('Open readme for', props.repo.repoUrl)
    // TODO: implement reading README file or opening its URL
}

// Open repository website
function handleWebsite() {
    const url = props.repo.repoUrl.replace(/\.git$/, '')
    window.open(url, '_blank')
}

// Repair repository (stub)
function handleRepair() {
    console.log('Repair repo', props.repo.repoUrl)
    // re-install
    invoke('install_addon_cmd', {
        url: props.repo.repoUrl,
        path: props.folderPath,
        branch: props.repo.currentBranch,
    })
}

// Computed properties for operation state
// Track selected branch and detect if changed from original
const selectedBranch = ref(props.repo.currentBranch)
const branchChanged = ref(false)
// Reset branchChanged when repo.currentBranch prop updates
watch(
    () => props.repo.currentBranch,
    (newBranch) => {
        selectedBranch.value = newBranch
        branchChanged.value = false
    }
)

// Computed properties for operation state
const isOperating = computed(() =>
    isOperationActive(props.repo.repoUrl, props.folderPath)
)

const operationType = computed(() =>
    getOperationType(props.repo.repoUrl, props.folderPath)
)

const operationProgress = computed(() =>
    getProgress(props.repo.repoUrl, props.folderPath)
)

function handleBranchChange(e: Event) {
    const target = e.target as HTMLSelectElement | null
    if (!target) return
    const newBranch = target.value
    selectedBranch.value = newBranch
    // Enable update only when new branch differs from disk state
    branchChanged.value = newBranch !== props.repo.currentBranch
    emit('branch-change', newBranch)
}

const updateAvailable = computed(() => {
    // If branch was changed, always show update available
    if (branchChanged.value) return true
    return props.repo.latestRef && props.repo.repoRef !== props.repo.latestRef
})

// Computed button text and state
const buttonText = computed(() => {
    if (isOperating.value) {
        switch (operationType.value) {
            case 'update':
                return 'Updating...'
            case 'install':
                return 'Installing...'
            default:
                return 'Processing...'
        }
    }

    // If repo is not installed (no repoRef), show Install
    if (!props.repo.repoRef) {
        return 'Install'
    }

    // If update is available, show Update
    if (updateAvailable.value) {
        return 'Update'
    }

    return 'Update'
})

const buttonDisabled = computed(() => {
    // Disable if currently operating
    if (isOperating.value) return true

    // If not installed, always allow install
    if (!props.repo.repoRef) return false

    // If branch was changed, always enable update
    if (branchChanged.value) return false

    // If installed, only allow update if update is available
    return !updateAvailable.value
})

const progressPercent = computed(() => {
    if (!operationProgress.value) return 0
    const { current, total } = operationProgress.value
    return total > 0 ? (current / total) * 100 : 0
})
</script>

<template>
    <div class="card card-bordered bg-base-100 flex-row items-center p-2">
        <div class="flex flex-1 flex-col gap-1 p-2">
            <span class="font-semibold">{{ repo.repoName }}</span>
            <span class="text-xs text-base-content/60">{{ repo.owner }}</span>
            <div v-if="repo.addons && repo.addons.length" class="mt-1">
                <ul class="ml-2 flex flex-col gap-1">
                    <li
                        v-for="addon in repo.addons"
                        :key="addon.name"
                        class="flex items-center gap-2"
                    >
                        <input
                            type="checkbox"
                            class="checkbox checkbox-sm"
                            v-model="addon.isSymlinked"
                            @change="handleToggleAddon(addon)"
                        />
                        <span class="font-mono text-xs">{{ addon.name }}</span>
                        <!-- <span v-if="!addon.isSymlinked" class="badge badge-xs badge-error" > disabled </span> -->
                    </li>
                </ul>
            </div>
        </div>
        <div class="flex items-center gap-2">
            <div class="w-40">
                <select
                    class="select select-bordered select-sm w-full truncate"
                    v-model="selectedBranch"
                    @change="handleBranchChange"
                >
                    <option
                        v-for="branch in repo.availableBranches"
                        :key="branch"
                        :value="branch"
                    >
                        {{ branch }}
                    </option>
                </select>
            </div>
            <button
                :class="[
                    'btn btn-sm relative overflow-hidden w-20',
                    updateAvailable || !repo.repoRef
                        ? 'btn-primary'
                        : 'btn-primary',
                ]"
                @click="handleButtonClick"
                :disabled="buttonDisabled"
            >
                <span class="relative z-10">{{ buttonText }}</span>
                <div
                    v-if="isOperating && operationProgress"
                    class="absolute left-0 top-0 h-full bg-primary/30 transition-all"
                    :style="{ width: progressPercent + '%' }"
                ></div>
            </button>
            <div class="dropdown dropdown-end">
                <button tabindex="0" class="btn btn-sm btn-ghost">
                    <Ellipsis />
                </button>
                <ul
                    tabindex="0"
                    class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-44"
                >
                    <li>
                        <button
                            class="flex items-center gap-2"
                            @click="handleReadme"
                        >
                            <FileText class="w-4 h-4" />
                            Readme
                        </button>
                    </li>
                    <li>
                        <button
                            class="flex items-center gap-2"
                            @click="handleWebsite"
                        >
                            <Globe class="w-4 h-4" />
                            Website
                        </button>
                    </li>
                    <li>
                        <button
                            class="flex items-center gap-2"
                            @click="handleRepair"
                        >
                            <Wrench class="w-4 h-4" />
                            Repair
                        </button>
                    </li>
                    <li>
                        <button
                            class="flex items-center gap-2 text-error"
                            @click="emit('delete')"
                        >
                            <Trash2 class="w-4 h-4" />
                            Delete
                        </button>
                    </li>
                </ul>
            </div>
        </div>
    </div>
</template>

<style scoped></style>
