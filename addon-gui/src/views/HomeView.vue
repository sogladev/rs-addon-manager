<script setup lang="ts">
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useTimeoutFn } from '@vueuse/core'
import { invoke } from '@tauri-apps/api/core'

import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'

import { useAddonData } from '@/composables/useAddonData'
import AddonToolbar from '@/components/AddonToolbar.vue'
import InstallStatusBar from '@/components/InstallStatusBar.vue'
import AddonCloneModal from '@/components/AddonCloneModal.vue'
import AddonFolderList from '@/components/AddonFolderList.vue'
import DeleteFolderModal from '@/components/DeleteFolderModal.vue'
import DeleteAddonModal from '@/components/DeleteAddonModal.vue'

const { addonFolders, folderPaths, installStatus, refreshAddonData } =
    useAddonData()

const showAddModal = ref(false)
const search = ref('')

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

const isOpening = ref(false)

const FOLDER_REVEAL_TIMEOUT_IN_MS = 800
function handleOpenPath(path: string) {
    if (isOpening.value) return
    isOpening.value = true
    revealItemInDir(path)
    useTimeoutFn(() => {
        isOpening.value = false
    }, FOLDER_REVEAL_TIMEOUT_IN_MS)
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

// Event handlers
function handleToggleAddon(repo: AddonRepository, addon: Addon) {
    console.log(
        `Toggled subAddon ${addon.name} enabled: ${addon.enabled} in repo ${repo.repoName}`
    )
}

function handleBranchChange(repo: AddonRepository, branch: string) {
    console.log('Branch change requested:', branch, 'for repo:', repo.repoUrl)
}

function handleUpdateRepo(repo: AddonRepository) {
    console.log('Update clicked', repo)
}

function handleRepoReadme(repo: AddonRepository) {
    console.log('Readme clicked', repo)
}

function handleRepoWebsite(repo: AddonRepository) {
    console.log('Website clicked', repo)
}

function handleRepoRepair(repo: AddonRepository) {
    console.log('Repair clicked', repo)
}
</script>

<template>
    <div class="flex flex-col h-full gap-4">
        <AddonToolbar
            v-model:search="search"
            @update-all="console.log('Update all clicked')"
            @refresh="refreshAddonData"
            @add-addon="showAddModal = true"
        />

        <InstallStatusBar :installStatus="installStatus" />

        <AddonCloneModal
            v-model:open="showAddModal"
            :folderPaths="folderPaths"
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
            :isOpening="isOpening"
            @open-folder="handleOpenPath"
            @delete-folder="requestDeleteAddonDirectory"
            @delete-addon="requestAddonDeletion"
            @toggle-addon="handleToggleAddon"
            @branch-change="handleBranchChange"
            @update-repo="handleUpdateRepo"
            @repo-readme="handleRepoReadme"
            @repo-website="handleRepoWebsite"
            @repo-repair="handleRepoRepair"
            @add-directory="addAddonDirectory"
        />
    </div>
</template>

<style scoped></style>
