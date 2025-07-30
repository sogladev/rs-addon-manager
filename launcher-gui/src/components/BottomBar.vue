<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { load } from '@tauri-apps/plugin-store'

import { computed, ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
const { t } = useI18n() // t() function to access translations

import type { TransactionReport, Progress } from '@/types/rs'
import { etaToHumanReadable, formatBytes } from '@/utils/format'

import Toast from '@/components/ToastMessages.vue'
const TOAST_ERROR_TIMEOUT_IN_MILLISECONDS = 5000
const toastRef = ref<InstanceType<typeof Toast> | null>(null)

enum AppState {
    Verifying = 'verifying',
    Nogame = 'nogame',
    CreatingTransaction = 'creating',
    AwaitingApproval = 'awaiting',
    ReadyToRetry = 'retry',
    Downloading = 'downloading',
    Complete = 'complete',
}
const appState = ref<AppState>(AppState.Verifying)

const selectedDirectory = ref<string>('.')
console.debug('selectedDirectory value:', selectedDirectory.value)
// True if the selected directory contains the base game files
const isValidDirectory = ref(false)

const transactionReport = ref<TransactionReport | null>(null)
const progressValue = ref(0)
const progressText = ref<string>('')

const primaryMessage = computed(() => t(`primary.${appState.value}`))
const secondaryMessage = computed(() => t(`secondary.${appState.value}`))
const buttonLabel = computed(() => t(`buttons.${appState.value}`))
const buttonDisabled = computed(() =>
    [AppState.Verifying, AppState.Downloading].includes(appState.value)
)

async function handleButtonClick() {
    switch (appState.value) {
        case AppState.Nogame:
            console.debug('Click Selecting game directory...')
            selectGameDirectory()
            break
        case AppState.AwaitingApproval:
            console.debug('Click Processing approval...')
            appState.value = AppState.Downloading
            initiateTransaction()
            break
        case AppState.ReadyToRetry:
            console.debug('Click Retrying transaction...')
            appState.value = AppState.CreatingTransaction
            createTransaction()
            break
        case AppState.Complete:
            console.debug('Click Launching game...')
            launchGame()
            break
        default:
            break
    }
}

async function verifyGameIntegrity(gameDirectory: string) {
    try {
        const isValid = await invoke('verify_game_integrity', {
            basePath: gameDirectory,
        })
        if (isValid) {
            console.debug('Game directory is valid')
            isValidDirectory.value = true
            appState.value = AppState.CreatingTransaction

            // Save the game directory in the store
            try {
                const store = await load('store.json')
                await store.set('game-directory', { value: gameDirectory })
                await store.save()
            } catch (error) {
                console.error('Failed to save to store:', error)
            }

            createTransaction()
        } else {
            console.debug('Game directory is invalid')
            isValidDirectory.value = false
            appState.value = AppState.Nogame
        }
    } catch (error: unknown) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error verifying game integrity:', error)
        toastRef.value?.showToast(
            'error',
            t('toasts.verify', { error: errorMessage }),
            TOAST_ERROR_TIMEOUT_IN_MILLISECONDS
        )
        isValidDirectory.value = false
    }
}

async function selectGameDirectory() {
    try {
        const directory = await open({
            multiple: false,
            directory: true,
        })
        if (directory) {
            selectedDirectory.value = directory
            console.debug(`Selected directory: ${directory}`)
            appState.value = AppState.Verifying
            verifyGameIntegrity(directory)
        } else {
            console.debug('No directory selected')
        }
    } catch (error: unknown) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error selecting directory:', error)
        toastRef.value?.showToast(
            'error',
            t('toasts.directory', { error: errorMessage }),
            TOAST_ERROR_TIMEOUT_IN_MILLISECONDS
        )
    }
}

async function createTransaction() {
    try {
        const report = await invoke<TransactionReport>('create_transaction', {
            basePath: selectedDirectory.value,
        })
        console.log('Transaction report:', report)
        transactionReport.value = report
        const isUpToDate =
            report.missing_files.length === 0 &&
            report.outdated_files.length === 0
        if (isUpToDate) {
            appState.value = AppState.Complete
            return
        }
        appState.value = AppState.AwaitingApproval
    } catch (error: unknown) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error creating transaction:', error)
        toastRef.value?.showToast(
            'error',
            t('toasts.creating', { error: errorMessage }),
            TOAST_ERROR_TIMEOUT_IN_MILLISECONDS
        )

        appState.value = AppState.ReadyToRetry
    }
}

