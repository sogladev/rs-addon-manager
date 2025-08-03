<script setup lang="ts">
import type { AddonRepository } from '@bindings/AddonRepository'
import type { Addon } from '@bindings/Addon'
import { Ellipsis } from 'lucide-vue-next'
import { FileText, Globe, Wrench, Trash2 } from 'lucide-vue-next'
import { ref, onMounted, computed } from 'vue'

const props = defineProps<{
    repo: AddonRepository & { latestRef?: string | null }
}>()
const emit = defineEmits<{
    'toggle-addon': [addon: Addon]
    'branch-change': [branch: string]
    update: []
    readme: []
    website: []
    repair: []
    delete: []
}>()

function handleBranchChange(e: Event) {
    const target = e.target as HTMLSelectElement | null
    if (!target) return
    const newBranch = target.value
    emit('branch-change', newBranch)
}

function handleToggleAddon(addon: Addon) {
    emit('toggle-addon', addon)
}

// mock update button progress background
const isUpdating = ref(true)
const updateProgress = ref(0)
onMounted(() => {
    const interval = setInterval(() => {
        if (updateProgress.value < 100) {
            updateProgress.value += 1
        } else {
            clearInterval(interval)
            isUpdating.value = false
        }
    }, 40)
})

const updateAvailable = computed(() => {
    return props.repo.latestRef && props.repo.repoRef !== props.repo.latestRef
})
</script>

<template>
    <div class="card card-bordered bg-base-100 flex-row items-center p-2">
        <div class="flex flex-1 flex-col gap-1 p-2">
            <span class="font-semibold">{{ repo.repoName }}</span>
            <span class="text-xs text-base-content/60">{{ repo.owner }}</span>
            <span v-if="repo.repoRef" class="text-xs text-base-content/50">
                Installed: {{ repo.repoRef }}
                <span v-if="updateAvailable" class="badge badge-warning ml-2"
                    >Update available</span
                >
            </span>
            <div v-if="repo.addons && repo.addons.length" class="mt-1">
                <ul class="ml-2 flex flex-col gap-1">
                    <li
                        v-for="addon in repo.addons"
                        :key="addon.name"
                        class="flex items-center gap-2"
                    >
                        <input
                            type="checkbox"
                            class="checkbox checkbox-sm"
                            v-model="addon.enabled"
                            @change="handleToggleAddon(addon)"
                        />
                        <span class="font-mono text-xs">{{ addon.name }}</span>
                        <span
                            v-if="!addon.enabled"
                            class="badge badge-xs badge-error"
                        >
                            disabled
                        </span>
                    </li>
                </ul>
            </div>
        </div>
        <div class="flex items-center gap-2">
            <div class="w-40">
                <select
                    class="select select-bordered select-sm w-full truncate"
                    :value="repo.currentBranch"
                    @change="handleBranchChange"
                >
                    <option
                        v-for="branch in repo.availableBranches"
                        :key="branch"
                        :value="branch"
                    >
                        {{ branch }}
                    </option>
                </select>
            </div>
            <button
                :class="[
                    'btn btn-sm relative overflow-hidden w-20',
                    updateAvailable ? 'btn-warning' : 'btn-primary',
                ]"
                @click="emit('update')"
                :disabled="isUpdating || !updateAvailable"
            >
                <span class="relative z-10">{{
                    isUpdating
                        ? 'Updating...'
                        : updateAvailable
                          ? 'Update'
                          : 'Up to date'
                }}</span>
                <div
                    v-if="isUpdating"
                    class="absolute left-0 top-0 h-full bg-primary/30 transition-all"
                    :style="{ width: updateProgress + '%' }"
                ></div>
            </button>
            <div class="dropdown dropdown-end">
                <button tabindex="0" class="btn btn-sm btn-ghost">
                    <Ellipsis />
                </button>
                <ul
                    tabindex="0"
                    class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-44"
                >
                    <li>
                        <button
                            class="flex items-center gap-2"
                            @click="emit('readme')"
                        >
                            <FileText class="w-4 h-4" />
                            Readme
                        </button>
                    </li>
                    <li>
                        <button
                            class="flex items-center gap-2"
                            @click="emit('website')"
                        >
                            <Globe class="w-4 h-4" />
                            Website
                        </button>
                    </li>
                    <li>
                        <button
                            class="flex items-center gap-2"
                            @click="emit('repair')"
                        >
                            <Wrench class="w-4 h-4" />
                            Repair
                        </button>
                    </li>
                    <li>
                        <button
                            class="flex items-center gap-2 text-error"
                            @click="emit('delete')"
                        >
                            <Trash2 class="w-4 h-4" />
                            Delete
                        </button>
                    </li>
                </ul>
            </div>
        </div>
    </div>
</template>

<style scoped></style>
