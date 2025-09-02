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
import { computed, ref } from 'vue'
import {
    Search,
    // Package,
    ExternalLink,
    Download,
    // Filter,
} from 'lucide-vue-next'

const { open, folderPaths, addonFolders } = defineProps<{
    open: boolean
    folderPaths: string[]
    addonFolders: AddOnsFolder[]
}>()

const emit = defineEmits<{
    (event: 'update:open', value: boolean): void
    (event: 'install-addon', addon: CatalogueAddon): void
}>()

const searchQuery = ref('')
const selectedCategory = ref<AddonCategory | 'all'>('all')

const categories = getAllCategories()

const filteredAddons = computed(() => {
    let addons = ADDON_CATALOGUE

    // Filter by category
    if (selectedCategory.value !== 'all') {
        addons = getAddonsByCategory(selectedCategory.value)
    }

    // Filter by search query
    if (searchQuery.value.trim()) {
        addons = searchAddons(searchQuery.value).filter(
            (addon) =>
                selectedCategory.value === 'all' ||
                addon.category === selectedCategory.value
        )
    }

    return addons
})

const handleInstall = (addon: CatalogueAddon) => {
    emit('install-addon', addon)
}

const getAddonStatusMessage = (addon: CatalogueAddon): string | null => {
    if (!folderPaths.length) return 'No addon directories configured'

    // Check if addon is already installed in any folder
    for (const folder of addonFolders) {
        const existingRepo = folder.repositories.find(
            (repo) => repo.repoUrl === addon.gitUrl
        )
        if (existingRepo) {
            return `Already installed in ${folder.path}`
        }
    }

    return null
}

const isAddonInstalled = (addon: CatalogueAddon): boolean => {
    return !!getAddonStatusMessage(addon)?.startsWith('Already installed')
}

const closeModal = () => {
    emit('update:open', false)
    // Reset filters when closing
    searchQuery.value = ''
    selectedCategory.value = 'all'
}
</script>

<template>
    <dialog :open="open" class="modal" @click.self="closeModal">
        <div class="modal-box max-w-4xl h-[80vh] flex flex-col">
            <div class="flex items-center justify-between mb-4">
                <h3 class="font-bold text-lg flex items-center gap-2">
                    <!-- <Package class="w-6 h-6" /> -->
                    Addon Catalogue
                </h3>
                <button
                    class="btn btn-sm btn-circle btn-ghost"
                    @click="closeModal"
                >
                    ✕
                </button>
            </div>

            <!-- Search and Filter -->
            <div class="flex flex-col sm:flex-row gap-4 mb-4">
                <div class="flex-1 relative">
                    <Search
                        class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-base-content/50"
                    />
                    <input
                        v-model="searchQuery"
                        type="text"
                        placeholder="Search addons..."
                        class="input input-bordered w-full pl-10"
                    />
                </div>
                <div class="flex items-center gap-2">
                    <!-- <Filter class="w-4 h-4 text-base-content/70" /> -->
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

            <!-- Results count -->
            <div class="text-sm text-base-content/70 mb-3">
                Showing {{ filteredAddons.length }} addon{{
                    filteredAddons.length !== 1 ? 's' : ''
                }}
            </div>

            <!-- Addon List -->
            <div class="flex-1 overflow-y-auto space-y-3">
                <div
                    v-for="addon in filteredAddons"
                    :key="addon.gitUrl"
                    class="card bg-base-100 border border-base-300 shadow-sm"
                >
                    <div class="card-body p-4">
                        <div class="flex items-start justify-between gap-4">
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2 mb-2">
                                    <h4 class="font-semibold text-lg truncate">
                                        {{ addon.name }}
                                    </h4>
                                    <div class="badge badge-outline badge-sm">
                                        {{
                                            getCategoryDisplayName(
                                                addon.category
                                            )
                                        }}
                                    </div>
                                    <div
                                        v-if="addon.serverCompat?.projectEpoch"
                                        class="badge badge-outline badge-sm"
                                    >
                                        Epoch
                                    </div>
                                </div>

                                <p
                                    class="text-sm text-base-content/80 mb-2 line-clamp-2"
                                >
                                    {{ addon.description }}
                                </p>

                                <div
                                    class="flex items-center gap-4 text-xs text-base-content/60"
                                >
                                    <span v-if="addon.author"
                                        >by {{ addon.author }}</span
                                    >
                                </div>

                                <div v-if="addon.notes" class="mt-2">
                                    <div
                                        class="alert alert-warning py-2 px-3 text-xs"
                                    >
                                        <span>{{ addon.notes }}</span>
                                    </div>
                                </div>

                                <!-- Installation status -->
                                <div
                                    v-if="getAddonStatusMessage(addon)"
                                    class="mt-2"
                                >
                                    <div
                                        :class="[
                                            'alert py-2 px-3 text-xs',
                                            isAddonInstalled(addon)
                                                ? 'alert-success'
                                                : 'alert-info',
                                        ]"
                                    >
                                        <span>{{
                                            getAddonStatusMessage(addon)
                                        }}</span>
                                    </div>
                                </div>
                            </div>

                            <div class="flex flex-col gap-2 min-w-0">
                                <template v-if="addon.installable !== false">
                                    <button
                                        class="btn btn-primary btn-sm"
                                        @click="handleInstall(addon)"
                                        :disabled="!folderPaths.length"
                                        :title="
                                            !folderPaths.length
                                                ? 'Please add an addon directory first'
                                                : undefined
                                        "
                                    >
                                        <Download class="w-4 h-4" />
                                        Install
                                    </button>
                                    <a
                                        :href="
                                            addon.gitUrl.replace(/\.git$/, '')
                                        "
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="btn btn-ghost btn-sm"
                                        title="View repository"
                                    >
                                        <ExternalLink class="w-4 h-4" />
                                    </a>
                                </template>
                                <template v-else>
                                    <button
                                        class="btn btn-secondary btn-sm"
                                        disabled
                                        title="Manual install only"
                                    >
                                        <Download class="w-4 h-4" />
                                        Manual Install
                                    </button>
                                    <a
                                        v-if="addon.manualUrl"
                                        :href="addon.manualUrl"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="btn btn-ghost btn-sm"
                                        title="Download manually"
                                    >
                                        <ExternalLink class="w-4 h-4" />
                                    </a>
                                </template>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Empty state -->
                <div v-if="!filteredAddons.length" class="text-center py-12">
                    <Package
                        class="w-12 h-12 mx-auto text-base-content/30 mb-4"
                    />
                    <h3 class="text-lg font-semibold text-base-content/70 mb-2">
                        No addons found
                    </h3>
                    <p class="text-base-content/50">
                        {{
                            searchQuery.trim()
                                ? 'Try adjusting your search or filter'
                                : 'No addons available in this category'
                        }}
                    </p>
                </div>
            </div>

            <!-- Footer -->
            <div class="border-t border-base-300 pt-4 mt-4">
                <div class="flex justify-between items-center">
                    <p class="text-xs text-base-content/60">
                        <span>Addons curated from the</span>
                        <a
                            href="https://project-epoch-wow.fandom.com/wiki/AddOns"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="link link-primary ml-1"
                            >Project Epoch Wiki</a
                        >
                        <span class="ml-2">and the</span>
                        <a
                            href="https://discord.gg/Px4T8VVZwr"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="link link-primary ml-1"
                            >Epoch Addons Discord</a
                        >
                    </p>
                    <button class="btn btn-ghost btn-sm" @click="closeModal">
                        Close
                    </button>
                </div>
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button @click="closeModal">close</button>
        </form>
    </dialog>
</template>

<style scoped>
.line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}
</style>
