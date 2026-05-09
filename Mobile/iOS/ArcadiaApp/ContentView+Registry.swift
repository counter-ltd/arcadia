import Foundation

extension ContentView {
    static func loadNavigationRegistry() -> NavigationRegistry {
        let fallback = NavigationRegistry(
            pages: [
                PageDefinition(id: "utility.shell", title: "Terminal", description: "Run and manage terminal commands.", glyph: "TM", systemImage: "terminal", accent: "emerald", requiredModule: ModuleNames.terminal),
                PageDefinition(id: "global.dashboard", title: "Dashboard", description: "Overview of the Arcadia application surface.", glyph: "DH", systemImage: "house", accent: "violet"),
                PageDefinition(id: "global.logs", title: "Logs", description: "Recent logs and activity stream appear here.", glyph: "LG", systemImage: "doc.text.magnifyingglass", accent: "sky"),
                PageDefinition(id: "global.settings", title: "Settings", description: "App preferences and configuration controls appear here.", glyph: "ST", systemImage: "gearshape", accent: "indigo"),
                PageDefinition(id: "global.modules", title: "Modules", description: "Manage global module availability and dependency requirements.", glyph: "MD", systemImage: "switch.2", accent: "fuchsia"),
                PageDefinition(id: "network.overview", title: "Overview", description: "Network status and module connectivity overview.", glyph: "NW", systemImage: "network", accent: "teal", requiredModule: ModuleNames.net),
                PageDefinition(id: "network.nodes", title: "Nodes", description: "Discover LAN peers and manage pairing with lan.scan / lan.node.", glyph: "ND", systemImage: "rectangle.connected.to.line.under.fill", accent: "cyan", requiredModule: ModuleNames.lan),
                PageDefinition(id: "late.now_playing", title: "Late.sh", description: "Live chat, now playing, votes, visualizer, and bonsai in one view.", glyph: "NP", systemImage: "music.note", accent: "violet", requiredModule: ModuleNames.late)
            ],
            groups: [
                GroupDefinition(id: "utilities", label: "Utilities", glyph: "UT", systemImage: "wrench.and.screwdriver", pageIDs: ["utility.shell"], accent: "amber"),
                GroupDefinition(id: "network", label: "Network", glyph: "NW", systemImage: "network", pageIDs: ["network.overview", "network.nodes"], accent: "cyan"),
                GroupDefinition(id: "social", label: "Social", glyph: "SC", systemImage: "bubble.left.and.bubble.right.fill", pageIDs: ["late.now_playing"], accent: "teal")
            ],
            globalPages: ["global.dashboard", "global.settings", "global.modules"],
            defaultGroup: "utilities",
            defaultPage: "global.dashboard"
        )

        let payload = navigationRegistryJson()
        guard let data = payload.data(using: .utf8),
              let decoded = try? JSONDecoder().decode(NavigationRegistry.self, from: data),
              !decoded.pages.isEmpty,
              !decoded.groups.isEmpty else {
            return fallback
        }
        return decoded
    }
}
