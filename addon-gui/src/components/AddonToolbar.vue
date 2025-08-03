<script setup lang="ts">
import { Plus, Menu } from 'lucide-vue-next'
import TimeoutButton from '@/components/TimeoutButton.vue'
import { ref } from 'vue'

defineProps<{
    search: string
    hasUpdates: boolean
    outOfDateCount: number
}>()

const emit = defineEmits<{
    'update:search': [value: string]
    'update-all': []
    refresh: []
    'add-addon': []
}>()

const showImportExport = ref(false)
</script>

<template>
    <div class="sticky top-0 z-10 bg-base-200 flex flex-col gap-0">
        <!--     <div class="tabs tabs-box tabs-sm w-full justify-center gap-4">
  <button class="tab" :class="{ 'tab-active': activeTab === 'addons' }" @click="activeTab = 'addons'">Addons</button>
  <button class="tab" :class="{ 'tab-active': activeTab === 'about' }" @click="activeTab = 'about'">About</button>
  <button class="tab" :class="{ 'tab-active': activeTab === 'config' }" @click="activeTab = 'config'">Config</button>
< /div>-->

        <div
            class="flex flex-wrap items-center gap-2 bg-base-200 pb-2 pt-2 px-2"
        >
            <TimeoutButton
                :timeout="2000"
                class="btn btn-secondary"
                @click="emit('refresh')"
            >
                Check for Updates
            </TimeoutButton>
            <TimeoutButton
                :timeout="5000"
                class="btn btn-primary w-40"
                :disabled="!hasUpdates"
                @click="emit('update-all')"
            >
                <span v-if="hasUpdates">Update All</span>
                <span v-else>Up-to-date</span>
            </TimeoutButton>
            <input
                :value="search"
                @input="
                    emit(
                        'update:search',
                        ($event.target as HTMLInputElement).value
                    )
                "
                class="input input-bordered flex-1 max-w-xs ml-auto"
                placeholder="Search installed"
                type="search"
            />
            <button class="btn btn-accent w-40" @click="emit('add-addon')">
                <!-- <Plus /> -->
                Install addon
            </button>
            <div class="dropdown dropdown-end">
                <button tabindex="0" class="btn btn-ghost btn-square">
                    <Menu />
                </button>
                <ul
                    tabindex="0"
                    class="dropdown-content menu shadow bg-base-100 rounded-box w-52"
                >
                    <li>
                        <button @click="showImportExport = true">
                            Import/Export
                        </button>
                    </li>
                    <li>
                        <a
                            href="https://github.com/sogladev/rs-game-launcher"
                            target="_blank"
                        >
                            About
                        </a>
                    </li>
                </ul>
            </div>
        </div>
        <span v-if="outOfDateCount > 0" class="badge badge-warning">
            {{ outOfDateCount }} addon{{ outOfDateCount > 1 ? 's' : '' }} need
            update
        </span>
    </div>
</template>
