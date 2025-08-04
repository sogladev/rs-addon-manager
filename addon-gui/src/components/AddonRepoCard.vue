<script setup lang="ts">
import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'
import { Ellipsis } from 'lucide-vue-next'
import { FileText, Globe, Wrench, Trash2 } from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'
import { useOperationTracker } from '@/composables/useOperationTracker'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { marked } from 'marked'
import { useGlobalError } from '@/composables/useGlobalError'

const { addIssue } = useGlobalError()

const { repo, folderPath } = defineProps<{
    repo: AddonRepository & { latestRef?: string | null }
    folderPath: string
}>()

const emit = defineEmits<{
    delete: []
    'branch-change': [newBranch: string]
}>()

const showReadmeModal = ref(false)
const readmeHtml = ref('')

const handleWebsite = () => {
    const url = repo.repoUrl.replace(/\.git$/, '')
    console.debug('Open website', url)
    openUrl(url)
}

const { isOperationActive, getOperationType } = useOperationTracker()

async function handleToggleAddon(addon: Addon) {
    try {
        if (addon.isSymlinked) {
            await invoke('create_addon_symlink', {
                repoUrl: repo.repoUrl,
                folderPath: folderPath,
                addonName: addon.name,
            })
        } else {
            await invoke('remove_addon_symlink', {
                repoUrl: repo.repoUrl,
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
    if (!repo.repoRef) {
        invoke('install_addon_cmd', {
            url: repo.repoUrl,
            path: folderPath,
            branch: selectedBranch.value,
        }).catch((e) => {
            console.error('Install failed:', e)
            addIssue(
                `Failed to install addon install_addon_cmd: ${repo.repoName}`,
                e?.message || String(e)
            )
        })
    } else {
        invoke('update_addon_cmd', {
            url: repo.repoUrl,
            path: folderPath,
            branch: selectedBranch.value,
        }).catch((e) => {
            console.error('Update failed:', e)
            addIssue(
                `Failed to update addon update_addon_cmd: ${repo.repoName}`,
                e?.message || String(e)
            )
        })
    }
}

async function handleReadme() {
    let content = ''
    const path = repo.readme

    if (!path) {
        console.warn('No README path provided for', repo.repoName)
        return
    }

    try {
        await invoke('allow_file', { path })
        content = await readTextFile(path)
    } catch (e) {
        console.error(`No README found at ${path}`, e)
        addIssue(
            `No README found at ${path} for ${repo.repoName} ${repo.repoUrl}`,
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
    console.log('Repair repo', repo.repoUrl)
    // re-install
    invoke('install_addon_cmd', {
        url: repo.repoUrl,
        path: folderPath,
        branch: repo.currentBranch,
    })
}

// Computed properties for operation state
// Track selected branch and detect if changed from original
const selectedBranch = ref(repo.currentBranch)
const branchChanged = ref(false)
// Reset branchChanged when repo.currentBranch prop updates
watch(
    () => repo.currentBranch,
    (newBranch) => {
        selectedBranch.value = newBranch
        branchChanged.value = false
    }
)

// Computed properties for operation state
const isOperating = computed(() => isOperationActive(repo.repoUrl, folderPath))

const operationType = computed(() => getOperationType(repo.repoUrl, folderPath))

function handleBranchChange(e: Event) {
    const target = e.target as HTMLSelectElement | null
    if (!target) return
    const newBranch = target.value
    selectedBranch.value = newBranch
    // Enable update only when new branch differs from disk state
    branchChanged.value = newBranch !== repo.currentBranch
    emit('branch-change', newBranch)
}

const updateAvailable = computed(() => {
    // If branch was changed, always show update available
    if (branchChanged.value) return true
    return repo.latestRef && repo.repoRef !== repo.latestRef
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
    if (!repo.repoRef) {
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
    if (!repo.repoRef) return false

    // If branch was changed, always enable update
    if (branchChanged.value) return false

    // If installed, only allow update if update is available
    return !updateAvailable.value
})
</script>

<template>
    <div class="card card-bordered bg-base-100 flex-row items-center">
        <div class="flex flex-1 flex-col gap-1 p-2">
            <span class="font-semibold">{{ repo.repoName }}</span>
            <span class="text-xs text-base-content/60">{{ repo.owner }}</span>
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
                    'btn btn-sm w-20',
                    updateAvailable || !repo.repoRef
                        ? 'btn-primary'
                        : 'btn-primary',
                ]"
                @click="handleButtonClick"
                :disabled="buttonDisabled"
            >
                {{ buttonText }}
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
                        :class="{ 'menu-disabled': !repo.readme }"
                        @click="handleReadme"
                    >
                        <button
                            class="flex items-center gap-2"
                            :disabled="!repo.readme"
                            tabindex="-1"
                        >
                            <FileText class="w-4 h-4" />
                            Readme
                        </button>
                    </li>
                    <li @click="handleWebsite">
                        <button class="flex items-center gap-2">
                            <Globe class="w-4 h-4" />
                            Website
                        </button>
                    </li>
                    <li @click="handleRepair">
                        <button class="flex items-center gap-2">
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
