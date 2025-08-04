<script setup lang="ts">
import TimeoutButton from '@/components/TimeoutButton.vue'
import {
    ChevronRight,
    FolderOpen,
    MessageSquareWarning,
    Trash2,
} from 'lucide-vue-next'
import { ref } from 'vue'

const { path, isValid } = defineProps({
    path: { type: String, required: true },
    isValid: { type: Boolean, default: true },
})
const emit = defineEmits(['open-folder', 'delete-folder'])

const open = ref(true)

const toggle = () => {
    open.value = !open.value
}
const onOpenFolder = () => {
    emit('open-folder', path)
}
</script>

<template>
    <div
        :class="[
            'rounded-box border border-base-300 bg-base-100',
            'transition-all',
            { shadow: open },
        ]"
    >
        <!-- border-l-4 border-primary/30 -->
        <div
            class="flex items-center px-4 py-2 gap-4 font-mono bg-base-200/80"
            tabindex="0"
        >
            <span class="flex items-center gap-1 flex-1">
                <span
                    v-if="isValid === false"
                    class="text-warning cursor-pointer tooltip tooltip-warning tooltip-right"
                    data-tip="This is not a valid AddOns directory. Please select the AddOns folder found under Interface/AddOns in your WoW directory."
                >
                    <MessageSquareWarning />
                </span>
                {{ path }}
            </span>
            <TimeoutButton class="btn btn-ghost btn-sm" @click="onOpenFolder">
                <FolderOpen class="w-5 h-5" />
            </TimeoutButton>

            <button
                class="btn btn-ghost btn-sm"
                @click.stop="() => emit('delete-folder', path)"
            >
                <Trash2 class="w-5 h-5" />
            </button>

            <button
                class="btn btn-ghost btn-sm p-0 min-w-0"
                @click="toggle"
                :aria-expanded="open"
                :aria-label="open ? 'Collapse' : 'Expand'"
            >
                <ChevronRight
                    :class="[
                        'w-5 h-5 transition-transform duration-200',
                        open ? 'rotate-90' : '',
                    ]"
                />
            </button>
        </div>
        <div v-show="open" class="px-4 pb-4">
            <slot></slot>
        </div>
    </div>
</template>