// Transaction Overview Modal
const modalRef = ref(<InstanceType<typeof HTMLDialogElement> | null>null)
watch(appState, (newState) => {
    if (newState === AppState.AwaitingApproval) {
        modalRef.value?.showModal()
    } else {
        modalRef.value?.close() // Close the modal when the state changes
    }
})

async function initiateTransaction() {
    try {
        console.debug('Downloading start')
        await invoke('download')
        console.debug('Downloading end')
        appState.value = AppState.Complete
    } catch (error: unknown) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error downloading: ', error)
        toastRef.value?.showToast(
            'error',
            t('toasts.downloading', { error: errorMessage }),
            TOAST_ERROR_TIMEOUT_IN_MILLISECONDS
        )
        appState.value = AppState.ReadyToRetry
    }
}

const downloadProgressData = ref<Progress | null>(null)

watch(appState, () => {
    if (appState.value === AppState.Downloading) return
    progressValue.value = appState.value === AppState.Complete ? 100 : 0
})

const showSpinner = ref(true)
watch(appState, (newState) => {
    showSpinner.value = [
        AppState.Verifying,
        AppState.CreatingTransaction,
    ].includes(newState)
})

watch(downloadProgressData, (progress) => {
    if (!progress) return
    const fileProgress = (progress.current / progress.file_size) * 100
    const overallProgress =
        ((progress.file_index - 1) * 100 + fileProgress) / progress.total_files
    progressValue.value = Math.round(overallProgress)
})

onMounted(async () => {
    try {
        const store = await load('store.json')
        const storeValue = await store.get<{ value: string }>('game-directory')
        selectedDirectory.value = storeValue?.value || '.'
        console.debug('Store value loaded:', selectedDirectory.value)
    } catch (error) {
        console.error('Failed to load from store:', error)
    }

    await listen<Progress>('download-progress', (event) => {
        downloadProgressData.value = event.payload as Progress
        const progress = downloadProgressData.value
        if (!progress) return
        const filesWidth = String(progress.total_files).length
        const fileIndexFmt = String(progress.file_index).padStart(filesWidth)
        const remainingFmt = formatBytes(progress.total_amount_left).padStart(8)
        const speedFmt = formatBytes(progress.speed).padStart(8) + '/s'
        const timeFmt = etaToHumanReadable(progress.expected_time_left)

        progressText.value =
            `[${fileIndexFmt}/${progress.total_files}]` +
            ` ${progress.filename}` +
            ` ${speedFmt}` +
            ` Remaining: ${remainingFmt}` +
            ` ETA: ${timeFmt}`
    })
    verifyGameIntegrity(selectedDirectory.value)
})

async function launchGame() {
    try {
        await invoke('launch_game', {
            basePath: selectedDirectory.value,
        })
    } catch (error: unknown) {
        const errorMessage =
            error instanceof Error ? error.message : String(error)
        console.error('Error launching game:', error)
        toastRef.value?.showToast(
            'error',
            t('toasts.launch', { error: errorMessage }),
            TOAST_ERROR_TIMEOUT_IN_MILLISECONDS
        )
    }
}
</script>

