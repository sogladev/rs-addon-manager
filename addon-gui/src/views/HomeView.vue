<script setup lang="ts">
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'
import { onMounted, ref } from 'vue'
import { useTimeoutFn } from '@vueuse/core'
import { Plus, Ellipsis } from 'lucide-vue-next'
import { FileText, Globe, Wrench, Trash2 } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'

import AddonCollapse from '@/components/AddonCollapse.vue'
import AddonCloneModal from '@/components/AddonCloneModal.vue'

const showAddModal = ref(false)

// Handle toggling a subAddon (enable/disable)
function handleToggleSubAddon(repo: AddonRepository, addon: Addon) {
    // @todo: fix toggle sub addon
    console.log(
        `Toggled subAddon ${addon.name} enabled: ${addon.enabled} in repo ${repo.repoName}`
    )
    // TODO: Implement backend call or state update if needed
}

type InstallKey = { path: string; url: string }
type InstallEventPayload = {
    key: InstallKey
    event:
        | { Progress: { current: number; total: number } }
        | { Status: string }
        | { Warning: string }
        | { Error: string }
}

onMounted(async () => {
    listen<InstallEventPayload>('install-event', ({ payload }) => {
        console.debug('[install-event]', payload)
        installStatus.value.active = true

        const event = payload.event
        if ('Progress' in event) {
            const { current, total } = event.Progress
            installStatus.value.progress = { current, total }
            installStatus.value.step = undefined
            installStatus.value.error = undefined
            installStatus.value.warning = undefined
        } else if ('Status' in event) {
            installStatus.value.step = event.Status
            installStatus.value.progress = undefined
            installStatus.value.error = undefined
            installStatus.value.warning = undefined
        } else if ('Warning' in event) {
            installStatus.value.warning = event.Warning
            installStatus.value.error = undefined
        } else if ('Error' in event) {
            installStatus.value.error = event.Error
            installStatus.value.warning = undefined
        } else {
            console.warn('[install-event] Unknown event type:', payload)
        }
    })

    // Load initial addon data from backend
    try {
        const folders = await invoke<AddOnsFolder[]>('refresh_addon_data')
        addonFolders.value = folders
    } catch (err) {
        console.error('Failed to load addon data:', err)
    }
})

const installStatus = ref<{
    progress?: { current: number; total: number }
    step?: string
    error?: string
    warning?: string
    active: boolean
}>({ active: false })

const addonFolders = ref<AddOnsFolder[]>([])
const folderPaths = computed(() => addonFolders.value.map((f) => f.path))

const addAddonDirectory = async () => {
    try {
        const path = await open({
            multiple: false,
            directory: true,
        })
        if (path) {
            console.debug('Adding path:', path)
        } else {
            console.debug('No directory selected')
        }
    } catch (error) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error selecting directory:', errorMessage)
    }
}

const isOpening = ref(false)

const search = ref('')
import { computed } from 'vue'
import { listen } from '@tauri-apps/api/event'

// Compute filtered folders and their addons based on search
const filteredFolders = computed(() => {
    const term = search.value.trim().toLowerCase()
    if (!term) {
        return addonFolders.value
    }
    // Filter folders by whether any of their addons match
    return addonFolders.value.filter((folder) => {
        const filteredRepos = folder.repositories.filter(
            (repo) =>
                repo.repoName.toLowerCase().includes(term) ||
                repo.owner.toLowerCase().includes(term) ||
                repo.addons.some((addon) =>
                    addon.name.toLowerCase().includes(term)
                )
        )
        return filteredRepos.length > 0
    })
})

const FOLDER_REVEAL_TIMEOUT_IN_MS = 800
function handleOpenPath(path: string) {
    if (isOpening.value) return
    isOpening.value = true
    revealItemInDir(path)
    useTimeoutFn(() => {
        isOpening.value = false
    }, FOLDER_REVEAL_TIMEOUT_IN_MS)
}

function requestDeleteFolder(path: string) {
    folderToDelete.value = path
    showDeleteModal.value = true
}

const showDeleteModal = ref(false)
const folderToDelete = ref<string | null>(null)
// --- Addon deletion modal state and logic ---
const showAddonDeleteModal = ref(false)
const addonToDelete = ref<AddonRepository | null>(null)
const folderOfAddonToDelete = ref<string | null>(null)

function requestAddonDeletion(folderPath: string, addon: AddonRepository) {
    folderOfAddonToDelete.value = folderPath
    addonToDelete.value = addon
    showAddonDeleteModal.value = true
}

// Remove folder from store and UI, and optionally notify backend
async function confirmDeleteFolder() {
    if (folderToDelete.value) {
        // @todo: Add a purge option to remove the .addonmanager folder and cleanup symbolic links in the AddOns folder
        addonFolders.value = addonFolders.value.filter(
            (f) => f.path !== folderToDelete.value
        )
        // @todo: Persist the removal of this folder path to storage
        showDeleteModal.value = false
        folderToDelete.value = null
    }
}

