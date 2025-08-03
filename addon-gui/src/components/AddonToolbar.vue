<script setup lang="ts">
import { Menu, Copy, Download, Save } from 'lucide-vue-next'
import TimeoutButton from '@/components/TimeoutButton.vue'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

import type { AddOnsFolder } from '@bindings/AddOnsFolder'

const props = defineProps<{
    search: string
    hasUpdates: boolean
    outOfDateCount: number
    folders: AddOnsFolder[]
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

function confirmImport() {
    // Each line: <path> <addonName> *<gitUrl> <branch>
    // Skip header or comment lines
    if (!importText.value.trim()) return
    importText.value.split(/\r?\n/).forEach((line) => {
        if (
            !line.trim() ||
            line.trim().startsWith('//') ||
            line.trim().startsWith('#')
        )
            return
        const parts = line.trim().split(/\s+/)
        if (parts.length >= 4) {
            const folderPath = parts[0]
            const gitUrl = parts[2].startsWith('*')
                ? parts[2].slice(1)
                : parts[2]
            const branch = parts[3]
            const alreadyManaged = props.folders.some?.(
                (f) => f.path === folderPath
            )
            if (alreadyManaged) {
                invoke('install_addon_cmd', {
                    url: gitUrl,
                    path: folderPath,
                    branch,
                }).catch((e) => console.error('Import install failed:', e))
            } else {
                invoke('add_addon_directory', { path: folderPath })
                    .then(() => {
                        invoke('install_addon_cmd', {
                            url: gitUrl,
                            path: folderPath,
                            branch,
                        }).catch((e) =>
                            console.error('Import install failed:', e)
                        )
                    })
                    .catch((e) =>
                        console.error('Failed to add addon directory:', e)
                    )
            }
        }
    })
    importText.value = ''
    showImport.value = false
}

function handleExport() {
    // Generate export lines with header comment
    const header = '// Each line: <path> <addonName> *<gitUrl> <branch>'
    const lines: string[] = []
    props.folders.forEach((folder) => {
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
function copyAndClose() {
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
            await writeTextFile({ path, contents: exportText.value })
        }
        showExport.value = false
    } catch (err) {
        console.error('Save failed:', err)
        showExport.value = false
    }
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
                <span v-if="hasUpdates">Update All</span>
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
            <div class="dropdown dropdown-end">
                <button tabindex="0" class="btn btn-ghost btn-square">
                    <Menu />
                </button>
                <ul
                    tabindex="0"
                    class="dropdown-content menu shadow bg-base-100 rounded-box w-52"
                >
                    <li>
                        <button @click="showImport = true">Import</button>
                    </li>
                    <li>
                        <button @click="handleExport">Export</button>
                    </li>
                    <li>
                        <a
                            href="https://github.com/sogladev/rs-game-launcher"
                            target="_blank"
                        >
                            About
                        </a>
                    </li>
                </ul>
            </div>
        </div>
        <span v-if="outOfDateCount > 0" class="badge badge-warning">
            {{ outOfDateCount }} addon{{ outOfDateCount > 1 ? 's' : '' }} need
            update
        </span>
    </div>
    <!-- Import/Addons Modal -->
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
                <button class="btn btn-primary" @click.prevent="confirmImport">
                    Import
                </button>
                <button
                    class="btn btn-outline"
                    @click.prevent="showImport = false"
                >
                    Cancel
                </button>
            </div>
        </div>
    </div>
    <!-- Export/Addons Modal -->
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
</template>
