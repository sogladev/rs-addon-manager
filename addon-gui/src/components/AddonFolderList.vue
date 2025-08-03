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
    isOpening: boolean
}>()

const emit = defineEmits<{
    'open-folder': [path: string]
    'delete-folder': [path: string]
    'delete-addon': [folderPath: string, addon: AddonRepository]
    'toggle-addon': [repo: AddonRepository, addon: Addon]
    'branch-change': [repo: AddonRepository, branch: string]
    'install-repo': [folderPath: string, repo: AddonRepository]
    'update-repo': [folderPath: string, repo: AddonRepository]
    'repo-readme': [repo: AddonRepository]
    'repo-website': [repo: AddonRepository]
    'repo-repair': [repo: AddonRepository]
    'add-directory': []
}>()

// Compute filtered folders and their addons based on search
const filteredFolders = computed(() => {
    const term = props.search.trim().toLowerCase()
    if (!term) {
        return props.folders
    }
    // Filter folders by whether any of their addons match
    return props.folders.filter((folder) => {
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

function handleToggleAddon(repo: AddonRepository, addon: Addon) {
    emit('toggle-addon', repo, addon)
}

function handleBranchChange(repo: AddonRepository, branch: string) {
    emit('branch-change', repo, branch)
}

function handleInstallRepo(repo: AddonRepository, folderPath: string) {
    emit('install-repo', folderPath, repo)
}

function handleUpdateRepo(repo: AddonRepository, folderPath: string) {
    emit('update-repo', folderPath, repo)
}

function handleRepoReadme(repo: AddonRepository) {
    emit('repo-readme', repo)
}

function handleRepoWebsite(repo: AddonRepository) {
    emit('repo-website', repo)
}

function handleRepoRepair(repo: AddonRepository) {
    emit('repo-repair', repo)
}

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
            :isOpening="isOpening"
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
                    @toggle-addon="handleToggleAddon(repo, $event)"
                    @branch-change="handleBranchChange(repo, $event)"
                    @install="handleInstallRepo(repo, folder.path)"
                    @update="handleUpdateRepo(repo, folder.path)"
                    @readme="handleRepoReadme(repo)"
                    @website="handleRepoWebsite(repo)"
                    @repair="handleRepoRepair(repo)"
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
