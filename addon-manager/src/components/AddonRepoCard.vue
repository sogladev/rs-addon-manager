<script setup lang="ts">
import { useGlobalError } from '@/composables/useGlobalError'
import { useOperationTracker } from '@/composables/useOperationTracker'
import { useExternalLink } from '@/composables/useExternalLink'
import type { Addon } from '@bindings/Addon'
import type { AddonRepository } from '@bindings/AddonRepository'
import { invoke } from '@tauri-apps/api/core'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { Ellipsis, FileText, Globe, Trash2, Wrench } from 'lucide-vue-next'
import { marked } from 'marked'
import { computed, ref, watch } from 'vue'

const { addIssue } = useGlobalError()

const { repo, folderPath } = defineProps<{
    repo: AddonRepository
    folderPath: string
}>()

const emit = defineEmits<{
    delete: []
    'branch-change': [newBranch: string]
}>()

// Helper computed properties to extract fields from the source union
const isGit = computed(() => repo.source.type === 'git')
const repoKey = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.repo_url
    }
    return `local://${repo.source.folder_name}`
})
const repoName = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.repo_name
    }
    return repo.source.folder_name
})
const owner = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.owner
    }
    return 'Unknown'
})
const currentBranch = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.current_branch
    }
    return null
})
const availableBranches = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.available_branches
    }
    return []
})
const repoRef = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.repo_ref
    }
    return null
})
const latestRef = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.latest_ref
    }
    return null
})
const readme = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.readme
    }
    return null
})
const repoUrl = computed(() => {
    if (repo.source.type === 'git') {
        return repo.source.repo_url
    }
    return `file://${repo.source.path}`
})

const showReadmeModal = ref(false)
const readmeHtml = ref('')

const { openWebsite } = useExternalLink()

const handleWebsite = () => {
    openWebsite(repoUrl.value)
}

const { isOperationActive, getOperationType, getProgress } =
    useOperationTracker()

async function handleToggleAddon(addon: Addon) {
    try {
        if (addon.isSymlinked) {
            await invoke('create_addon_symlink', {
                repoUrl: repoKey.value,
                folderPath: folderPath,
                addonName: addon.name,
            })
        } else {
            await invoke('remove_addon_symlink', {
                repoUrl: repoKey.value,
                folderPath: folderPath,
                addonName: addon.name,
            })
        }
    } catch (e) {
        console.error('Symlink operation failed:', e)
        addIssue(`Failed to toggle symlink for addon: ${addon.name}`, e)
    }
}

function handleButtonClick() {
    if (!repoRef.value) {
        invoke('install_addon_cmd', {
            url: repoUrl.value,
            path: folderPath,
            branch: selectedBranch.value,
        }).catch((e) => {
            console.error('Install failed:', e)
            addIssue(
                `Failed to install addon install_addon_cmd: ${repoName.value}`,
                e?.message || String(e)
            )
        })
    } else {
        invoke('update_addon_cmd', {
            url: repoUrl.value,
            path: folderPath,
            branch: selectedBranch.value,
        }).catch((e) => {
            console.error('Update failed:', e)
            addIssue(
                `Failed to update addon update_addon_cmd: ${repoName.value}`,
                e?.message || String(e)
            )
        })
    }
}

async function handleReadme() {
    let content = ''
    const path = readme.value

    if (!path) {
        console.warn('No README path provided for', repoName.value)
        return
    }

    try {
        await invoke('allow_file', { path })
        content = await readTextFile(path)
    } catch (e) {
        console.error(`No README found at ${path}`, e)
        addIssue(
            `No README found at ${path} for ${repoName.value} ${repoUrl.value}`,
            e
        )
    }

    if (!content) {
        console.warn('No README content found at', path)
        return
    }

    const rawHtml = marked(content)
    readmeHtml.value = typeof rawHtml === 'string' ? rawHtml : await rawHtml
    showReadmeModal.value = true
}

function closeReadmeModal() {
    showReadmeModal.value = false
    readmeHtml.value = ''
}

function handleRepair() {
    console.log('Repair repo', repoUrl.value)
    // re-install
    if (isGit.value) {
        invoke('install_addon_cmd', {
            url: repoUrl.value,
            path: folderPath,
            branch: currentBranch.value,
        })
    }
}

const selectedBranch = ref(currentBranch.value)
const branchChanged = ref(false)
watch(
    () => currentBranch.value,
    (newBranch) => {
        selectedBranch.value = newBranch
        branchChanged.value = false
    }
)

