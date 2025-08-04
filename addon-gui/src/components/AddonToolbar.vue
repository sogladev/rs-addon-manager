<script setup lang="ts">
import {
    Menu,
    Copy,
    Save,
    Info,
    Import,
    ArrowRightFromLine,
    Palette,
} from 'lucide-vue-next'
import TimeoutButton from '@/components/TimeoutButton.vue'
import ThemeController from '@/components/ThemeController.vue'
import OperationEventLog from '@/components/OperationEventLog.vue'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { useGlobalError } from '@/composables/useGlobalError'

const { addIssue } = useGlobalError()

import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import { OperationKey } from '@bindings/OperationKey'
import { OperationState } from '@/composables/useOperationTracker'

const {
    search,
    hasUpdates,
    outOfDateCount,
    folders,
    operations,
    activeOperationCount,
    recentlyCompleted,
} = defineProps<{
    search: string
    hasUpdates: boolean
    outOfDateCount: number
    folders: AddOnsFolder[]
    operations: Map<OperationKey, OperationState>
    activeOperationCount?: number
    recentlyCompleted?: Array<{
        id: string
        type: string
        time: number
        repoName: string
    }>
}>()

const emit = defineEmits<{
    'update:search': [value: string]
    'update-all': []
    refresh: []
    'add-addon': []
}>()

const showImport = ref(false)
const importText = ref('')
const showExport = ref(false)
const exportText = ref('')
const showAbout = ref(false)
const showTheme = ref(false)
const isImporting = ref(false)
const importProgress = ref({ current: 0, total: 0 })

const confirmImport = async () => {
    // Each line: <path> <addonName> *<gitUrl> <branch>
    // Skip header or comment lines
    if (!importText.value.trim() || isImporting.value) return

    isImporting.value = true
    const lines = importText.value.split(/\r?\n/).filter((line) => {
        const trimmed = line.trim()
        return trimmed && !trimmed.startsWith('//') && !trimmed.startsWith('#')
    })
    importProgress.value = { current: 0, total: lines.length }

    try {
        // Process imports sequentially to avoid overwhelming the system
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i]
            const parts = line.trim().split(/\s+/)
            if (parts.length >= 4) {
                const folderPath = parts[0]
                const gitUrl = parts[2].startsWith('*')
                    ? parts[2].slice(1)
                    : parts[2]
                const branch = parts[3]

                try {
                    const alreadyManaged = folders.some?.(
                        (f) => f.path === folderPath
                    )
                    if (!alreadyManaged) {
                        await invoke('add_addon_directory', {
                            path: folderPath,
                        })
                    }

                    await invoke('install_addon_cmd', {
                        url: gitUrl,
                        path: folderPath,
                        branch,
                    })

                    // Small delay to prevent overwhelming the system
                    await new Promise((resolve) => setTimeout(resolve, 100))
                } catch (e) {
                    addIssue(
                        `Import failed on line ${i + 1}: "${line}"\nGit URL: ${gitUrl}\nError: ${e instanceof Error ? e.message : String(e)}`,
                        {
                            lineNumber: i + 1,
                            lineContent: line,
                            gitUrl,
                            branch,
                            folderPath,
                            error: e,
                            stack: e instanceof Error ? e.stack : undefined,
                        }
                    )
                    console.error(`Import failed for ${gitUrl}:`, e)
                }
            }
            importProgress.value.current = i + 1
        }
    } finally {
        isImporting.value = false
        importText.value = ''
        showImport.value = false
        importProgress.value = { current: 0, total: 0 }
    }
}

// Generate export lines with header comment
const handleExport = () => {
    const header = '// Each line: <path> <addonName> *<gitUrl> <branch>'
    const lines: string[] = []
    folders.forEach((folder) => {
        folder.repositories.forEach((repo) => {
            const branch = repo.currentBranch || 'main'
            repo.addons.forEach((addon) => {
                lines.push(
                    `${folder.path} ${addon.name} *${repo.repoUrl} ${branch}`
                )
            })
        })
    })
    exportText.value = [header, ...lines].join('\n')
    showExport.value = true
}

