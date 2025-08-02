<script setup lang="ts">
defineProps<{
    installStatus: {
        progress?: { current: number; total: number }
        step?: string
        error?: string
        warning?: string
        active: boolean
    }
}>()
</script>

<template>
    <div
        v-if="
            installStatus.active &&
            (installStatus.step ||
                installStatus.progress ||
                installStatus.warning ||
                installStatus.error)
        "
        class="w-full p-2 bg-base-300 rounded-box mb-2"
    >
        <div v-if="installStatus.step" class="text-base-content mb-1">
            {{ installStatus.step }}
        </div>
        <div v-if="installStatus.progress">
            <progress
                class="progress progress-primary w-full"
                :value="installStatus.progress.current"
                :max="installStatus.progress.total"
            ></progress>
            <span>
                {{ installStatus.progress.current }} /
                {{ installStatus.progress.total }}
            </span>
        </div>
        <div v-if="installStatus.warning" class="alert alert-warning mt-2">
            {{ installStatus.warning }}
        </div>
        <div v-if="installStatus.error" class="alert alert-error mt-2">
            {{ installStatus.error }}
        </div>
    </div>
</template>
