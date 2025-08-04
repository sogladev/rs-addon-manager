<script setup lang="ts">
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

import type { AddonRepository } from '@bindings/AddonRepository'

import { useGlobalError } from '@/composables/useGlobalError'
import { useAddonData } from '@/composables/useAddonData'
import AddonToolbar from '@/components/AddonToolbar.vue'
import AddonCloneModal from '@/components/AddonCloneModal.vue'
import AddonFolderList from '@/components/AddonFolderList.vue'
import DeleteFolderModal from '@/components/DeleteFolderModal.vue'
import DeleteAddonModal from '@/components/DeleteAddonModal.vue'

const { addonFolders, folderPaths, refreshAddonData } = useAddonData()

const showAddModal = ref(false)
const search = ref('')

const { globalErrorMessage, clearError } = useGlobalError()

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

// Modal state
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
    <!-- gap-4 -->
    <div class="flex flex-col h-full z-10">
        <AddonToolbar
            v-model:search="search"
            :folders="addonFolders"
            :hasUpdates="hasUpdates"
            :outOfDateCount="outOfDateCount"
            @update-all="handleUpdateAll"
            @refresh="refreshAddonData(true)"
            @add-addon="showAddModal = true"
        />

        <!-- Global Error Bar -->
        <div
            v-if="globalErrorMessage"
            class="alert alert-error fixed top-0 left-0 w-full z-50"
        >
            <div class="flex items-center justify-between">
                <span>{{ globalErrorMessage }}</span>
                <button class="btn btn-sm btn-outline ml-4" @click="clearError">
                    Dismiss
                </button>
            </div>
        </div>

        <AddonCloneModal
            v-model:open="showAddModal"
            :folderPaths="folderPaths"
            :addonFolders="addonFolders"
        />

        <DeleteFolderModal
            :open="showDeleteModal"
            :folderPath="folderToDelete"
            @confirm="confirmDeleteAddonDirectory"
            @cancel="cancelDeleteFolder"
        />

        <DeleteAddonModal
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
