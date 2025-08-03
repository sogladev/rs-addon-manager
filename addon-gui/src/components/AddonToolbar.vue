<script setup lang="ts">
import { Plus } from 'lucide-vue-next'

defineProps<{
    search: string
}>()

const emit = defineEmits<{
    'update:search': [value: string]
    'update-all': []
    refresh: []
    'add-addon': []
}>()
</script>

<template>
    <div
        class="sticky top-0 z-10 bg-base-200 rounded-box mb-2 flex flex-col gap-0"
    >
        <div class="navbar justify-center">
            <div class="navbar-center w-full flex justify-center">
                <div class="tabs tabs-box text-lg">
                    <button class="tab tab-active px-8 py-2">addons</button>
                    <button class="tab px-8 py-2">about</button>
                    <button class="tab px-8 py-2">config</button>
                </div>
            </div>
        </div>
        <div
            class="flex flex-wrap items-center gap-2 bg-base-200 pb-2 pt-2 px-2"
        >
            <button class="btn btn-primary" @click="emit('update-all')">
                Update All
            </button>
            <button class="btn btn-secondary" @click="emit('refresh')">
                Check for Updates
            </button>
            <input
                :value="search"
                @input="
                    emit(
                        'update:search',
                        ($event.target as HTMLInputElement).value
                    )
                "
                class="input input-bordered flex-1 max-w-xs ml-auto"
                placeholder="Search installed addons..."
                type="search"
            />
            <button class="btn btn-accent ml-2" @click="emit('add-addon')">
                <Plus />
                Add addon
            </button>
        </div>
    </div>
</template>
