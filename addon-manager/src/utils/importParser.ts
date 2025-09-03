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

    // Split the path part to separate base path from addon name
    // Find the last space that separates the addon name from the path
    const lastSpaceIdx = beforeStar.lastIndexOf(' ')
    if (lastSpaceIdx === -1) {
        throw new Error(
            'Invalid format: expected <path> <addonName> *<gitUrl> <branch>'
        )
    }

    const folderPath = beforeStar.slice(0, lastSpaceIdx).trim()
    // const addonName = beforeStar.slice(lastSpaceIdx + 1).trim() // not used

    if (!folderPath) {
        throw new Error('Folder path is empty')
    }
    if (!gitUrl) {
        throw new Error('Git URL is empty')
    }
    if (!branch) {
        throw new Error('Branch is empty')
    }

    return { folderPath, gitUrl, branch }
}
