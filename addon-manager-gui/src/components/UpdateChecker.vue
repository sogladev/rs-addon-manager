<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'

const update = ref<Update | null>(null)

const checking = ref(false)
const progress = ref(0)
const updateError = ref<string | null>(null)
const currentVersion = ref<string>('')

onMounted(async () => {
    try {
        currentVersion.value = await getVersion()
    } catch (e) {
        console.error('Failed to get app version:', e)
        currentVersion.value = 'Unknown'
    }
})

async function handleCheck() {
    checking.value = true
    try {
        const result = await check()
        update.value = result || null
    } catch (e: any) {
        console.error('Update check failed', e)
        update.value = null
        updateError.value =
            e?.message ||
            e?.toString() ||
            'Unknown error while checking for updates'
    } finally {
        checking.value = false
    }
}

async function handleDownload() {
    if (!update.value) return
    let downloaded = 0
    let total = 0
    updateError.value = null
    try {
        await update.value.downloadAndInstall((event: any) => {
            switch (event.event) {
                case 'Started':
                    progress.value = 0
                    break
                case 'Progress':
                    downloaded = event?.data?.downloaded ?? downloaded
                    total =
                        event?.data?.total ??
                        event?.data?.contentLength ??
                        total
                    if (total > 0) {
                        progress.value = Math.round((downloaded / total) * 100)
                    } else {
                        progress.value = 0
                    }
                    break
                case 'Finished':
                    progress.value = 100
                    break
            }
        })

        await relaunch()
    } catch (e: any) {
        console.error('Update install failed:', e)
        updateError.value =
            e?.message ||
            e?.toString() ||
            'Unknown error while installing update'
    }
}
</script>

<template>
    <div class="flex flex-col gap-4">
        <div v-if="currentVersion" class="text-sm">
            <span class="font-semibold">Current version:</span>
            {{ currentVersion }}
        </div>

        <button
            class="btn btn-outline btn-sm"
            @click="handleCheck"
            :disabled="checking"
        >
            <span
                v-if="checking"
                class="loading loading-spinner loading-xs"
            ></span>
            <span v-else>Check for updates</span>
        </button>

        <div v-if="updateError" class="alert alert-error py-2">
            <span>{{ updateError }}</span>
        </div>

        <div v-if="update" class="flex flex-col gap-2">
            <div class="text-sm font-semibold">
                Version {{ update.version }}
                <small>({{ update.date || 'n/a' }})</small>
            </div>
            <div class="prose text-xs max-w-full" v-text="update.body || ''" />
            <button class="btn btn-primary btn-sm" @click="handleDownload">
                Download & Install
            </button>
            <progress
                v-if="progress > 0"
                class="progress progress-primary w-full"
                :value="progress"
                max="100"
            />
        </div>
    </div>
</template>