async function confirmAddonDelete() {
    if (folderOfAddonToDelete.value && addonToDelete.value) {
        try {
            await invoke('delete_addon', {
                path: folderOfAddonToDelete.value,
                url: addonToDelete.value.repoUrl,
            })
            // @todo: Backend will send an event to update the UI
            // The backend should emit an event to refresh the data
        } catch (err) {
            console.error('Failed to delete addon', err)
        }
    }
    showAddonDeleteModal.value = false
    addonToDelete.value = null
    folderOfAddonToDelete.value = null
}

function cancelDeleteFolder() {
    showDeleteModal.value = false
    folderToDelete.value = null
}

function cancelAddonDelete() {
    showAddonDeleteModal.value = false
    addonToDelete.value = null
    folderOfAddonToDelete.value = null
}
</script>

<template>
    <!-- <MainLayout> -->
    <div class="flex flex-col h-full gap-4">
        <!-- top bar: navbar + controls row -->
        <div
            class="sticky top-0 z-10 bg-base-200 rounded-box mb-2 flex flex-col gap-0"
        >
            <div class="navbar justify-center">
                <div class="navbar-center w-full flex justify-center">
                    <div class="tabs tabs-box text-lg">
                        <button class="tab tab-active px-8 py-2">addons</button>
                        <button class="tab px-8 py-2">about</button>
                        <button class="tab px-8 py-2">config</button>
                    </div>
                </div>
            </div>
            <div
                class="flex flex-wrap items-center gap-2 bg-base-200 pb-2 pt-2 px-2"
            >
                <button class="btn btn-primary">Update All</button>
                <!-- <button class="btn btn-primary"> <ArrowDownToLine /> </button> -->
                <!-- <button class="btn btn-primary"> <ArrowDownToLine /> Update All </button> -->

                <button class="btn btn-secondary">Refresh</button>
                <!-- <button class="btn btn-secondary"> <RefreshCcw /> Refresh </button> -->
                <!-- <button class="btn btn-secondary"> <RefreshCcw /> </button> -->

                <input
                    v-model="search"
                    class="input input-bordered flex-1 max-w-xs ml-auto"
                    placeholder="Search installed addons..."
                    type="search"
                />
                <button
                    class="btn btn-accent ml-2"
                    @click="showAddModal = true"
                >
                    <Plus />
                    Add addon
                </button>
            </div>
        </div>

        <!-- feedback bar-->
        <div
            v-if="
                installStatus.active &&
                (installStatus.step ||
                    installStatus.progress ||
                    installStatus.warning ||
                    installStatus.error)
            "
            class="w-full p-2 bg-base-300 rounded-box mb-2"
        >
            <div v-if="installStatus.step" class="text-base-content mb-1">
                {{ installStatus.step }}
            </div>
            <div v-if="installStatus.progress">
                <progress
                    class="progress progress-primary w-full"
                    :value="installStatus.progress.current"
                    :max="installStatus.progress.total"
                ></progress>
                <span>
                    {{ installStatus.progress.current }} /
                    {{ installStatus.progress.total }}
                </span>
            </div>
            <div v-if="installStatus.warning" class="alert alert-warning mt-2">
                {{ installStatus.warning }}
            </div>
            <div v-if="installStatus.error" class="alert alert-error mt-2">
                {{ installStatus.error }}
            </div>
        </div>

        <AddonCloneModal
            v-model:open="showAddModal"
            :folderPaths="folderPaths"
        />

        <!-- Delete Folder Confirmation Modal -->
        <dialog :open="showDeleteModal" class="modal">
            <div class="modal-box">
                <h3 class="font-bold text-lg mb-4">Delete Folder</h3>
                <p>
                    Are you sure you want to stop managing
                    <span class="font-mono">{{ folderToDelete }}</span
                    >?<br />
                    <strong
                        >This will not delete the AddOns folder, any installed
                        addons, or remove the
                        <code>.addonmanager</code> folder.</strong
                    ><br />
                    The folder will simply be removed from the list of managed
                    directories.
                </p>
                <div class="modal-action">
                    <button class="btn btn-error" @click="confirmDeleteFolder">
                        Delete
                    </button>
                    <button class="btn" @click="cancelDeleteFolder">
                        Cancel
                    </button>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop">
                <button @click="cancelDeleteFolder">close</button>
            </form>
        </dialog>

        <!-- Paths and Addons list -->
        <div class="flex flex-col gap-4 overflow-y-auto p-4">
            <AddonCollapse
                v-for="folder in filteredFolders"
                :key="folder.path"
                :path="folder.path"
                :isOpening="isOpening"
                :isValid="folder.isValid"
                @open-folder="handleOpenPath"
                @delete-folder="requestDeleteFolder"
            >
                <div class="flex flex-col gap-1.5 mt-2">
                    <div
                        v-for="repo in folder.repositories"
                        :key="repo.repoUrl + (repo.currentBranch || '')"
                        class="card card-bordered bg-base-100 flex-row items-center p-2"
                    >
                        <div class="flex flex-1 flex-col gap-1 p-2">
                            <span class="font-semibold">{{
                                repo.repoName
                            }}</span>
                            <span class="text-xs text-base-content/60">{{
                                repo.owner
                            }}</span>
                            <span
                                v-if="repo.repoRef"
                                class="text-xs text-base-content/50"
                                >Installed: {{ repo.repoRef }}</span
                            >
                            <div
                                v-if="repo.addons && repo.addons.length"
                                class="mt-1"
                            >
                                <!-- <span class="text-xs font-semibold mb-1 block" -->
                                <!-- >Sub-addons:</span -->
                                <!-- > -->
                                <ul class="ml-2 flex flex-col gap-1">
                                    <li
                                        v-for="addon in repo.addons"
                                        :key="addon.name"
                                        class="flex items-center gap-2"
                                    >
                                        <input
                                            type="checkbox"
                                            class="checkbox checkbox-sm"
                                            v-model="addon.enabled"
                                            @change="
                                                handleToggleSubAddon(
                                                    repo,
                                                    addon
                                                )
                                            "
                                        />
                                        <span class="font-mono text-xs">{{
                                            addon.name
                                        }}</span>
                                        <span
                                            v-if="!addon.enabled"
                                            class="badge badge-xs badge-error"
                                            >disabled</span
                                        >
                                    </li>
                                </ul>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <div class="w-40">
                                <select
                                    class="select select-bordered select-sm w-full truncate"
                                    :value="repo.currentBranch"
                                    @change="
                                        (e) => {
                                            const target =
                                                e.target as HTMLSelectElement | null
                                            if (!target) return
                                            const newBranch = target.value
                                            console.log(
                                                'Branch change requested:',
                                                newBranch,
                                                'for repo:',
                                                repo.repoUrl
                                            )
                                        }
                                    "
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
                                class="btn btn-sm btn-primary"
                                @click="console.log('Update clicked', repo)"
                            >
                                Update
                            </button>
                            <div class="dropdown dropdown-end">
                                <button
                                    tabindex="0"
                                    class="btn btn-sm btn-ghost"
                                >
                                    <Ellipsis />
                                </button>
                                <ul
                                    tabindex="0"
                                    class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-44"
                                >
                                    <li>
                                        <button
                                            class="flex items-center gap-2"
                                            @click="
                                                console.log(
                                                    'Readme clicked',
                                                    repo
                                                )
                                            "
                                        >
                                            <FileText class="w-4 h-4" />
                                            Readme
                                        </button>
                                    </li>
                                    <li>
                                        <button
                                            class="flex items-center gap-2"
                                            @click="
                                                console.log(
                                                    'Website clicked',
                                                    repo
                                                )
                                            "
                                        >
                                            <Globe class="w-4 h-4" />
                                            Website
                                        </button>
                                    </li>
                                    <li>
                                        <button
                                            class="flex items-center gap-2"
                                            @click="
                                                console.log(
                                                    'Repair clicked',
                                                    repo
                                                )
                                            "
                                        >
                                            <Wrench class="w-4 h-4" />
                                            Repair
                                        </button>
                                    </li>
                                    <li>
                                        <button
                                            class="flex items-center gap-2 text-error"
                                            @click="
                                                requestAddonDeletion(
                                                    folder.path,
                                                    repo
                                                )
                                            "
                                        >
                                            <Trash2 class="w-4 h-4" />
                                            Delete
                                        </button>
                                    </li>
                                </ul>
                                <!-- Addon Delete Confirmation Modal -->
                                <dialog
                                    :open="showAddonDeleteModal"
                                    class="modal"
                                >
                                    <div class="modal-box">
                                        <h3 class="font-bold text-lg mb-4">
                                            Delete Addon
                                        </h3>
                                        <p>
                                            Are you sure you want to delete
                                            addon
                                            <span class="font-mono">{{
                                                addonToDelete?.repoName
                                            }}</span>
                                            from directory
                                            <span class="font-mono">{{
                                                folderOfAddonToDelete
                                            }}</span
                                            >?
                                        </p>
                                        <div class="modal-action">
                                            <button
                                                class="btn btn-error"
                                                @click="confirmAddonDelete"
                                            >
                                                Delete
                                            </button>
                                            <button
                                                class="btn"
                                                @click="cancelAddonDelete"
                                            >
                                                Cancel
                                            </button>
                                        </div>
                                    </div>
                                    <form
                                        method="dialog"
                                        class="modal-backdrop"
                                    >
                                        <button @click="cancelAddonDelete">
                                            close
                                        </button>
                                    </form>
                                </dialog>
                            </div>
                        </div>
                    </div>
                </div>
            </AddonCollapse>
            <!-- Add addon directory entry -->
            <button
                class="btn btn-outline btn-accent mt-2 self-start"
                @click="addAddonDirectory"
            >
                Add addon directory
            </button>
        </div>
    </div>
</template>

<style scoped></style>
