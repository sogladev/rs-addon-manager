<script setup lang="ts">
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { open } from '@tauri-apps/plugin-dialog';
import { ref } from 'vue';
import { useTimeoutFn } from '@vueuse/core';
import { Plus, ArrowDownToLine, Ellipsis, CircleArrowDown, RefreshCcw } from 'lucide-vue-next';
import { FileText, Globe, Wrench, Trash2 } from 'lucide-vue-next';
import AddonCollapse from '@/components/AddonCollapse.vue';
import { invoke } from '@tauri-apps/api/core';

async function isValidGitUrl(url: string): Promise<boolean> {
    return await invoke<boolean>('is_valid_repo_url', { url });
}

const gitUrl = ref('');
const isGitUrlValid = ref<boolean | null>(null);

import { watch } from 'vue';

watch(gitUrl, async () => {
    if (!trimmedGitUrl.value) {
        isGitUrlValid.value = null;
        return;
    }
    isGitUrlValid.value = await isValidGitUrl(trimmedGitUrl.value);
});


const paths = ref([
    {
        path: '/home/jelle/Games',
        addons: [
            { name: 'Addon One', notes: '#.toc notes', branch: 'main', branches: ['main', 'dev', 'release', 'origin/HEAD/verylongbranchanemasaaa'], isUpdateAvailable: false },
            { name: 'Addon Two', notes: '#.toc notes', branch: 'dev', branches: ['main', 'dev', 'release'], isUpdateAvailable: true },
            { name: 'Addon Three', notes: '#.toc notes', branch: 'release', branches: ['main', 'dev', 'release'], isUpdateAvailable: false },
            { name: 'Addon Four', notes: '#.toc notes', branch: 'dev', branches: ['main', 'dev', 'release'], isUpdateAvailable: true },
            { name: 'Addon Five', notes: '#.toc notes', branch: 'dev', branches: ['main', 'dev', 'release'], isUpdateAvailable: false },
            { name: 'Addon Six', notes: '#.toc notes', branch: 'main', branches: ['main', 'dev', 'release'], isUpdateAvailable: false },
            { name: 'Addon Seven', notes: '#.toc notes', branch: 'release', branches: ['main', 'dev', 'release'], isUpdateAvailable: true },
        ]
    },
    {
        path: '/mnt/games/wow/addons',
        addons: [
            { name: 'Addon Eight', notes: '#.toc notes', branch: 'release', branches: ['main', 'release'], isUpdateAvailable: false },
            { name: 'Addon Nine', notes: '#.toc notes', branch: 'main', branches: ['main', 'release'], isUpdateAvailable: true },
            { name: 'Addon Ten', notes: '#.toc notes', branch: 'main', branches: ['main', 'release'], isUpdateAvailable: false },
            { name: 'Addon Eleven', notes: '#.toc notes', branch: 'release', branches: ['main', 'release'], isUpdateAvailable: false },
        ]
    }
])

const addAddonDirectory = async () => {
    try {
        const directory = await open({
            multiple: false,
            directory: true,
        });
        if (directory) {
            // Add the new directory to the paths list if not already present
            if (!paths.value.some(p => p.path === directory)) {
                paths.value.push({ path: directory, addons: [] });
            }
            console.debug(`Selected directory: ${directory}`);
        } else {
            console.debug('No directory selected');
        }
    } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        console.error('Error selecting directory:', errorMessage);
    }
};

const showAddModal = ref(false)
const availableDirs = computed(() =>
    paths.value.map(pathObj => pathObj.path)
);
const selectedDir = ref(availableDirs.value[0] || '')

const isOpening = ref(false)

const search = ref('')
import { computed } from 'vue'

const filteredPaths = computed(() => {
    if (!search.value.trim()) return paths.value
    const term = search.value.trim().toLowerCase()
    return paths.value
        .map(pathObj => {
            const filteredAddons = pathObj.addons.filter(addon =>
                addon.name.toLowerCase().includes(term) ||
                addon.notes.toLowerCase().includes(term)
            )
            return filteredAddons.length
                ? { ...pathObj, addons: filteredAddons }
                : null
        })
        .filter(Boolean)
})

const FOLDER_REVEAL_TIMEOUT_IN_MS = 800;
function handleOpenPath(path: string) {
    if (isOpening.value) return
    isOpening.value = true
    revealItemInDir(path)
    useTimeoutFn(() => {
        isOpening.value = false
    }, FOLDER_REVEAL_TIMEOUT_IN_MS)
}

const handleClone = async () => {
    if (!isGitUrlValid.value) return;
    // Use trimmedGitUrl.value
    showAddModal.value = false;
};

const trimmedGitUrl = computed(() => gitUrl.value.trim());
</script>

