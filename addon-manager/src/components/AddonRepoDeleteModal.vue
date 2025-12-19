<script setup lang="ts">
import type { AddonRepository } from '@bindings/AddonRepository'

defineProps<{
    open: boolean
    addon: AddonRepository | null
    folderPath: string | null
}>()

const emit = defineEmits<{
    confirm: []
    cancel: []
}>()
</script>

<template>
    <dialog :open="open" class="modal">
        <div class="modal-box">
            <h3 class="font-bold text-lg mb-4">Delete Addon</h3>
            <p>
                Are you sure you want to delete addon
                <span class="font-mono">{{
                    addon?.source.type === 'git'
                        ? addon.source.repo_name
                        : addon?.source.folder_name
                }}</span>
                from directory
                <span class="font-mono">{{ folderPath }}</span
                >?
            </p>
            <div class="modal-action">
                <button class="btn btn-error" @click="emit('confirm')">
                    Delete
                </button>
                <button class="btn" @click="emit('cancel')">Cancel</button>
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button @click="emit('cancel')">close</button>
        </form>
    </dialog>
</template>
