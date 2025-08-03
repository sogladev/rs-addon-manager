<script setup lang="ts">
import { computed } from 'vue'
import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'
import AddonCollapse from '@/components/AddonCollapse.vue'
import AddonRepoCard from '@/components/AddonRepoCard.vue'

const props = defineProps<{
    folders: AddOnsFolder[]
    search: string
}>()

const emit = defineEmits<{
    'open-folder': [path: string]
    'delete-folder': [path: string]
    'delete-addon': [folderPath: string, addon: AddonRepository]
    'toggle-addon': [repo: AddonRepository, addon: Addon]
    'add-directory': []
}>()

// Compute filtered folders and their repositories based on search
const filteredFolders = computed(() => {
    const term = props.search.trim().toLowerCase()
    if (!term) {
        return props.folders.map((folder) => ({
            ...folder,
            repositories: folder.repositories,
        }))
    }
    // For each folder, filter its repositories
    return props.folders
        .map((folder) => {
            const filteredRepos = folder.repositories.filter(
                (repo) =>
                    repo.repoName.toLowerCase().includes(term) ||
                    repo.owner.toLowerCase().includes(term) ||
                    repo.addons.some((addon) =>
                        addon.name.toLowerCase().includes(term)
                    )
            )
            return {
                ...folder,
                repositories: filteredRepos,
            }
        })
        .filter((folder) => folder.repositories.length > 0)
})

function handleDeleteAddon(repo: AddonRepository, folderPath: string) {
    emit('delete-addon', folderPath, repo)
}
</script>

<template>
    <div class="flex flex-col gap-4 overflow-y-auto p-4">
        <AddonCollapse
            v-for="folder in filteredFolders"
            :key="folder.path"
            :path="folder.path"
            :isValid="folder.isValid"
            @open-folder="emit('open-folder', $event)"
            @delete-folder="emit('delete-folder', $event)"
        >
            <div class="flex flex-col gap-1.5 mt-2">
                <AddonRepoCard
                    v-for="repo in folder.repositories"
                    :key="repo.repoUrl + (repo.currentBranch || '')"
                    :repo="repo"
                    :folderPath="folder.path"
                    @delete="handleDeleteAddon(repo, folder.path)"
                />
            </div>
        </AddonCollapse>
        <!-- Add addon directory entry -->
        <button
            class="btn btn-outline btn-accent mt-2 self-start"
            @click="emit('add-directory')"
        >
            Add addon directory
        </button>
    </div>
</template>
