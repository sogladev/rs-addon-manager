<script setup lang="ts">
import AddonCatalogueModal from '@/components/AddonCatalogueModal.vue'
import AddonFolderDeleteModal from '@/components/AddonFolderDeleteModal.vue'
import AddonFolderList from '@/components/AddonFolderList.vue'
import AddonGlobalToolbar from '@/components/AddonGlobalToolbar.vue'
import AddonRepoCloneModal from '@/components/AddonRepoCloneModal.vue'
import AddonRepoDeleteModal from '@/components/AddonRepoDeleteModal.vue'
import { useAddonData } from '@/composables/useAddonData'
import { useOperationTracker } from '@/composables/useOperationTracker'
import type { CatalogueAddon } from '@/data/addonCatalogue'
import type { AddonRepository } from '@bindings/AddonRepository'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { computed, onMounted, ref } from 'vue'

const {
    addonFolders,
    folderPaths,
    checkForUpdates,
    checkingForUpdates,
    hasCompletedFirstUpdate,
    operations,
    activeOperationCount,
} = useAddonData()
const { recentlyCompleted } = useOperationTracker()

const showAddModal = ref(false)
const showCatalogueModal = ref(false)
const search = ref('')
const cloneModalPrefill = ref<{ gitUrl?: string; branch?: string }>({})

onMounted(async () => {
    try {
        const theme = await invoke<string>('load_theme')
        if (theme) {
            document.documentElement.setAttribute('data-theme', theme)
        }
    } catch (err) {
        console.error('Failed to load theme', err)
    }
})

const addAddonDirectory = async () => {
    try {
        const path = await open({
            multiple: false,
            directory: true,
        })
        if (path) {
            console.debug('Adding path:', path)
            await invoke('add_addon_directory', { path })
        } else {
            console.debug('No directory selected')
        }
    } catch (error) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error selecting directory:', errorMessage)
    }
}

function handleOpenPath(path: string) {
    revealItemInDir(path)
}

const showDeleteModal = ref(false)
const folderToDelete = ref<string | null>(null)
const showAddonDeleteModal = ref(false)
const addonToDelete = ref<AddonRepository | null>(null)
const folderOfAddonToDelete = ref<string | null>(null)

function requestDeleteAddonDirectory(path: string) {
    folderToDelete.value = path
    showDeleteModal.value = true
}

function requestAddonDeletion(folderPath: string, addon: AddonRepository) {
    folderOfAddonToDelete.value = folderPath
    addonToDelete.value = addon
    showAddonDeleteModal.value = true
}

async function confirmDeleteAddonDirectory() {
    if (folderToDelete.value) {
        await invoke('delete_addon_directory', { path: folderToDelete.value })
        showDeleteModal.value = false
        folderToDelete.value = null
    }
}

async function confirmAddonDelete() {
    if (folderOfAddonToDelete.value && addonToDelete.value) {
        try {
            await invoke('delete_addon_cmd', {
                path: folderOfAddonToDelete.value,
                url: addonToDelete.value.repoUrl,
            })
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

async function handleUpdateAll() {
    console.log('Update all clicked')
    try {
        await invoke('update_all_addons_cmd')
        console.log('Update all completed')
    } catch (error) {
        console.error('Update all failed:', error)
    }
}

function handleCatalogueInstall(addon: CatalogueAddon) {
    // Close catalogue modal and open clone modal with pre-filled data
    showCatalogueModal.value = false
    cloneModalPrefill.value = {
        gitUrl: addon.gitUrl,
        branch: addon.defaultBranch || 'main',
    }
    showAddModal.value = true
}

const hasUpdates = computed(() =>
    addonFolders.value.some((folder) =>
        folder.repositories.some(
            (repo) => repo.latestRef && repo.repoRef !== repo.latestRef
        )
    )
)

const outOfDateCount = computed(() =>
    addonFolders.value.reduce(
        (sum, folder) =>
            sum +
            folder.repositories.filter(
                (repo) => repo.latestRef && repo.repoRef !== repo.latestRef
            ).length,
        0
    )
)
</script>

<template>
    <div class="flex flex-col h-full z-10">
        <AddonGlobalToolbar
            v-model:search="search"
            :folders="addonFolders"
            :hasUpdates="hasUpdates"
            :outOfDateCount="outOfDateCount"
            :checkingForUpdates="checkingForUpdates"
            :hasCompletedFirstUpdate="hasCompletedFirstUpdate"
            :operations="operations"
            :activeOperationCount="activeOperationCount"
            :recentlyCompleted="recentlyCompleted"
            @update-all="handleUpdateAll"
            @refresh="checkForUpdates(true)"
            @add-addon="showAddModal = true"
            @show-catalogue="showCatalogueModal = true"
        />

        <AddonRepoCloneModal
            v-model:open="showAddModal"
            :folderPaths="folderPaths"
            :addonFolders="addonFolders"
            :prefill="cloneModalPrefill"
        />

        <AddonCatalogueModal
            v-model:open="showCatalogueModal"
            :folderPaths="folderPaths"
            :addonFolders="addonFolders"
            @install-addon="handleCatalogueInstall"
        />

        <AddonFolderDeleteModal
            :open="showDeleteModal"
            :folderPath="folderToDelete"
            @confirm="confirmDeleteAddonDirectory"
            @cancel="cancelDeleteFolder"
        />

        <AddonRepoDeleteModal
            :open="showAddonDeleteModal"
            :addon="addonToDelete"
            :folderPath="folderOfAddonToDelete"
            @confirm="confirmAddonDelete"
            @cancel="cancelAddonDelete"
        />

        <AddonFolderList
            :folders="addonFolders"
            :search="search"
            @open-folder="handleOpenPath"
            @delete-folder="requestDeleteAddonDirectory"
            @delete-addon="requestAddonDeletion"
            @add-directory="addAddonDirectory"
        />
    </div>
</template>

<style scoped></style>