<template>
    <!-- <MainLayout> -->
    <div class="flex flex-col h-full gap-4">

        <!-- top bar: navbar + controls row -->
        <div class="sticky top-0 z-10 bg-base-200 rounded-box mb-2 flex flex-col gap-0">
            <div class="navbar justify-center">
                <div class="navbar-center w-full flex justify-center">
                    <div class="tabs tabs-box text-lg">
                        <button class="tab tab-active px-8 py-2">addons</button>
                        <button class="tab px-8 py-2">about</button>
                        <button class="tab px-8 py-2">config</button>
                    </div>
                </div>
            </div>
            <div class="flex flex-wrap items-center gap-2 bg-base-200 pb-2 pt-2 px-2">
                <button class="btn btn-primary">Update All</button>
                <!-- <button class="btn btn-primary"> <ArrowDownToLine /> </button> -->
                <!-- <button class="btn btn-primary"> <ArrowDownToLine /> Update All </button> -->

                <button class="btn btn-secondary">Refresh</button>
                <!-- <button class="btn btn-secondary"> <RefreshCcw /> Refresh </button> -->
                <!-- <button class="btn btn-secondary"> <RefreshCcw /> </button> -->

                <input v-model="search" class="input input-bordered flex-1 max-w-xs ml-auto"
                    placeholder="Search installed addons..." type="search" />
                <button class="btn btn-accent ml-2" @click="showAddModal = true">
                    <Plus />
                    Add addon
                </button>
            </div>
        </div>

        <!-- Add Addon Modal -->
        <dialog :open="showAddModal" class="modal">
            <div class="modal-box">
                <h3 class="font-bold text-lg mb-4">Clone Repository</h3>
                <div class="form-control mb-2">
                    <label class="label">
                        <span class="label-text">Git URL</span>
                    </label>
                    <input v-model="gitUrl" class="input input-bordered w-full"
                        placeholder="https://github.com/user/repo.git" />
                    <div :class="{ 'visible': isGitUrlValid === false && gitUrl, 'invisible': !gitUrl || isGitUrlValid !== false }"
                        class="text-error text-xs mt-1">
                        Please enter a valid HTTPS Git URL ending with <code>.git</code>
                    </div>
                </div>
                <div class="form-control mb-4">
                    <label class="label">
                        <span class="label-text">Install Directory</span>
                    </label>
                    <select v-model="selectedDir" class="select select-bordered w-full">
                        <option value="" disabled>Select directory</option>
                        <option v-for="dir in availableDirs" :key="dir" :value="dir">{{ dir }}</option>
                    </select>
                </div>
                <div class="modal-action">
                    <button class="btn btn-primary" @click="handleClone" :disabled="!isGitUrlValid">
                        Clone
                    </button>
                    <button class="btn" @click="showAddModal = false">Cancel</button>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop">
                <button @click="showAddModal = false">close</button>
            </form>
        </dialog>

        <!-- Paths and Addons list -->
        <div class="flex flex-col gap-4 overflow-y-auto p-4">
            <AddonCollapse v-for="(pathObj, idx) in filteredPaths" :key="idx" :path="pathObj.path"
                :isOpening="isOpening" @open-folder="handleOpenPath">
                <div class="flex flex-col gap-1.5 mt-2">
                    <div v-for="(addon, idx) in pathObj.addons" :key="idx"
                        class="card card-bordered bg-base-100 flex-row items-center p-2">
                        <div class="flex flex-1 flex-col gap-1 p-2">
                            <span class="font-semibold">{{ addon.name }}</span>
                            <span class="text-xs text-base-content/60">{{ addon.notes }}</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <div class="w-40">
                                <select class="select select-bordered select-sm w-full truncate" v-model="addon.branch">
                                    <option v-for="branch in addon.branches" :key="branch" :value="branch">
                                        {{ branch }}
                                    </option>
                                </select>
                            </div>
                            <button v-if="addon.isUpdateAvailable" class="btn btn-sm btn-primary"
                                @click="console.log('Update clicked', addon)">Update</button>
                            <button v-else class="btn btn-sm btn-ghost btn-disabled">Update</button>
                            <!-- <button class="btn btn-sm btn-primary" @click="console.log('Download clicked', addon)"> <CircleArrowDown /> </button> -->
                            <!-- <button class="btn btn-sm btn-ghost btn-disabled"> <CircleArrowDown /> </button> -->
                            <div class="dropdown dropdown-end">
                                <button tabindex="0" class="btn btn-sm btn-ghost">
                                    <Ellipsis />
                                </button>
                                <ul tabindex="0" class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-44">
                                    <li>
                                        <button class="flex items-center gap-2"
                                            @click="console.log('Readme clicked', addon)">
                                            <FileText class="w-4 h-4" />
                                            Readme
                                        </button>
                                    </li>
                                    <li>
                                        <button class="flex items-center gap-2"
                                            @click="console.log('Website clicked', addon)">
                                            <Globe class="w-4 h-4" />
                                            Website
                                        </button>
                                    </li>
                                    <li>
                                        <button class="flex items-center gap-2"
                                            @click="console.log('Repair clicked', addon)">
                                            <Wrench class="w-4 h-4" />
                                            Repair
                                        </button>
                                    </li>
                                    <li>
                                        <button class="flex items-center gap-2 text-error"
                                            @click="console.log('Delete clicked', addon)">
                                            <Trash2 class="w-4 h-4" />
                                            Delete
                                        </button>
                                    </li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </AddonCollapse>
            <!-- Add addon directory entry -->
            <button class="btn btn-outline btn-accent mt-2 self-start" @click="addAddonDirectory">
                <!-- <Plus class="mr-2" /> -->
                Add addon directory
            </button>
        </div>
    </div>
</template>

<style scoped></style>
