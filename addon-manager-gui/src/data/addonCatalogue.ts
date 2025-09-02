export interface CatalogueAddon {
    name: string
    gitUrl: string
    description: string
    author?: string
    category: AddonCategory
    notes?: string
    installable?: boolean // whether the addon can be auto-installed via git
    manualUrl?: string // URL for manual download if not installable
    serverCompat?: ServerCompatibility
}

export type ServerCompatibility = {
    projectEpoch?: boolean
}

export type AddonCategory =
    | 'ui-enhancement' // UI modifications and improvements
    | 'combat' // Combat, DPS meters, threat meters
    | 'questing' // Quest helpers and guides
    | 'utility' // General utility addons
    | 'social' // Guild, chat, social features
    | 'profession' // Crafting, gathering, auction house
    | 'pvp' // PvP focused addons
    | 'cosmetic' // Appearance, themes, visual enhancements
    | 'maps' // Maps, atlases, location tools
    | 'leveling' // Leveling guides and assistance
    | 'economy' // Economy, trading, auction house
    | 'bags' // Bag and inventory management

export const ADDON_CATALOGUE: CatalogueAddon[] = [
    // Project Epoch Specific AddOns
    {
        name: 'AtlasLoot Project Epoch',
        gitUrl: 'https://github.com/Raynbock/AtlaslootProjectEpoch.git',
        description:
            'UI mod allowing for loot tables of bosses to be browsed whenever needed within the game. Specifically built for Project Epoch.',
        author: 'Raynbock',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    {
        name: 'Questie Epoch',
        gitUrl: 'https://github.com/trav346/Questie-Epoch.git',
        description:
            'An all-in-one questing addon designed to support the new custom quests on Project Epoch. Behaves similarly to Classic Questie.',
        author: 'trav346',
        category: 'questing',
        serverCompat: { projectEpoch: true },
    },
    {
        name: 'CircadianRhythm',
        gitUrl: 'https://github.com/Bennylavaa/CircadianRhythm.git',
        description:
            'GUI window with the Day/Night cycle information specific to Project Epoch.',
        author: 'Bennylavaa',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    {
        name: 'ElvUI Epoch',
        gitUrl: 'https://github.com/Bennylavaa/ElvUI-Epoch.git',
        description:
            'A user interface designed around user-friendliness with extra features. Forked from ElvUI-WotLK to include an Epoch crash fix.',
        author: 'Bennylavaa',
        category: 'ui-enhancement',
        serverCompat: { projectEpoch: true },
    },
    {
        name: 'LFG (Dungeon Finder)',
        gitUrl: 'https://github.com/Bennylavaa/LFG.git',
        description:
            'Dungeon finder without teleport. Attempts to arrange groups automatically among other addon users.',
        author: 'Bennylavaa',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    {
        name: 'GroupBulletinBoard Epoch',
        gitUrl: 'https://github.com/TheNielDeal/GroupBulletinBoard.git',
        description:
            'Epoch version of GroupBulletinBoard for finding and creating groups.',
        author: 'TheNielDeal',
        category: 'social',
        serverCompat: { projectEpoch: true },
    },
    {
        name: 'GroupBulletinBoard Epoch',
        gitUrl: 'https://github.com/sogladev/GroupBulletinBoard-epoch.git',
        description:
            'Epoch version of GroupBulletinBoard for finding and creating groups.',
        author: 'sogladev',
        category: 'social',
        serverCompat: { projectEpoch: true },
    },

    // UI Enhancement
    {
        name: 'AdiBags',
        gitUrl: 'https://github.com/Sattva-108/AdiBags.git',
        description:
            'Displays the contents of your bags in single view, distributed into several sections using smart filters.',
        author: 'Sattva-108',
        category: 'ui-enhancement',
    },
    {
        name: 'Bagnon',
        gitUrl: 'https://github.com/RichSteini/Bagnon-3.3.5.git',
        description: 'Inventory manager with added autoclean functionality.',
        author: 'RichSteini',
        category: 'ui-enhancement',
    },
    {
        name: 'DiabolicUI',
        gitUrl: 'https://github.com/thExiled/DiabolicUI.git',
        description: 'Diablo 3 UI for WoW with a dark, immersive design.',
        author: 'thExiled',
        category: 'ui-enhancement',
    },
    {
        name: 'KkthnxUI',
        gitUrl: 'https://github.com/mrrosh/KkthnxUI_WotLK.git',
        description:
            'Simplistic user interface that holds onto information and functionality while keeping good looks.',
        author: 'mrrosh',
        category: 'ui-enhancement',
    },
    {
        name: 'Dominos',
        gitUrl: 'https://github.com/bkader/Dominos.git',
        description:
            'Replaces your main bar with movable parts that offer additional customization.',
        author: 'bkader',
        category: 'ui-enhancement',
    },
    {
        name: 'RetailUI',
        gitUrl: 'https://github.com/a3st/RetailUI.git',
        description: 'Brings the modern Retail WoW UI to Project Epoch.',
        author: 'a3st',
        category: 'ui-enhancement',
    },
    {
        name: 'TidyPlates',
        gitUrl: 'https://github.com/Ravendwyr/tidy-plates.git',
        description:
            'Enhanced and customizable nameplates for better target visibility.',
        author: 'Ravendwyr',
        category: 'ui-enhancement',
        notes: 'May need compatibility testing',
    },
    {
        name: 'NotPlater',
        gitUrl: 'https://github.com/RichSteini/NotPlater.git',
        description:
            'Plater imitation addon for 3.3.5. Customizes nameplate text, icons, and style.',
        author: 'RichSteini',
        category: 'ui-enhancement',
    },
    {
        name: 'pretty_lootalert',
        gitUrl: 'https://github.com/sogladev/pretty_lootalert',
        description:
            'Loot alert UI with configurable display for item pickup notifications.',
        author: 'sogladev',
        category: 'ui-enhancement',
        notes: 'Configuration via a "config.lua" file; this is a pre-configured fork of s0h2x/pretty_lootalert to show fewer toasts',
    },

    // Combat
    {
        name: 'Details',
        gitUrl: 'https://github.com/Bunny67/Details-WotLK.git',
        description:
            'Advanced damage meter with threat plugin and detailed combat statistics.',
        author: 'Bunny67',
        category: 'combat',
    },
    {
        name: 'Skada Revisited',
        gitUrl: 'https://github.com/bkader/Skada-WoTLK.git',
        description:
            'Damage meter with extra modules that replicate popular features of other combat meters.',
        author: 'bkader',
        category: 'combat',
    },
    {
        name: 'Omen3',
        gitUrl: 'https://github.com/Exposeya/Omen.git',
        description: 'Threat meter. Single purpose and easy set up.',
        author: 'Exposeya',
        category: 'combat',
        notes: 'May need compatibility verification',
    },

    // Utility
    {
        name: 'LeatrixPlus',
        gitUrl: 'https://github.com/Sattva-108/Leatrix_Plus.git',
        description: 'Quality-of-life improvements and system tweaks.',
        author: 'Sattva-108',
        category: 'utility',
    },
    {
        name: 'WeakAuras',
        gitUrl: 'https://github.com/NoM0Re/WeakAuras-WotLK.git',
        description:
            'Create visual alerts and condition tracking for abilities, buffs, etc.',
        author: 'NoM0Re',
        category: 'utility',
    },
    {
        name: 'Immersion',
        gitUrl: 'https://github.com/s0h2x/Immersion-WotLK.git',
        description:
            'Immersive questing addon that makes quest interactions more engaging.',
        author: 's0h2x',
        category: 'questing',
    },
    {
        name: 'ItemRack',
        gitUrl: 'https://github.com/FeanoroWoW/ItemRack-2.243.git',
        description:
            'Make swapping equipment easier through popout slot menus and gear sets.',
        author: 'FeanoroWoW',
        category: 'utility',
    },

    // Social
    {
        name: 'Postal',
        gitUrl: 'https://github.com/Bennylavaa/Postal.git',
        description:
            'Enhanced mail UI with mass operations and better organization.',
        author: 'Bennylavaa',
        category: 'social',
    },
    {
        name: 'Chat Emoji',
        gitUrl: 'https://github.com/Bennylavaa/ChatEmojis.git',
        description: 'Adds emoji support to chat for better communication.',
        author: 'Bennylavaa',
        category: 'social',
    },
    {
        name: 'TwitchEmotes-335',
        gitUrl: 'https://github.com/sogladev/TwitchEmotes-335',
        description:
            'Shows Twitch emotes in your chat by typing them as you would into Twitch chat',
        author: 'sogladev',
        category: 'social',
    },
    {
        name: 'Hermes',
        gitUrl: 'https://github.com/dobleedpurple/Hermes.git',
        description:
            'Enables cross-faction communication with minimal comprehension loss.',
        author: 'dobleedpurple',
        category: 'social',
    },

    // Unified Epoch Addons List from Discord
    // 1. Adi-bags
    {
        name: 'Adi-bags',
        gitUrl: 'https://github.com/Sattva-108/AdiBags',
        description: 'Bag mod',
        author: 'Sattva-108',
        category: 'ui-enhancement',
    },
    // 2. Atlas
    {
        name: 'Atlas',
        gitUrl: 'https://github.com/Raynbock/Atlas-Project-Epoch',
        description: 'Maps for dungeons (Epoch Addon)',
        author: 'Raynbock',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 3. AtlaslootProjectEpoch
    {
        name: 'Atlasloot',
        gitUrl: 'https://github.com/Raynbock/AtlaslootProjectEpoch',
        description: 'Loot for dungeons (Epoch Addon)',
        author: 'Raynbock',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 4. Atlas Quest
    {
        name: 'Atlas Quest',
        gitUrl: 'https://github.com/Rofos2011/AtlasQuest',
        description: 'AtlasQuest UI integration (Epoch Addon)',
        author: 'Rofos2011',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 5. Auctionlite
    {
        name: 'Auctionlite',
        gitUrl: 'https://felbite.com/addon/4303-auctionlite/',
        description: 'AH replacement',
        category: 'utility',
        installable: false,
        manualUrl: 'https://felbite.com/addon/4303-auctionlite/',
    },
    // 6. AUX
    {
        name: 'AUX',
        gitUrl: 'https://github.com/Bestoriop/Aux-addon-Epoch/tree/main',
        description: 'Auction house replacement tool (Epoch Addon)',
        author: 'Bestoriop',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 7. Bagnon
    {
        name: 'Bagnon',
        gitUrl: 'https://github.com/RichSteini/Bagnon-3.3.5',
        description: 'Bag mod',
        author: 'RichSteini',
        category: 'ui-enhancement',
    },
    // 8. Chat Filter
    {
        name: 'Chat Filter',
        gitUrl: 'https://github.com/sousou63/ChatFilter',
        description: 'Lightweight chat filter (Epoch Addon)',
        author: 'sousou63',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 9. ClassicAPI
    {
        name: 'ClassicAPI',
        gitUrl: 'https://share.google/mORNZjd5KjIJn72dn',
        description: 'Brings the Classic API functionality to 3.3.5',
        category: 'utility',
    },
    // 10. Cleanerlootmessages
    {
        name: 'CleanerLootMessages',
        gitUrl: 'https://www.curseforge.com/wow/addons/cleanerlootmessages/files/3998350',
        description: 'Reduces loot messages for clarity',
        category: 'utility',
    },
    // 11. Color Picker Plus
    {
        name: 'Color Picker Plus',
        gitUrl: 'https://www.curseforge.com/wow/addons/colorpickerplus/files/all?page=1&pageSize=20&version=7.2.0',
        description: 'Replaces the standard Color Picker',
        category: 'ui-enhancement',
    },
    // 12. CombatLogFix
    {
        name: 'CombatLogFix',
        gitUrl: 'https://github.com/KhalGH/CombatLogFix-WotLK',
        description: 'Fixes known bugs with the combat log',
        category: 'combat',
    },
    // 13. CompactRaidFrames
    {
        name: 'CompactRaidFrames',
        gitUrl: 'https://gitlab.com/Tsoukie/compactraidframe-3.3.5',
        description: 'Compacts Blizzard raid frames',
        category: 'ui-enhancement',
    },
    // 14. ConsolePort
    {
        name: 'ConsolePort',
        gitUrl: 'https://github.com/leoaviana/ConsolePortLK',
        description: 'ConsolePort integration',
        author: 'leoaviana',
        category: 'utility',
        notes: 'See instructions at https://github.com/leoaviana/WoWmapperX',
    },
    // 15. CircadianRhythm
    {
        name: 'CircadianRhythm',
        gitUrl: 'https://github.com/Bennylavaa/CircadianRhythm',
        description: 'Shows the time of day/night (Epoch Addon)',
        author: 'Bennylavaa',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 16. Cromulent
    {
        name: 'Cromulent',
        gitUrl: 'https://felbite.com/addon/4119-cromulent/',
        description: 'Shows zone level and fishing level',
        category: 'utility',
    },
    // 17. Decursive
    {
        name: 'Decursive',
        gitUrl: 'https://cdn.discordapp.com/attachments/1410021039500689505/1411848406548091061/Decursive.zip',
        description: 'Simplifies decursing targets with one click',
        category: 'combat',
        installable: false,
        manualUrl:
            'https://cdn.discordapp.com/attachments/1410021039500689505/1411848406548091061/Decursive.zip',
    },
    // 18. Details
    {
        name: 'Details',
        gitUrl: 'https://github.com/Bunny67/Details-WotLK',
        description: 'Damage/healing meter',
        author: 'Bunny67',
        category: 'combat',
    },
    // 19. DragonUI
    {
        name: 'DragonUI',
        gitUrl: 'https://github.com/NeticSoul/DragonUI',
        description: 'Dragonflight UI in Epoch (Epoch Addon)',
        author: 'NeticSoul',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 20. ElvUI Epoch
    {
        name: 'ElvUI Epoch',
        gitUrl: 'https://github.com/Bennylavaa/ElvUI-Epoch',
        description: 'ElvUI for Epoch (Epoch Addon)',
        author: 'Bennylavaa',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 21. Epoch Drops
    {
        name: 'Epoch Drops',
        gitUrl: 'https://github.com/sebastianpiresmolin/epoch-drops',
        description: 'Database collection tool (Epoch Addon)',
        author: 'sebastianpiresmolin',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 22. FSRTracker
    {
        name: 'Five Second Rule',
        gitUrl: 'https://github.com/RetroUnreal/FSRTracker',
        description: 'Track the five second rule for drinking (Epoch Addon)',
        author: 'RetroUnreal',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 23. Grid2
    {
        name: 'Grid2',
        gitUrl: 'https://felbite.com/addon/4264-grid2/',
        description: 'Raid frame replacement',
        category: 'ui-enhancement',
    },
    // 24. Group Bulletin Board
    {
        name: 'GroupBulletinBoard',
        gitUrl: 'https://github.com/sogladev/GroupBulletinBoard-epoch',
        description: 'LFG bulletin board like tool (Epoch Addon)',
        author: 'sogladev',
        category: 'social',
        notes: 'Alternatives: gimikz, olliebdev',
        serverCompat: { projectEpoch: true },
    },
    // 25. Hardcore Death Announcer
    {
        name: 'Hardcore Death Announcer',
        gitUrl: 'https://github.com/TVBrowntown/HCDeaths/',
        description: 'Announces hardcore deaths (Epoch Addon)',
        author: 'TVBrowntown',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 26. Tidy Plates
    {
        name: 'TidyPlates',
        gitUrl: 'https://github.com/bkader/TidyPlates_WoTLK',
        description: 'Nameplate replacement',
        author: 'bkader',
        category: 'ui-enhancement',
    },
    // 27. Trade Logger
    {
        name: 'TradeLogger',
        gitUrl: 'https://github.com/mastersuleman/WoW-Tradelogger',
        description: 'Logs trades (Epoch Addon)',
        author: 'mastersuleman',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 28. TradeSkillMaster
    {
        name: 'TradeSkillMaster',
        gitUrl: 'https://github.com/eyougo/TradeSkillMaster',
        description: 'AH replacement',
        author: 'eyougo',
        category: 'utility',
    },
    // 29. Twitch Emotes
    {
        name: 'TwitchEmotes',
        gitUrl: 'https://github.com/sogladev/TwitchEmotes-335',
        description: 'Static and animated Twitch emotes in chat (Epoch Addon)',
        author: 'sogladev',
        category: 'social',
    },
    // 30. Vanilla Graphics Boost
    {
        name: 'Vanilla Graphics Boost',
        gitUrl: 'https://github.com/fleekx/VanillaGraphicBoost',
        description: 'Auto applies graphics boosts via macros (Epoch Addon)',
        author: 'fleekx',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 31. VuhDo
    {
        name: 'VuhDo',
        gitUrl: 'https://github.com/NoM0Re/WoW-3.3.5a-Addons/raw/main/src/Addons/VuhDo.zip',
        description: 'Click-cast raid frame replacement',
        author: 'NoM0Re',
        category: 'ui-enhancement',
    },
    // 32. What''s Training
    {
        name: 'WhatsTraining',
        gitUrl: 'https://github.com/ZythDr/Whats-Training-Epoch',
        description: 'Shows your class spells in a new tab (Epoch Addon)',
        author: 'ZythDr',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
    // 33. WeakAuras
    {
        name: 'WeakAuras',
        gitUrl: 'https://github.com/NoM0Re/WeakAuras-WotLK',
        description: 'Master list of working Epoch WA',
        author: 'NoM0Re',
        category: 'utility',
    },
    // 34. WIM
    {
        name: 'WIM',
        gitUrl: 'https://warperia.com/addon-wotlk/wowinstantmessenger/',
        description: 'Messaging enhancement',
        category: 'social',
    },
    // 35. Zygor Guides Viewer
    {
        name: 'ZygorGuidesViewer',
        gitUrl: 'https://github.com/SimonGaufreteau/ZygorGuidesViewer',
        description: 'Leveling guide (Epoch Addon)',
        author: 'SimonGaufreteau',
        category: 'utility',
        serverCompat: { projectEpoch: true },
    },
]

/**
 * Get addons by category
 */
export function getAddonsByCategory(category: AddonCategory): CatalogueAddon[] {
    return ADDON_CATALOGUE.filter((addon) => addon.category === category)
}

/**
 * Search addons by name or description
 */
export function searchAddons(query: string): CatalogueAddon[] {
    const searchTerm = query.toLowerCase().trim()
    if (!searchTerm) return ADDON_CATALOGUE

    return ADDON_CATALOGUE.filter(
        (addon) =>
            addon.name.toLowerCase().includes(searchTerm) ||
            addon.description.toLowerCase().includes(searchTerm) ||
            addon.author?.toLowerCase().includes(searchTerm) ||
            (addon.serverCompat?.projectEpoch && 'epoch' === searchTerm)
    )
}

/**
 * Get all unique categories
 */
export function getAllCategories(): AddonCategory[] {
    return Array.from(new Set(ADDON_CATALOGUE.map((addon) => addon.category)))
}

/**
 * Get category display name
 */
export function getCategoryDisplayName(category: AddonCategory): string {
    const displayNames: Record<AddonCategory, string> = {
        'ui-enhancement': 'UI Enhancement',
        combat: 'Combat & Meters',
        questing: 'Questing',
        utility: 'Utility',
        social: 'Social & Communication',
        profession: 'Professions & Economy',
        pvp: 'PvP',
        cosmetic: 'Cosmetic & Themes',
        maps: 'Maps & Navigation',
        leveling: 'Leveling & Guides',
        economy: 'Economy & Trading',
        bags: 'Bags & Inventory',
    }

    return displayNames[category] || category
}
