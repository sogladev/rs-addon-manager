import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

interface Issue {
    timestamp: string
    message: string
    details?: unknown
}

const globalErrorMessage = ref<string | null>(null)
const issues = ref<Issue[]>([])

export function useGlobalError() {
    const setError = (error: unknown) => {
        console.error('Unexpected error:', error)
        const errorMessage =
            typeof error === 'string'
                ? error
                : error instanceof Error
                  ? error.message
                  : JSON.stringify(error)
        globalErrorMessage.value = errorMessage
        addIssue(errorMessage, error)
    }

    const clearError = () => {
        globalErrorMessage.value = null
    }

    const addIssue = (message: string, details?: unknown) => {
        const issue: Issue = {
            timestamp: new Date().toISOString(),
            message: message,
            details: details,
        }
        issues.value.push(issue)
    }

    const getIssueLog = () => {
        return issues.value
            .map(
                (issue) =>
                    `[${issue.timestamp}] ${issue.message} ${issue.details ? JSON.stringify(issue.details) : ''}`
            )
            .join('\n')
    }

    const saveIssueLog = async () => {
        try {
            const filePath = await open({
                multiple: false,
                directory: false,
                filters: [{ name: 'Text', extensions: ['txt'] }],
            })
            if (filePath) {
                await writeTextFile(filePath, getIssueLog())
                console.log('Issue log saved to:', filePath)
            }
        } catch (e) {
            console.error('Failed to save issue log:', e)
        }
    }

    return {
        globalErrorMessage,
        setError,
        clearError,
        issues,
        getIssueLog,
        saveIssueLog,
        addIssue,
    }
}
