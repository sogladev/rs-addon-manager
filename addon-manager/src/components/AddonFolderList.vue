<script setup lang="ts">
import AddonFolderCollapse from '@/components/AddonFolderCollapse.vue'
import AddonRepoCard from '@/components/AddonRepoCard.vue'
import type { AddOnsFolder } from '@bindings/AddOnsFolder'
import type { Addon } from '@bindings/Addon'
import type { AddonRepository } from '@bindings/AddonRepository'
import { computed } from 'vue'

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
                const filteredRepos = folder.repositories.filter((repo) => {
                    const repoName =
                        repo.source.type === 'git'
                            ? repo.source.repo_name
                            : repo.source.folder_name
                    const owner =
                        repo.source.type === 'git'
                            ? repo.source.owner
                            : 'unknown'

                    return (
                        repoName.toLowerCase().includes(term) ||
                        owner.toLowerCase().includes(term) ||
                        repo.addons.some((addon) =>
                            addon.name.toLowerCase().includes(term)
                        )
                    )
                })
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

// Custom sort: out-of-date or updating repos first, then alphabetically
function sortedRepositoriesByUpdate(repositories: AddonRepository[]) {
    function repoPriority(repo: AddonRepository): number {
        // 1: needs update, 2: normal
        if (repo.source.type === 'git') {
            if (
                repo.source.latest_ref &&
                repo.source.repo_ref !== repo.source.latest_ref
            )
                return 1
        }
        return 2
    }

    function getRepoName(repo: AddonRepository): string {
        if (repo.source.type === 'git') {
            return repo.source.repo_name
        }
        return repo.source.folder_name
    }

    // Sort repositories in each folder by priority, then alphabetically
    repositories = repositories.sort((a, b) => {
        const pa = repoPriority(a)
        const pb = repoPriority(b)
        if (pa !== pb) return pa - pb
        return getRepoName(a).localeCompare(getRepoName(b))
    })
    return repositories
}
</script>

<template>
    <div class="flex flex-col gap-4 p-2">
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
                    v-for="(repo, index) in sortedRepositoriesByUpdate(
                        folder.repositories
                    )"
                    :key="`${folder.path}-${index}`"
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
            Add AddOns directory
        </button>
    </div>
</template>
