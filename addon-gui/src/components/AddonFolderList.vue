<script setup lang="ts">
import { computed } from 'vue'
import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'
import AddonFolderCollapse from '@/components/AddonFolderCollapse.vue'
import AddonRepoCard from '@/components/AddonRepoCard.vue'

const { folders, search } = defineProps<{
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

const filteredFolders = computed(() => {
    const term = search.trim().toLowerCase()
    let foldersToShow = folders.map((folder) => ({
        ...folder,
        repositories: folder.repositories,
    }))
    if (term) {
        foldersToShow = folders
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
    }
    return foldersToShow.sort((a, b) => a.path.localeCompare(b.path))
})

const handleDeleteAddon = (repo: AddonRepository, folderPath: string) => {
    emit('delete-addon', folderPath, repo)
}
</script>

<template>
    <div class="flex flex-col gap-4 overflow-y-auto p-2">
        <AddonFolderCollapse
            v-for="folder in filteredFolders"
            :key="folder.path"
            :path="folder.path"
            :isValid="folder.isValid"
            @open-folder="emit('open-folder', $event)"
            @delete-folder="emit('delete-folder', $event)"
        >
            <div class="flex flex-col">
                <AddonRepoCard
                    v-for="repo in [...folder.repositories].sort((a, b) =>
                        a.repoName.localeCompare(b.repoName)
                    )"
                    :key="repo.repoUrl + (repo.currentBranch || '')"
                    :repo="repo"
                    :folderPath="folder.path"
                    @delete="handleDeleteAddon(repo, folder.path)"
                />
            </div>
        </AddonFolderCollapse>
        <button
            class="btn btn-outline btn-accent mt-2 self-start"
            @click="emit('add-directory')"
        >
            Add addon directory
        </button>
    </div>
</template>