<template>
    <Toast ref="toastRef" />
    <dialog ref="modalRef" id="transaction_modal" class="modal">
        <div class="modal-box">
            <h3 class="text-lg font-bold">Transaction Overview</h3>
            <div v-if="!transactionReport">
                <p>No transaction report available</p>
            </div>
            <div v-else>
                <h4 class="text-md font-bold mt-4">Base path</h4>
                <p>{{ transactionReport?.base_path }}</p>

                <h4
                    v-if="transactionReport.up_to_date_files.length"
                    class="text-md font-bold mt-4"
                >
                    Up-to-date files
                </h4>
                <ul v-if="transactionReport.up_to_date_files.length">
                    <li
                        v-for="file in transactionReport.up_to_date_files"
                        :key="file.path"
                    >
                        {{ file.path }}
                        <span v-if="file.current_size" class="opacity-50">
                            (Size: {{ formatBytes(file.current_size) }})
                        </span>
                    </li>
                </ul>

                <h4
                    v-if="transactionReport.outdated_files.length"
                    class="text-md font-bold mt-4"
                >
                    Outdated files (will be updated)
                </h4>
                <ul v-if="transactionReport.outdated_files.length">
                    <li
                        v-for="file in transactionReport.outdated_files"
                        :key="file.path"
                    >
                        {{ file.path }}
                        <span v-if="file.current_size" class="opacity-50">
                            (Current Size: {{ formatBytes(file.current_size) }},
                            New Size: {{ formatBytes(file.new_size) }})
                        </span>
                    </li>
                </ul>

                <h4
                    v-if="transactionReport.missing_files.length"
                    class="text-md font-bold mt-4"
                >
                    Missing files (will be downloaded)
                </h4>
                <ul v-if="transactionReport.missing_files.length">
                    <li
                        v-for="file in transactionReport.missing_files"
                        :key="file.path"
                    >
                        {{ file.path }}
                        <span class="opacity-50">
                            (New Size: {{ formatBytes(file.new_size) }})
                        </span>
                    </li>
                </ul>

                <h4
                    v-if="transactionReport.removed_files.length"
                    class="text-md font-bold mt-4"
                >
                    Files to be removed:
                </h4>
                <ul v-if="transactionReport.removed_files.length">
                    <li
                        v-for="file in transactionReport.removed_files"
                        :key="file.path"
                    >
                        {{ file.path }}
                        <span class="opacity-50">
                            (Current Size:
                            {{ formatBytes(file.current_size!) }})
                        </span>
                    </li>
                </ul>

                <h4 class="text-md font-bold mt-4">Transaction Summary</h4>
                Installing/Updating:
                {{
                    transactionReport.missing_files.length +
                    transactionReport.outdated_files.length
                }}
                files
                <br />
                Removing: {{ transactionReport.removed_files.length }} files
                <br />
                Total size of inbound files is
                {{ formatBytes(transactionReport.total_download_size) }}. Need
                to download
                {{ formatBytes(transactionReport.total_download_size) }}.
                <span v-if="transactionReport.disk_space_change > 0">
                    After this operation,
                    {{ formatBytes(transactionReport.disk_space_change) }} of
                    additional disk space will be used.
                </span>
                <span v-else>
                    After this operation,
                    {{ formatBytes(transactionReport.disk_space_change) }} of
                    additional disk space will be freed.
                </span>
            </div>
            <div class="mt-2 flex flex-row">
                <button
                    class="btn btn-primary flex-grow items-center justify-center text-center text-neutral-content"
                    @click="handleButtonClick"
                >
                    {{ buttonLabel }}
                </button>
                <button
                    v-if="transactionReport"
                    class="ml-2 btn btn-md btn-secondary"
                    @click="() => modalRef?.close()"
                >
                    Deny
                </button>
            </div>
        </div>
    </dialog>
    <button
        v-if="appState == AppState.AwaitingApproval"
        class="btn"
        onclick="transaction_modal.showModal()"
    >
        Open Transaction
    </button>
    <!-- For debugging -->
    <!-- <button class="btn" onclick="transaction_modal.showModal()">Open Transaction</button> -->

    <div class="p-4 bg-base-200/75">
        <div class="flex items-end">
            <div class="flex flex-col flex-grow">
                <p class="text-lg">
                    {{ primaryMessage }}
                    <span
                        v-if="showSpinner"
                        class="loading loading-spinner text-primary"
                    ></span>
                </p>
                <div class="flex justify-between">
                    <p class="text-sm font-mono">{{ secondaryMessage }}</p>
                    <p class="text-sm font-mono">{{ progressText }}</p>
                </div>
                <progress
                    class="progress progress-primary"
                    :value="progressValue"
                    max="100"
                ></progress>
            </div>
            <div class="ml-2 flex flex-col items-end">
                <button
                    class="btn btn-primary w-full items-center justify-center text-center"
                    :disabled="buttonDisabled"
                    @click="handleButtonClick"
                >
                    {{ buttonLabel }}
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped></style>
