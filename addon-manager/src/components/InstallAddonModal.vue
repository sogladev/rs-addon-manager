<script setup lang="ts">
import {
    ADDON_CATALOGUE,
    getAllCategories,
    getCategoryDisplayName,
    getAddonsByCategory,
    searchAddons,
    type CatalogueAddon,
    type AddonCategory,
} from '@/data/addonCatalogue'
import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import { computed, ref, watch } from 'vue'
import { Globe, AlertTriangle, X, Search, ExternalLink } from 'lucide-vue-next'
import { useExternalLink } from '@/composables/useExternalLink'
import { useGlobalError } from '@/composables/useGlobalError'
import { invoke } from '@tauri-apps/api/core'

const { addIssue } = useGlobalError()
const { openWebsite } = useExternalLink()

const {
    open,
    folderPaths,
    addonFolders,
    prefill,
    showSuggestedTab = true,
} = defineProps<{
    open: boolean
    folderPaths: string[]
    addonFolders: AddOnsFolder[]
    prefill?: { gitUrl?: string }
    showSuggestedTab?: boolean
}>()

const emit = defineEmits<{
    (event: 'update:open', value: boolean): void
}>()

// Tab state
type TabType = 'clone' | 'github' | 'suggested'
const activeTab = ref<TabType>('clone')

// Shared state
const selectedDirectory = ref<string>('')

watch(
    () => [open, folderPaths],
    ([modalOpen, paths]) => {
        if (modalOpen) {
            const arr = Array.isArray(paths) ? paths : []
            if (
                !selectedDirectory.value ||
                !arr.includes(selectedDirectory.value)
            ) {
                selectedDirectory.value = arr.length > 0 ? arr[0] : ''
            }
        }
    },
    { immediate: true }
)

// Clone tab state
const gitUrl = ref('')
const isGitUrlValid = ref<boolean | null>(null)
const errorMessage = ref('')

watch(gitUrl, async () => {
    if (!trimmedGitUrl.value) {
        isGitUrlValid.value = null
        return
    }
    isGitUrlValid.value = await isValidGitUrl(trimmedGitUrl.value)
})

async function isValidGitUrl(url: string): Promise<boolean> {
    return await invoke<boolean>('is_valid_repo_url', { url })
}

const trimmedGitUrl = computed(() => gitUrl.value.trim())

const existingRepoUrl = computed(() => {
    if (!selectedDirectory.value || !gitUrl.value) return false
    if (!Array.isArray(addonFolders) || !addonFolders.length) return false
    const folder = addonFolders.find((f) => f.path === selectedDirectory.value)
    if (!folder || !Array.isArray(folder.repositories)) return false
    return folder.repositories.some(
        (repo) => repo.repoUrl === gitUrl.value.trim()
    )
})

// Watch for prefill prop changes and update form
watch(
    () => prefill,
    (newPrefill) => {
        if (newPrefill?.gitUrl && open) {
            gitUrl.value = newPrefill.gitUrl
            activeTab.value = 'clone'
            errorMessage.value = ''
        }
    },
    { immediate: true, deep: true }
)

// GitHub search tab state
const githubSearchQuery = ref('')

const handleGithubSearch = () => {
    const query = githubSearchQuery.value.trim()
    if (!query) return

    // Build GitHub search URL
    const searchUrl = `https://github.com/search?q=${encodeURIComponent(query)}&type=repositories`
    openWebsite(searchUrl)
}

// Suggested tab state
const suggestedSearch = ref('')
const selectedCategory = ref<AddonCategory | 'all'>('all')
const showEpochOnly = ref(false)

const categories = getAllCategories()

const filteredAddons = computed(() => {
    let addons = ADDON_CATALOGUE

    // Filter by category
    if (selectedCategory.value !== 'all') {
        addons = getAddonsByCategory(selectedCategory.value)
    }

    // Filter by Epoch compatibility
    if (showEpochOnly.value) {
        addons = addons.filter((addon) => addon.serverCompat?.projectEpoch)
    }

    // Filter by search query
    if (suggestedSearch.value.trim()) {
        addons = searchAddons(suggestedSearch.value).filter(
            (addon) =>
                (selectedCategory.value === 'all' ||
                    addon.category === selectedCategory.value) &&
                (!showEpochOnly.value || addon.serverCompat?.projectEpoch)
        )
    }

    return [...addons].sort((a, b) => a.name.localeCompare(b.name))
})

// Map addon gitUrl to installed folder path
const installedMap = computed(() => {
    const map = new Map<string, string>()
    for (const folder of addonFolders) {
        for (const repo of folder.repositories) {
            map.set(repo.repoUrl, folder.path)
        }
    }
    return map
})

