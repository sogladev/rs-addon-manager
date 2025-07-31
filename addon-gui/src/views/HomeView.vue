<script setup lang="ts">
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'
import { onMounted, ref } from 'vue'
import { useTimeoutFn } from '@vueuse/core'
import { Plus, Ellipsis } from 'lucide-vue-next'
import { FileText, Globe, Wrench, Trash2 } from 'lucide-vue-next'
import AddonCollapse from '@/components/AddonCollapse.vue'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'

const STORE_FILE = 'addon-manager.json'
const STORE_KEY = 'addon-directories'

type StoredAddonDirectory = {
    path: string
    isValid: boolean
}

async function loadAddonDirectoriesFromStore(): Promise<
    StoredAddonDirectory[]
> {
    try {
        const store = await load(STORE_FILE)
        const dirs = await store.get<StoredAddonDirectory[]>(STORE_KEY)
        return Array.isArray(dirs) ? dirs : []
    } catch (error: any) {
        console.error('Failed to load addon directories from store:', error)
        return []
    }
}

async function saveAddonDirectoriesToStore(dirs: StoredAddonDirectory[]) {
    try {
        const store = await load(STORE_FILE)
        await store.set(STORE_KEY, dirs)
        await store.save()
    } catch (error) {
        console.error('Failed to save addon directories to store:', error)
    }
}

async function isValidGitUrl(url: string): Promise<boolean> {
    return await invoke<boolean>('is_valid_repo_url', { url })
}

// @todo: Uncomment auto filling
// const gitUrl = ref('')
const gitUrl = ref('https://github.com/sogladev/addon-335-train-all-button.git')
const isGitUrlValid = ref<boolean | null>(true)
// const isGitUrlValid = ref<boolean | null>(null)

watch(gitUrl, async () => {
    if (!trimmedGitUrl.value) {
        isGitUrlValid.value = null
        return
    }
    isGitUrlValid.value = await isValidGitUrl(trimmedGitUrl.value)
})

type SubAddon = {
    name: string // symlink name in AddOns
    dir: string // relative path inside repo
    names: string[] // normalized base names from .toc
    toc_files: string[] // .toc file names
    enabled: boolean
}

enum InstallStatus {
    Pending = 'pending',
    Installing = 'installing',
    Success = 'success',
    Error = 'error',
}

type AddonMeta = {
    repo_url: string // git repository URL
    owner: string // repository owner
    repo_name: string // repository name
    branch?: string | null // branch
    repo_ref?: string | null // commit hash or tag
    sub_addons: SubAddon[]
    // --- UI-only fields for install state ---
    installStatus?: InstallStatus
    installProgress?: { current: number; total: number }
    installError?: string
    installStep?: string
}

type AddonFolder = {
    path: string // absolute path to AddOns folder
    isValid: boolean
    addons: AddonMeta[]
}

import { listen } from '@tauri-apps/api/event'

type InstallKey = { path: string; repo_url: string }
type InstallEventPayload = { key: InstallKey } & (
    | { Progress: { current: number; total: number } }
    | { Status: string }
    | { Warning: string }
    | { Error: string }
)

onMounted(async () => {
    listen<InstallEventPayload>('install-event', ({ payload }) => {
        const { key, ...event } = payload
        console.debug('[install-event]', payload)
        installStatus.value.active = true
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
            console.warn('Unknown event type:', event)
        }
    })

    listen<AddonFolder>('addon-manager-data-updated', ({ payload }) => {
        const idx = addonFolders.value.findIndex((f) => f.path === payload.path)
        if (idx !== -1) {
            addonFolders.value[idx] = {
                ...addonFolders.value[idx],
                ...payload,
                isValid: addonFolders.value[idx].isValid,
            }
        } else {
            addonFolders.value.push(payload)
        }
    })

    // On startup, load managed directories from store and request backend to load them
    const dirs = await loadAddonDirectoriesFromStore()
    console.debug('[startup] loaded addon directories:', dirs)
    for (const entry of dirs) {
        if (!entry || !entry.path) {
            console.warn(
                '[startup] Skipping invalid entry in addon directories:',
                entry
            )
            continue
        }
        console.debug(
            '[startup] requesting get_addon_manager_data for',
            entry.path
        )
        await invoke('get_addon_manager_data', { path: entry.path })
    }
})

