import { openUrl } from '@tauri-apps/plugin-opener'

export function useExternalLink() {
    const openWebsite = (gitUrl: string) => {
        const url = gitUrl.replace(/\.git$/, '')
        console.debug('Open website', url)
        openUrl(url)
    }

    return { openWebsite }
}