const getInstalledPath = (addon: CatalogueAddon): string | undefined => {
    return installedMap.value.get(addon.gitUrl)
}

const isInstalled = (addon: CatalogueAddon): boolean => {
    return !!getInstalledPath(addon)
}

// Install handlers
const handleCloneInstall = async () => {
    if (
        !isGitUrlValid.value ||
        !selectedDirectory.value ||
        existingRepoUrl.value
    )
        return

    try {
        emit('update:open', false)
        await invoke('install_addon_cmd', {
            url: trimmedGitUrl.value,
            path: selectedDirectory.value,
        })
        console.log('Addon cloned successfully')
        resetCloneForm()
    } catch (err: unknown) {
        console.error('Failed to clone addon', err)
        errorMessage.value = err instanceof Error ? err.message : String(err)
        addIssue('Failed to clone', err)
        emit('update:open', true)
    }
}

const handleSuggestedInstall = async (addon: CatalogueAddon) => {
    if (!selectedDirectory.value || addon.installable === false) return

    try {
        emit('update:open', false)
        await invoke('install_addon_cmd', {
            url: addon.gitUrl,
            path: selectedDirectory.value,
        })
        console.log('Addon installed successfully:', addon.name)
        resetForms()
    } catch (err: unknown) {
        console.error('Failed to install addon', err)
        errorMessage.value = err instanceof Error ? err.message : String(err)
        addIssue('Failed to install addon', err)
        emit('update:open', true)
    }
}

const resetCloneForm = () => {
    gitUrl.value = ''
    isGitUrlValid.value = null
    errorMessage.value = ''
}

const resetSuggestedForm = () => {
    suggestedSearch.value = ''
    selectedCategory.value = 'all'
    showEpochOnly.value = false
}

const resetForms = () => {
    resetCloneForm()
    resetSuggestedForm()
    githubSearchQuery.value = ''
}

const closeModal = () => {
    emit('update:open', false)
    resetForms()
}

// Compute if install button should be enabled
const canInstallClone = computed(() => {
    return (
        isGitUrlValid.value && selectedDirectory.value && !existingRepoUrl.value
    )
})
</script>