const installStatus = ref<{
    progress?: { current: number; total: number }
    step?: string
    error?: string
    warning?: string
    active: boolean
}>({ active: false })

import { watch } from 'vue'

const addonFolders = ref<AddonFolder[]>([])
const folderPaths = computed(() => addonFolders.value.map((f) => f.path))

const addAddonDirectory = async () => {
    try {
        const path = await open({
            multiple: false,
            directory: true,
        })
        if (path) {
            // Load current from store, add new, save
            let dirs = await loadAddonDirectoriesFromStore()
            // Check if already present
            if (!dirs.some((d) => d.path === path)) {
                const isValid: boolean = await invoke(
                    'is_valid_addons_folder_str',
                    { path }
                )
                dirs.push({ path, isValid })
                await saveAddonDirectoriesToStore(dirs)
                if (!Array.isArray(addonFolders)) {
                    addonFolders = []
                }
                // Only add folder if not present
                if (!addonFolders.some((f) => f.path === path)) {
                    addonFolders.push({ path, isValid, addons: [] })
                }
            }
            // Request backend to load metadata for this directory
            await invoke('get_addon_manager_data', { path })
        } else {
            console.debug('No directory selected')
        }
    } catch (error) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error selecting directory:', errorMessage)
    }
}

const showAddModal = ref(false)
// For the select, we want the path string of the selected folder
const selectedDirectory = ref<string>('')

const isOpening = ref(false)

const search = ref('')
import { computed } from 'vue'