const isOperating = computed(() => isOperationActive(repoKey.value, folderPath))
const operationType = computed(() =>
    getOperationType(repoKey.value, folderPath)
)
const operationProgress = computed(() => getProgress(repoKey.value, folderPath))

function handleBranchChange(e: Event) {
    const target = e.target as HTMLSelectElement | null
    if (!target) return
    const newBranch = target.value
    selectedBranch.value = newBranch
    // Enable update only when new branch differs from disk state
    branchChanged.value = newBranch !== currentBranch.value
    emit('branch-change', newBranch)
}

const updateAvailable = computed(() => {
    // If branch was changed, always show update available
    if (branchChanged.value) return true
    return latestRef.value && repoRef.value !== latestRef.value
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

    if (!isGit.value) {
        return 'Local'
    }

    if (!repoRef.value) {
        return 'Install'
    }

    if (updateAvailable.value) {
        return 'Update'
    }

    return 'Update'
})

const buttonDisabled = computed(() => {
    // Disable if currently operating
    if (isOperating.value) return true

    // Disable if non-git repository (no updates available)
    if (!isGit.value) return true

    // If not installed, always allow install
    if (!repoRef.value) return false

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
    <div class="card card-bordered bg-base-100 flex-row items-center">
        <div class="flex flex-1 flex-col gap-1 p-2">
            <span class="font-semibold">{{ repoName }}</span>
            <span class="text-xs text-base-content/60">{{ owner }}</span>
            <div v-if="repo.addons && repo.addons.length">
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
                        <span
                            class="font-mono text-xs flex items-center gap-1"
                            :class="
                                addon.notes
                                    ? 'tooltip tooltip-right cursor-pointer'
                                    : ''
                            "
                            :data-tip="addon.notes || undefined"
                        >
                            {{ addon.name }}
                        </span>
                    </li>
                </ul>
            </div>
        </div>
        <div class="flex items-center gap-2">
            <div v-if="isGit" class="w-32">
                <select
                    class="select select-bordered select-sm w-full truncate"
                    v-model="selectedBranch"
                    @change="handleBranchChange"
                >
                    <option
                        v-for="branch in availableBranches"
                        :key="branch"
                        :value="branch"
                    >
                        {{ branch }}
                    </option>
                </select>
            </div>
            <div v-else class="badge badge-neutral">Local Folder</div>
            <button
                :class="[
                    'btn btn-sm relative overflow-hidden w-20',
                    updateAvailable || !repoRef ? 'btn-primary' : 'btn-primary',
                ]"
                @click="handleButtonClick"
                :disabled="buttonDisabled"
                :title="!isGit ? 'Local folders cannot be updated' : ''"
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
                    <li
                        :class="{ 'menu-disabled': !readme }"
                        @click="handleReadme"
                    >
                        <button
                            class="flex items-center gap-2"
                            :disabled="!readme"
                            tabindex="-1"
                        >
                            <FileText class="w-4 h-4" />
                            Readme
                        </button>
                    </li>
                    <li
                        :class="{ 'menu-disabled': !isGit }"
                        @click="handleWebsite"
                    >
                        <button
                            class="flex items-center gap-2"
                            :disabled="!isGit"
                        >
                            <Globe class="w-4 h-4" />
                            Website
                        </button>
                    </li>
                    <li
                        :class="{ 'menu-disabled': !isGit }"
                        @click="handleRepair"
                    >
                        <button
                            class="flex items-center gap-2"
                            :disabled="!isGit"
                        >
                            <Wrench class="w-4 h-4" />
                            Repair
                        </button>
                    </li>
                    <li @click="emit('delete')">
                        <button class="flex items-center gap-2 text-error">
                            <Trash2 class="w-4 h-4" />
                            Delete
                        </button>
                    </li>
                </ul>
            </div>
        </div>
    </div>

    <div v-if="showReadmeModal" class="modal modal-open">
        <div class="modal-box max-w-3xl">
            <button
                class="btn btn-sm btn-circle absolute right-2 top-2"
                @click="closeReadmeModal"
            >
                ✕
            </button>
            <div class="prose max-w-none" v-html="readmeHtml"></div>
        </div>
        <div class="modal-backdrop" @click="closeReadmeModal"></div>
    </div>
</template>

<style scoped></style>