<template>
    <dialog :open="open" class="modal" @click.self="closeModal">
        <div class="modal-box max-w-4xl h-[90vh] flex flex-col">
            <!-- Header -->
            <div class="flex items-center justify-between mb-4">
                <h3 class="font-bold text-lg">Install Addon</h3>
                <button class="btn btn-sm btn-ghost" @click="closeModal">
                    <X class="w-4 h-4" />
                </button>
            </div>

            <!-- Directory Selection (Shared) -->
            <div class="form-control mb-4">
                <label class="label">
                    <span class="label-text font-semibold"
                        >Install Directory</span
                    >
                    <a
                        v-if="!folderPaths.length"
                        href="#"
                        class="label-text-alt link link-primary"
                        @click.prevent="closeModal"
                    >
                        Manage directories
                    </a>
                </label>
                <select
                    v-model="selectedDirectory"
                    class="select select-bordered w-full"
                    :disabled="!folderPaths.length"
                >
                    <option value="" disabled>
                        {{
                            folderPaths.length
                                ? 'Select directory'
                                : 'No directories configured'
                        }}
                    </option>
                    <option
                        v-for="path in [...folderPaths].sort((a, b) =>
                            a.localeCompare(b)
                        )"
                        :key="path"
                        :value="path"
                    >
                        {{ path }}
                    </option>
                </select>
            </div>

            <!-- Warning for no directories -->
            <div v-if="!folderPaths.length" class="alert alert-warning mb-4">
                <div class="flex items-start gap-3">
                    <AlertTriangle class="w-6 h-6 shrink-0" />
                    <div>
                        <h3 class="font-bold">
                            No addon directories configured
                        </h3>
                        <div class="text-sm">
                            Please add an AddOns directory in the main menu to
                            enable installations.
                        </div>
                    </div>
                </div>
            </div>

            <!-- Tabs -->
            <div role="tablist" class="tabs tabs-box mb-4">
                <a
                    role="tab"
                    class="tab"
                    :class="{ 'tab-active': activeTab === 'clone' }"
                    @click="activeTab = 'clone'"
                >
                    Clone
                </a>
                <a
                    role="tab"
                    class="tab"
                    :class="{ 'tab-active': activeTab === 'github' }"
                    @click="activeTab = 'github'"
                >
                    GitHub Search
                </a>
                <a
                    v-if="showSuggestedTab"
                    role="tab"
                    class="tab"
                    :class="{ 'tab-active': activeTab === 'suggested' }"
                    @click="activeTab = 'suggested'"
                >
                    Suggested
                </a>
            </div>

            <!-- Tab Content -->
            <div class="flex-1 overflow-y-auto">
                <!-- Clone Tab -->
                <div v-show="activeTab === 'clone'" class="space-y-4">
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text"
                                >Clone using the web URL</span
                            >
                        </label>
                        <input
                            v-model="gitUrl"
                            class="input input-bordered w-full"
                            placeholder="https://github.com/user/repo.git"
                            @keyup.enter="handleCloneInstall"
                        />
                        <div
                            v-if="isGitUrlValid === false && gitUrl"
                            class="text-error text-xs mt-1"
                        >
                            Please enter a valid HTTPS Git URL ending with
                            <code>.git</code>
                        </div>
                        <div
                            v-if="existingRepoUrl"
                            class="text-error text-xs mt-1"
                        >
                            An addon with this repository URL already exists in
                            the selected directory
                        </div>
                    </div>

                    <div v-if="errorMessage" class="alert alert-error">
                        <span>{{ errorMessage }}</span>
                    </div>

                    <div class="text-sm text-base-content/70">
                        <p class="mb-2">
                            Enter the HTTPS clone URL of a Git repository
                            containing WoW addons.
                        </p>
                        <p>
                            Example:
                            <code class="text-xs"
                                >https://github.com/user/MyAddon.git</code
                            >
                        </p>
                    </div>

                    <button
                        class="btn btn-primary"
                        @click="handleCloneInstall"
                        :disabled="!canInstallClone"
                    >
                        Install
                    </button>
                </div>

                <!-- GitHub Search Tab -->
                <div v-show="activeTab === 'github'" class="space-y-4">
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text"
                                >Search GitHub for WoW addons</span
                            >
                        </label>
                        <div class="join w-full">
                            <input
                                v-model="githubSearchQuery"
                                class="input input-bordered join-item flex-1"
                                placeholder="e.g., weakauras, dbm, details"
                                @keyup.enter="handleGithubSearch"
                            />
                            <button
                                class="btn btn-primary join-item"
                                @click="handleGithubSearch"
                                :disabled="!githubSearchQuery.trim()"
                            >
                                <Search class="w-4 h-4 mr-2" />
                                Search GitHub
                            </button>
                        </div>
                    </div>

                    <div class="alert alert-info">
                        <div class="flex items-start gap-3">
                            <ExternalLink class="w-5 h-5 shrink-0" />
                            <div class="text-sm">
                                <p class="font-semibold mb-1">
                                    This will open GitHub in your browser
                                </p>
                                <p>
                                    Once you find an addon, copy its HTTPS clone
                                    URL and use the Clone tab to install it.
                                </p>
                            </div>
                        </div>
                    </div>

                    <div class="text-sm text-base-content/70">
                        <p class="font-semibold mb-2">
                            Tips for finding addons:
                        </p>
                        <ul class="list-disc list-inside space-y-1">
                            <li>
                                Search by addon name (e.g., "ElvUI",
                                "WeakAuras")
                            </li>
                            <li>
                                Look for repositories with many stars and recent
                                updates
                            </li>
                            <li>
                                Check the README for installation instructions
                                and compatibility
                            </li>
                            <li>
                                Copy the clone URL (ends with .git) to install
                                via the Clone tab
                            </li>
                        </ul>
                    </div>
                </div>

                <!-- Suggested Tab -->
                <div v-show="activeTab === 'suggested'" class="space-y-4">
                    <!-- Filters -->
                    <div class="flex flex-col sm:flex-row gap-4">
                        <div class="flex-1">
                            <input
                                v-model="suggestedSearch"
                                placeholder="Search suggested addons..."
                                class="input input-bordered w-full"
                                type="search"
                            />
                        </div>
                        <div class="flex items-center gap-2">
                            <select
                                v-model="selectedCategory"
                                class="select select-bordered min-w-48"
                            >
                                <option value="all">All Categories</option>
                                <option
                                    v-for="category in categories"
                                    :key="category"
                                    :value="category"
                                >
                                    {{ getCategoryDisplayName(category) }}
                                </option>
                            </select>
                        </div>
                    </div>

                    <!-- Epoch Filter Toggle -->
                    <div class="form-control">
                        <label class="label cursor-pointer justify-start gap-2">
                            <input
                                type="checkbox"
                                v-model="showEpochOnly"
                                class="checkbox checkbox-sm"
                            />
                            <span class="label-text"
                                >Show Project Epoch addons only</span
                            >
                        </label>
                    </div>

                    <!-- Results count -->
                    <div class="text-sm text-base-content/70">
                        Showing {{ filteredAddons.length }} addon{{
                            filteredAddons.length !== 1 ? 's' : ''
                        }}
                    </div>

                    <!-- Addon List -->
                    <div class="space-y-3">
                        <div
                            v-for="addon in filteredAddons"
                            :key="addon.gitUrl"
                            class="card bg-base-100 border border-base-300 shadow-sm"
                        >
                            <div class="card-body p-4">
                                <div
                                    class="flex items-start justify-between gap-4"
                                >
                                    <div class="flex-1 min-w-0">
                                        <div
                                            class="flex items-center gap-2 mb-2 flex-wrap"
                                        >
                                            <h4
                                                class="font-semibold text-base truncate"
                                            >
                                                {{ addon.name }}
                                            </h4>
                                            <div
                                                class="badge badge-outline badge-sm"
                                            >
                                                {{
                                                    getCategoryDisplayName(
                                                        addon.category
                                                    )
                                                }}
                                            </div>
                                            <div
                                                v-if="
                                                    addon.serverCompat
                                                        ?.projectEpoch
                                                "
                                                class="badge badge-outline badge-sm"
                                            >
                                                Epoch
                                            </div>
                                            <div
                                                v-if="isInstalled(addon)"
                                                class="tooltip"
                                                :data-tip="
                                                    folderPaths.length > 1
                                                        ? `Installed at ${getInstalledPath(addon)}`
                                                        : 'Already installed'
                                                "
                                            >
                                                <span
                                                    class="badge badge-success badge-sm"
                                                >
                                                    Installed
                                                </span>
                                            </div>
                                        </div>

                                        <p
                                            class="text-sm text-base-content/80 mb-2"
                                        >
                                            {{ addon.description }}
                                        </p>

                                        <div
                                            class="flex items-center gap-4 text-xs text-base-content/60"
                                        >
                                            <span v-if="addon.author">{{
                                                addon.author
                                            }}</span>
                                            <button
                                                class="btn btn-ghost btn-xs flex items-center gap-1"
                                                @click="
                                                    openWebsite(addon.gitUrl)
                                                "
                                                title="Open repository"
                                            >
                                                <Globe class="w-3 h-3" />
                                                Repository
                                            </button>
                                        </div>

                                        <div v-if="addon.notes" class="mt-2">
                                            <div
                                                class="alert alert-warning py-2 px-3 text-xs"
                                            >
                                                <span>{{ addon.notes }}</span>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="flex flex-col gap-2">
                                        <button
                                            v-if="addon.installable !== false"
                                            class="btn btn-accent btn-sm"
                                            @click="
                                                handleSuggestedInstall(addon)
                                            "
                                            :disabled="
                                                !selectedDirectory ||
                                                isInstalled(addon)
                                            "
                                        >
                                            Install
                                        </button>
                                        <button
                                            v-else
                                            class="btn btn-secondary btn-sm"
                                            @click="
                                                addon.cloneUrl
                                                    ? openWebsite(
                                                          addon.cloneUrl
                                                      )
                                                    : openWebsite(addon.gitUrl)
                                            "
                                            title="Clone installation required"
                                        >
                                            Clone
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Empty state -->
                        <div
                            v-if="!filteredAddons.length"
                            class="text-center py-12"
                        >
                            <h3
                                class="text-lg font-semibold text-base-content/70 mb-2"
                            >
                                No addons found
                            </h3>
                            <p class="text-base-content/50">
                                {{
                                    suggestedSearch.trim()
                                        ? 'Try adjusting your search or filters'
                                        : 'No addons available with current filters'
                                }}
                            </p>
                        </div>
                    </div>

                    <!-- Attribution Footer -->
                    <div
                        class="text-xs text-base-content/60 border-t border-base-300 pt-3 mt-4"
                    >
                        <p>
                            Addons curated from the
                            <a
                                href="https://project-epoch-wow.fandom.com/wiki/AddOns"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link link-primary"
                            >
                                Project Epoch Wiki
                            </a>
                            and the
                            <a
                                href="https://discord.gg/Px4T8VVZwr"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="link link-primary"
                            >
                                Epoch Addons Discord </a
                            >. This may contain moved, outdated, or broken
                            repository links.
                        </p>
                    </div>
                </div>
            </div>

            <!-- Footer Actions -->
            <div class="border-t border-base-300 pt-4 mt-4">
                <div class="flex justify-end gap-2">
                    <button class="btn btn-ghost" @click="closeModal">
                        Cancel
                    </button>
                </div>
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button @click="closeModal">close</button>
        </form>
    </dialog>
</template>