// Copy export text to clipboard and close modal
const copyAndClose = () => {
    navigator.clipboard
        .writeText(exportText.value)
        .then(() => {
            showExport.value = false
        })
        .catch((err) => console.error('Copy failed:', err))
}

const saveToFile = async () => {
    try {
        const path = await save({
            filters: [{ name: 'Text', extensions: ['txt'] }],
            defaultPath: 'addons-export.txt',
        })
        if (path) {
            await writeTextFile(path, exportText.value)
        }
        showExport.value = false
    } catch (err) {
        console.error('Save failed:', err)
        showExport.value = false
    }
}

const { getIssueLog, saveIssueLog } = useGlobalError()
const showLog = ref(false)

// Copy log to clipboard and close modal
const copyLogAndClose = async () => {
    try {
        await navigator.clipboard.writeText(getIssueLog())
    } catch (e) {
        console.error('Failed to copy log:', e)
    }
    showLog.value = false
}
// Save log and close modal
const saveLogAndClose = async () => {
    await saveIssueLog()
    showLog.value = false
}
</script>

<template>
    <div class="sticky top-0 z-10 bg-base-200 flex flex-col gap-0">
        <div
            class="flex flex-wrap items-center gap-2 bg-base-200 pb-2 pt-2 px-2"
        >
            <TimeoutButton
                :timeout="2000"
                class="btn btn-secondary"
                @click="emit('refresh')"
            >
                Check for Updates
            </TimeoutButton>
            <TimeoutButton
                :timeout="5000"
                class="btn btn-primary w-40"
                :disabled="!hasUpdates"
                @click="emit('update-all')"
            >
                <span v-if="hasUpdates"
                    >Update All
                    <span v-if="outOfDateCount > 0"
                        >({{ outOfDateCount }})</span
                    >
                </span>
                <span v-else>Up-to-date</span>
            </TimeoutButton>
            <input
                :value="search"
                @input="
                    emit(
                        'update:search',
                        ($event.target as HTMLInputElement).value
                    )
                "
                class="input input-bordered flex-1 max-w-xs ml-auto"
                placeholder="Search installed"
                type="search"
            />
            <button class="btn btn-accent w-40" @click="emit('add-addon')">
                <!-- <Plus /> -->
                Install addon
            </button>
            <OperationEventLog
                v-if="operations && recentlyCompleted"
                :activeOperations="operations"
                :recentlyCompleted="recentlyCompleted"
                :activeCount="activeOperationCount || 0"
            />
            <div class="dropdown dropdown-end">
                <button tabindex="0" class="btn btn-ghost btn-square">
                    <Menu />
                </button>
                <ul
                    tabindex="0"
                    class="dropdown-content menu shadow bg-base-100 rounded-box w-52"
                >
                    <li>
                        <button
                            @click="showTheme = true"
                            class="flex items-center gap-2"
                        >
                            <Palette class="w-4 h-4" />
                            Theme
                        </button>
                    </li>
                    <li>
                        <button
                            @click="showImport = true"
                            class="flex items-center gap-2"
                        >
                            <Import class="w-4 h-4" />
                            Import
                        </button>
                    </li>
                    <li>
                        <button
                            @click="handleExport"
                            class="flex items-center gap-2"
                        >
                            <ArrowRightFromLine class="w-4 h-4" />
                            Export
                        </button>
                    </li>
                    <li>
                        <button
                            @click="showAbout = true"
                            class="flex items-center gap-2"
                        >
                            <Info class="w-4 h-4" />
                            About
                        </button>
                    </li>
                    <li>
                        <button
                            @click="showLog = true"
                            class="flex items-center gap-2"
                        >
                            <Copy class="w-4 h-4" />
                            Report Issue
                        </button>
                    </li>
                </ul>
            </div>
        </div>
    </div>
    <!-- Import Modal -->
    <div
        v-if="showImport"
        class="modal modal-open"
        @click.self="showImport = false"
    >
        <div class="modal-box">
            <h3 class="font-bold text-lg">Import Addons</h3>
            <textarea
                v-model="importText"
                rows="6"
                class="textarea textarea-bordered w-full mt-2"
                placeholder="<path> <name> *<gitUrl> <branch>..."
            ></textarea>
            <div class="modal-action">
                <button
                    class="btn btn-primary flex items-center gap-2"
                    @click.prevent="confirmImport"
                    :disabled="isImporting"
                >
                    <span v-if="isImporting">
                        <span class="loading loading-spinner loading-xs"></span>
                        Importing {{ importProgress.current }} /
                        {{ importProgress.total }}...
                    </span>
                    <span v-else>Import</span>
                </button>
                <button
                    class="btn btn-outline"
                    @click.prevent="showImport = false"
                    :disabled="isImporting"
                >
                    Cancel
                </button>
            </div>
        </div>
    </div>
    <!-- Export Modal -->
    <div
        v-if="showExport"
        class="modal modal-open"
        @click.self="showExport = false"
    >
        <div class="modal-box">
            <h3 class="font-bold text-lg">Export Addons</h3>
            <textarea
                v-model="exportText"
                rows="6"
                class="textarea textarea-bordered w-full mt-2"
                readonly
            ></textarea>
            <div class="modal-action flex gap-2">
                <button class="btn btn-accent" @click.prevent="saveToFile">
                    <Save />Save to File
                </button>
                <button class="btn btn-primary" @click.prevent="copyAndClose">
                    <Copy />Copy & Close
                </button>
                <button
                    class="btn btn-outline"
                    @click.prevent="showExport = false"
                >
                    Close
                </button>
            </div>
        </div>
    </div>
    <!-- About Modal -->
    <div
        v-if="showAbout"
        class="modal modal-open"
        @click.self="showAbout = false"
    >
        <div class="modal-box max-w-md">
            <h3 class="font-bold text-lg flex items-center gap-2">
                <Info class="w-6 h-6" />
                About
            </h3>
            <p class="mt-4">Addon manager using git for version control</p>
            <p class="mt-4">
                Source:
                <a
                    href="https://github.com/sogladev/rs-game-launcher"
                    target="_blank"
                    class="link link-primary"
                >
                    https://github.com/sogladev/rs-game-launcher
                </a>
            </p>
            <p class="mt-4">
                Author:
                <a
                    href="https://github.com/sogladev"
                    target="_blank"
                    class="link link-primary"
                    >Sogladev</a
                >
            </p>
            <div class="modal-action">
                <button class="btn btn-primary" @click="showAbout = false">
                    Close
                </button>
            </div>
        </div>
    </div>
    <!-- Theme Modal -->
    <div
        v-if="showTheme"
        class="modal modal-open"
        @click.self="showTheme = false"
    >
        <div class="modal-box max-w-xs">
            <h3 class="font-bold text-lg flex items-center gap-2">
                <Palette class="w-6 h-6" /> Theme
            </h3>
            <ThemeController />
            <div class="modal-action">
                <button class="btn btn-primary" @click="showTheme = false">
                    Close
                </button>
            </div>
        </div>
    </div>
    <!-- Report Issue Modal -->
    <div v-if="showLog" class="modal modal-open" @click.self="showLog = false">
        <div class="modal-box max-w-xl">
            <h3 class="font-bold text-lg mb-2">Report Issue</h3>
            <p class="mb-2 text-sm text-base-content">
                This box displays issues that occur during the app. Copy the log
                below when sharing it in a GitHub issue.
            </p>
            <textarea
                readonly
                placeholder="No issues logged yet :)"
                rows="10"
                class="textarea textarea-bordered w-full mb-4 font-mono text-xs"
                >{{ getIssueLog() }}</textarea
            >
            <div class="modal-action flex gap-2">
                <button class="btn btn-accent" @click="saveLogAndClose">
                    <Save />Save Log
                </button>
                <button class="btn btn-primary" @click="copyLogAndClose">
                    <Copy />Copy Log & Close
                </button>
                <button class="btn btn-outline" @click="showLog = false">
                    Close
                </button>
            </div>
        </div>
    </div>
</template>