// Compute filtered folders and their addons based on search
const filteredFolders = computed(() => {
    // Defensive: always return an array of AddonFolder objects
    const folders = Array.isArray(addonManagerData.value?.folders)
        ? addonFolders.filter(
              (f) => f && typeof f.path === 'string' && Array.isArray(f.addons)
          )
        : []
    if (!search.value.trim()) {
        // Show all folders, even if they have no addons
        return folders
    }
    const term = search.value.trim().toLowerCase()
    // Filter folders by whether any of their addons match
    return folders
        .map((folder) => {
            const filteredAddons = Array.isArray(folder.addons)
                ? folder.addons.filter(
                      (addon) =>
                          addon.repo_name?.toLowerCase().includes(term) ||
                          addon.owner?.toLowerCase().includes(term) ||
                          (Array.isArray(addon.sub_addons) &&
                              addon.sub_addons.some((sub) =>
                                  sub.name?.toLowerCase().includes(term)
                              ))
                  )
                : []
            return { ...folder, addons: filteredAddons }
        })
        .filter((folder) => folder.addons.length > 0)
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

const handleClone = async () => {
    showAddModal.value = false
    if (!isGitUrlValid.value) return
    try {
        await invoke('install_addon_cmd', {
            url: trimmedGitUrl.value,
            dir: selectedDirectory.value,
        })
        console.log('Addon cloned successfully')
    } catch (err) {
        console.error('Failed to clone addon', err)
    }
}

const trimmedGitUrl = computed(() => gitUrl.value.trim())

const showDeleteModal = ref(false)
const folderToDelete = ref<string | null>(null)

function requestDeleteFolder(path: string) {
    folderToDelete.value = path
    showDeleteModal.value = true
}

// Request backend to remove folder, update store, backend will emit updated data
async function confirmDeleteFolder() {
    if (folderToDelete.value) {
        let dirs = await loadAddonDirectoriesFromStore()
        dirs = dirs.filter((d) => d.path !== folderToDelete.value)
        await saveAddonDirectoriesToStore(dirs)
        // @todo: Add a purge option to remove the .addonmanager folder and cleanup symbolic links in the AddOns folder
        // await invoke('remove_addon_directory', { path: folderToDelete.value })
    }
    showDeleteModal.value = false
    folderToDelete.value = null
}

function cancelDeleteFolder() {
    showDeleteModal.value = false
    folderToDelete.value = null
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

        <!-- Add Addon Modal -->
        <dialog :open="showAddModal" class="modal">
            <div class="modal-box">
                <h3 class="font-bold text-lg mb-4">Clone Repository</h3>
                <div class="form-control mb-2">
                    <label class="label">
                        <span class="label-text">Clone using the web URL</span>
                    </label>
                    <input
                        v-model="gitUrl"
                        class="input input-bordered w-full"
                        placeholder="https://github.com/user/repo.git"
                    />
                    <div
                        :class="{
                            visible: isGitUrlValid === false && gitUrl,
                            invisible: !gitUrl || isGitUrlValid !== false,
                        }"
                        class="text-error text-xs mt-1"
                    >
                        Please enter a valid HTTPS Git URL ending with
                        <code>.git</code>
                    </div>
                </div>
                <div class="form-control mb-4">
                    <label class="label">
                        <span class="label-text">Install Directory</span>
                    </label>
                    <select
                        v-model="selectedDirectory"
                        class="select select-bordered w-full"
                    >
                        <option value="" disabled>Select directory</option>
                        <option
                            v-for="path in folderPaths"
                            :key="path"
                            :value="path"
                        >
                            {{ path }}
                        </option>
                    </select>
                    <!-- <div :class="{ 'visible': selectedDirectory.isValid === false && gitUrl, 'invisible': !gitUrl || selectedDirectory.isValid !== false }" -->
                    <!-- class="text-error text-xs mt-1"> -->
                    <!-- Please enter a valid HTTPS Git URL ending with <code>.git</code> -->
                    <!-- </div> -->
                </div>
                <div class="modal-action">
                    <button
                        class="btn btn-primary"
                        @click="handleClone"
                        :disabled="!isGitUrlValid"
                    >
                        Clone
                    </button>
                    <button class="btn" @click="showAddModal = false">
                        Cancel
                    </button>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop">
                <button @click="showAddModal = false">close</button>
            </form>
        </dialog>

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
                @open-folder="handleOpenPath"
                @delete-folder="requestDeleteFolder"
            >
                <div class="flex items-center gap-2">
                    <span
                        v-if="folder.isValid === false"
                        class="alert alert-warning alert-soft ml-2"
                    >
                        Warning! Not a valid AddOns directory
                    </span>
                </div>
                <div class="flex flex-col gap-1.5 mt-2">
                    <div
                        v-for="addon in folder.addons"
                        :key="addon.repo_url + (addon.branch || '')"
                        class="card card-bordered bg-base-100 flex-row items-center p-2"
                    >
                        <div class="flex flex-1 flex-col gap-1 p-2">
                            <span class="font-semibold">{{
                                addon.repo_name
                            }}</span>
                            <span class="text-xs text-base-content/60">{{
                                addon.owner
                            }}</span>
                            <span
                                v-if="addon.repo_ref"
                                class="text-xs text-base-content/40"
                                >Installed: {{ addon.repo_ref }}</span
                            >
                            <div
                                v-if="
                                    addon.sub_addons && addon.sub_addons.length
                                "
                                class="mt-1"
                            >
                                <span class="text-xs font-semibold"
                                    >Sub-addons:</span
                                >
                                <ul class="ml-2 list-disc text-xs">
                                    <li
                                        v-for="sub in addon.sub_addons"
                                        :key="sub.name"
                                    >
                                        <span class="font-mono">{{
                                            sub.name
                                        }}</span>
                                        <span
                                            v-if="!sub.enabled"
                                            class="text-error"
                                            >(disabled)</span
                                        >
                                    </li>
                                </ul>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <div class="w-40">
                                <select
                                    class="select select-bordered select-sm w-full truncate"
                                    v-model="addon.branch"
                                >
                                    <option
                                        v-if="addon.branch"
                                        :value="addon.branch"
                                    >
                                        {{ addon.branch }}
                                    </option>
                                    <!-- Optionally, you can add more branch options here if available -->
                                </select>
                            </div>
                            <button
                                v-if="
                                    addon.installStatus ===
                                        InstallStatus.Pending ||
                                    addon.installStatus === InstallStatus.Error
                                "
                                class="btn btn-sm btn-primary"
                                @click="console.log('Update clicked', addon)"
                            >
                                Update
                            </button>
                            <button
                                v-else
                                class="btn btn-sm btn-ghost btn-disabled"
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
                                                    addon
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
                                                    addon
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
                                                    addon
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
                                                console.log(
                                                    'Delete clicked',
                                                    addon
                                                )
                                            "
                                        >
                                            <Trash2 class="w-4 h-4" />
                                            Delete
                                        </button>
                                    </li>
                                </ul>
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
