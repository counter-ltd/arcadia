import Foundation

struct PageDefinition: Identifiable, Codable {
    let id: String
    let title: String
    let description: String
    let glyph: String
    let systemImage: String
    let accent: String
    /// Module registry name; when set, the page is visible only if that module is enabled.
    let requiredModule: String?

    enum CodingKeys: String, CodingKey {
        case id
        case title
        case description
        case glyph
        case systemImage = "system_image"
        case accent
        case requiredModule = "required_module"
    }

    init(
        id: String,
        title: String,
        description: String,
        glyph: String,
        systemImage: String,
        accent: String,
        requiredModule: String? = nil
    ) {
        self.id = id
        self.title = title
        self.description = description
        self.glyph = glyph
        self.systemImage = systemImage
        self.accent = accent
        self.requiredModule = requiredModule
    }
}

struct GroupDefinition: Identifiable, Codable {
    let id: String
    let label: String
    let glyph: String
    let systemImage: String
    let pageIDs: [String]
    let accent: String

    enum CodingKeys: String, CodingKey {
        case id
        case label
        case glyph
        case systemImage = "system_image"
        case pageIDs = "pages"
        case accent
    }
}

struct NavigationRegistry: Codable {
    let pages: [PageDefinition]
    let groups: [GroupDefinition]
    let globalPages: [String]
    /// Pages rendered as compact controls in the top bar (e.g. Logs, Modules).
    /// Distinct from `globalPages` (sidebar) so each surface places them appropriately.
    let topBarPages: [String]
    /// Nested pages under the sidebar Settings hub (same IDs may appear in `topBarPages`).
    let settingsHubPages: [String]
    let defaultGroup: String
    let defaultPage: String

    enum CodingKeys: String, CodingKey {
        case pages
        case groups
        case globalPages = "global_pages"
        case topBarPages = "top_bar_pages"
        case settingsHubPages = "settings_hub_pages"
        case defaultGroup = "default_group"
        case defaultPage = "default_page"
    }

    init(
        pages: [PageDefinition],
        groups: [GroupDefinition],
        globalPages: [String],
        topBarPages: [String],
        settingsHubPages: [String],
        defaultGroup: String,
        defaultPage: String
    ) {
        self.pages = pages
        self.groups = groups
        self.globalPages = globalPages
        self.topBarPages = topBarPages
        self.settingsHubPages = settingsHubPages
        self.defaultGroup = defaultGroup
        self.defaultPage = defaultPage
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        pages = try c.decode([PageDefinition].self, forKey: .pages)
        groups = try c.decode([GroupDefinition].self, forKey: .groups)
        globalPages = try c.decode([String].self, forKey: .globalPages)
        topBarPages = (try? c.decode([String].self, forKey: .topBarPages)) ?? []
        settingsHubPages = (try? c.decode([String].self, forKey: .settingsHubPages)) ?? [
            "global.logs", "global.modules", "global.settings"
        ]
        defaultGroup = try c.decode(String.self, forKey: .defaultGroup)
        defaultPage = try c.decode(String.self, forKey: .defaultPage)
    }
}
