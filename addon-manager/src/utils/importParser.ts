/**
 * Parses an import line in the format: <path> <addonName> *<gitUrl> <branch>
 * The addonName is ignored as it can be extracted from the path later.
 * Returns the base path (without addon name), gitUrl, and branch.
 * Handles paths with spaces by finding the first '*' character as the delimiter.
 */
export function parseImportLine(line: string) {
    const starIdx = line.indexOf('*')
    if (starIdx === -1) {
        throw new Error('No git URL found (missing *)')
    }

    const beforeStar = line.slice(0, starIdx).trim()
    const rest = line.slice(starIdx + 1).trim()
    const [gitUrl, branch] = rest.split(/\s+/, 2)

    if (!gitUrl) {
        throw new Error('Git URL is empty')
    }
    if (!branch) {
        throw new Error('Branch is empty')
    }

    // The format can be either:
    // 1) Legacy (with path): <path> <addonName> *<gitUrl> <branch>
    // 2) New (no path): <addonName> *<gitUrl> <branch>
    // We'll parse `beforeStar` and try to split into optional folderPath and addonName.

    // If there's a space in beforeStar we assume legacy format (path + addonName)
    const lastSpaceIdx = beforeStar.lastIndexOf(' ')
    let folderPath: string | undefined = undefined
    let addonName: string

    if (lastSpaceIdx === -1) {
        // No space => only addonName provided (new format)
        addonName = beforeStar
        if (!addonName) {
            throw new Error('Addon name is empty')
        }
    } else {
        // Legacy: split into folderPath and addonName
        folderPath = beforeStar.slice(0, lastSpaceIdx).trim()
        addonName = beforeStar.slice(lastSpaceIdx + 1).trim()

        if (!folderPath) {
            throw new Error('Folder path is empty')
        }
        if (!addonName) {
            throw new Error('Addon name is empty')
        }
    }

    return { folderPath, addonName, gitUrl, branch }
}
