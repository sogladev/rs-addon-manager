import { describe, it, expect } from 'vitest'
import { parseImportLine } from '@/utils/importParser'

describe('parseImportLine', () => {
    it('parses a basic import line', () => {
        const line =
            '/path/to/addon AddonName *https://github.com/user/repo.git main'
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath: '/path/to/addon',
            addonName: 'AddonName',
            gitUrl: 'https://github.com/user/repo.git',
            branch: 'main',
        })
    })

    it('parses new format without path', () => {
        const line = 'AddonName *https://github.com/user/repo.git main'
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath: undefined,
            addonName: 'AddonName',
            gitUrl: 'https://github.com/user/repo.git',
            branch: 'main',
        })
    })

    it('parses import line with spaces in path', () => {
        const line =
            '/home/pc/Games/ascension-wow/drive_c/Program Files/Ascension Launcher/resources/epoch_live/Interface/Addons GroupBulletinBoard *https://github.com/TheNielDeal/GroupBulletinBoard main'
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath:
                '/home/pc/Games/ascension-wow/drive_c/Program Files/Ascension Launcher/resources/epoch_live/Interface/Addons',
            addonName: 'GroupBulletinBoard',
            gitUrl: 'https://github.com/TheNielDeal/GroupBulletinBoard',
            branch: 'main',
        })
    })

    it('handles different branch names', () => {
        const line =
            '/simple/path AddonName *https://github.com/user/repo.git develop'
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath: '/simple/path',
            addonName: 'AddonName',
            gitUrl: 'https://github.com/user/repo.git',
            branch: 'develop',
        })
    })

    it('trims whitespace from all components', () => {
        const line =
            '  /path/with/spaces   AddonName   *  https://github.com/user/repo.git   main  '
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath: '/path/with/spaces',
            addonName: 'AddonName',
            gitUrl: 'https://github.com/user/repo.git',
            branch: 'main',
        })
    })

    it('throws error when no git URL found (missing *)', () => {
        const line =
            '/path/to/addon AddonName https://github.com/user/repo.git main'

        expect(() => parseImportLine(line)).toThrow(
            'No git URL found (missing *)'
        )
    })

    it('throws error when git URL is empty', () => {
        const line = '/path/to/addon AddonName *'

        expect(() => parseImportLine(line)).toThrow('Git URL is empty')
    })

    it('throws error when branch is empty', () => {
        const line =
            '/path/to/addon AddonName *https://github.com/user/repo.git'

        expect(() => parseImportLine(line)).toThrow('Branch is empty')
    })

    it('throws error when format is invalid (no addon name)', () => {
        const line = '*https://github.com/user/repo.git main'

        expect(() => parseImportLine(line)).toThrow('Addon name is empty')
    })

    it('handles complex Windows paths with spaces', () => {
        const line =
            'C:\\Program Files\\World of Warcraft\\Interface\\AddOns MyAddon *https://github.com/user/myaddon.git master'
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath:
                'C:\\Program Files\\World of Warcraft\\Interface\\AddOns',
            addonName: 'MyAddon',
            gitUrl: 'https://github.com/user/myaddon.git',
            branch: 'master',
        })
    })

    it('handles URLs with special characters', () => {
        const line =
            '/path MyAddon *https://github.com/user/my-repo_name.git feature/branch-name'
        const result = parseImportLine(line)

        expect(result).toEqual({
            folderPath: '/path',
            addonName: 'MyAddon',
            gitUrl: 'https://github.com/user/my-repo_name.git',
            branch: 'feature/branch-name',
        })
    })
})
