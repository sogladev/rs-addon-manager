<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGlobalError } from '@/composables/useGlobalError'
import { AddOnsFolder } from '@bindings/AddOnsFolder'

const { addIssue } = useGlobalError()

const { open, folderPaths, addonFolders } = defineProps<{
    open: boolean
    folderPaths: string[]
    addonFolders: AddOnsFolder[]
}>()

const emit = defineEmits<{ (event: 'update:open', value: boolean): void }>()

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

const existingRepoUrl = computed(() => {
    if (!selectedDirectory.value || !gitUrl.value) return false
    if (!Array.isArray(addonFolders) || !addonFolders.length) return false
    const folder = addonFolders.find((f) => f.path === selectedDirectory.value)
    if (!folder || !Array.isArray(folder.repositories)) return false
    return folder.repositories.some(
        (repo) => repo.repoUrl === gitUrl.value.trim()
    )
})

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

const directoryTouched = ref(false)

const handleClone = async () => {
    directoryTouched.value = true
    if (!isGitUrlValid.value || !selectedDirectory.value) return

    try {
        emit('update:open', false)
        await invoke('install_addon_cmd', {
            url: trimmedGitUrl.value,
            path: selectedDirectory.value,
        })
        console.log('Addon cloned successfully')
        gitUrl.value = ''
        isGitUrlValid.value = null
        directoryTouched.value = false
        errorMessage.value = ''
    } catch (err: unknown) {
        console.error('Failed to clone addon', err)
        errorMessage.value = err instanceof Error ? err.message : String(err)
        addIssue('Failed to clone', err)
        // Reopen modal on error so user can retry
        emit('update:open', true)
    }
}
</script>
<template>
    <dialog :open="open" class="modal">
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
            <div class="form-control mb-2">
                <label class="label">
                    <span class="label-text">Install Directory</span>
                </label>
                <select
                    v-model="selectedDirectory"
                    :class="[
                        'select select-bordered w-full',
                        existingRepoUrl ? 'border-error' : '',
                    ]"
                >
                    <option value="" disabled>Select directory</option>
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
                <div
                    :class="{
                        visible: existingRepoUrl === true,
                        invisible: !existingRepoUrl || existingRepoUrl !== true,
                    }"
                    class="text-error text-xs mt-1"
                >
                    An addon with this repository URL already exists in the
                    selected directory
                </div>
            </div>
            <div v-if="errorMessage" class="text-error text-xs mt-1">
                {{ errorMessage }}
            </div>
            <div class="modal-action">
                <button
                    class="btn btn-primary"
                    @click="handleClone"
                    :disabled="
                        !isGitUrlValid || !selectedDirectory || existingRepoUrl
                    "
                >
                    Clone
                </button>
                <button class="btn" @click="emit('update:open', false)">
                    Cancel
                </button>
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button @click="emit('update:open', false)">close</button>
        </form>
    </dialog>
</template>
