<script setup lang="ts">
defineProps<{
    open: boolean
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
            <h3 class="font-bold text-lg mb-4">Delete Folder</h3>
            <p>
                Are you sure you want to stop managing
                <span class="font-mono">{{ folderPath }}</span
                >?<br />
                <strong>
                    This will not delete the AddOns folder, any installed
                    addons, or remove the
                    <code>.addonmanager</code> folder. </strong
                ><br />
                The folder will simply be removed from the list of managed
                directories.
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
