<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
    open: boolean
    folderPaths: string[]
}>()

const emit = defineEmits<{ (event: 'update:open', value: boolean): void }>()

const selectedDirectory = ref<string>('')

watch(
    () => props.open,
    (open) => {
        if (open) {
            if (props.folderPaths.length > 0) {
                if (
                    !selectedDirectory.value ||
                    !props.folderPaths.includes(selectedDirectory.value)
                ) {
                    selectedDirectory.value = props.folderPaths[0]
                }
            } else {
                selectedDirectory.value = ''
            }
        }
    }
)

// @todo: Uncomment auto filling
// const gitUrl = ref('')
const gitUrl = ref('https://github.com/sogladev/addon-335-train-all-button.git')
const isGitUrlValid = ref<boolean | null>(true)
// const isGitUrlValid = ref<boolean | null>(null)

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

const handleClone = async () => {
    if (!isGitUrlValid.value) return
    emit('update:open', false)
    try {
        await invoke('install_addon_cmd', {
            url: trimmedGitUrl.value,
            path: selectedDirectory.value,
        })
        console.log('Addon cloned successfully')
    } catch (err) {
        console.error('Failed to clone addon', err)
    }
}
</script>
<template>
    <!-- Add Addon Modal -->
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
            <div class="form-control mb-4">
                <label class="label">
                    <span class="label-text">Install Directory</span>
                </label>
                <select
                    v-model="selectedDirectory"
                    class="select select-bordered w-full"
                >
                    <option value="" disabled>Select directory</option>
                    <option
                        v-for="path in folderPaths"
                        :key="path"
                        :value="path"
                    >
                        {{ path }}
                    </option>
                </select>
                <!-- <div :class="{ 'visible': selectedDirectory.isValid === false && gitUrl, 'invisible': !gitUrl || selectedDirectory.isValid !== false }" -->
                <!-- class="text-error text-xs mt-1"> -->
                <!-- Please enter a valid HTTPS Git URL ending with <code>.git</code> -->
                <!-- </div> -->
            </div>
            <div class="modal-action">
                <button
                    class="btn btn-primary"
                    @click="handleClone"
                    :disabled="!isGitUrlValid"
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
