import SwiftUI

struct SidebarView: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    /// Matches `SETTINGS_HUB_ROOT_PAGE_ID` in `navigation.rs`.
    private static let settingsHubRootId = "global.settings"

    let registry: NavigationRegistry
    let sidebarWidth: CGFloat
    let sidebarSwipeThreshold: CGFloat
    let isPageVisible: (String) -> Bool
    let remoteSessionEnabled: Bool
    @Binding var remoteRoute: String?
    let remoteTargets: [RemoteTarget]
    let refreshRemoteTargets: () -> Void

    @Binding var activeGroupID: String
    @Binding var activePageID: String

    @State private var settingsHubExpanded = false

    private var visibleGroups: [GroupDefinition] {
        registry.groups.filter { group in
            group.pageIDs.contains { isPageVisible($0) }
        }
    }

    private var activeGroup: GroupDefinition {
        visibleGroups.first(where: { $0.id == activeGroupID })
            ?? visibleGroups.first
            ?? registry.groups[0]
    }

    private var activeGroupPages: [PageDefinition] {
        activeGroup.pageIDs
            .filter { isPageVisible($0) }
            .compactMap { id in registry.pages.first(where: { $0.id == id }) }
    }

    /// Global sidebar rows excluding the Settings hub root (that row is rendered as an expandable group).
    private var flatGlobalPageIDs: [String] {
        registry.globalPages.filter { $0 != Self.settingsHubRootId && isPageVisible($0) }
    }

    private var visibleSettingsHubPageIDs: [String] {
        registry.settingsHubPages.filter { isPageVisible($0) }
    }

    private var sessionChipTitle: String {
        guard let route = remoteRoute, route.hasPrefix("lan:") else { return "Local" }
        let ip = String(route.dropFirst(4))
        return remoteTargets.first(where: { $0.ip == ip })?.hostname ?? ip
    }

    private func selectGroup(_ groupID: String) {
        activeGroupID = groupID
        if let group = visibleGroups.first(where: { $0.id == groupID }),
           let firstPageID = group.pageIDs.first(where: { isPageVisible($0) }) {
            activePageID = firstPageID
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .center, spacing: 10) {
                Text("Arcadia")
                    .font(.system(size: 28, weight: .semibold, design: .rounded))
                    .foregroundStyle(theme.primaryTextColor)

                if remoteSessionEnabled {
                    Menu {
                        Button("Local") {
                            remoteRoute = nil
                            refreshRemoteTargets()
                        }
                        ForEach(remoteTargets) { target in
                            Button("\(target.hostname) (\(target.ip))") {
                                remoteRoute = "lan:\(target.ip)"
                                refreshRemoteTargets()
                            }
                        }
                    } label: {
                        HStack(spacing: 4) {
                            Text(sessionChipTitle)
                            Image(systemName: "chevron.down")
                                .font(.system(size: 7, weight: .bold))
                        }
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(theme.secondaryTextColor)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(theme.cardFillColor, in: Capsule())
                        .overlay {
                            Capsule()
                                .stroke(theme.cardStrokeColor, lineWidth: 1)
                        }
                    }
                    .onAppear { refreshRemoteTargets() }
                }
            }
            .padding(.horizontal, 22)
            .padding(.top, 28)
            .padding(.bottom, 10)

            Text("Groups")
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.tertiaryTextColor)
                .padding(.horizontal, 16)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(visibleGroups) { group in
                        groupTabButton(group: group)
                    }
                }
                .padding(.horizontal, 14)
            }

            ScrollView(.vertical, showsIndicators: false) {
                VStack(alignment: .leading, spacing: 8) {
                    Text(activeGroup.label)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(theme.tertiaryTextColor)
                        .padding(.top, 2)
                        .padding(.horizontal, 16)

                    ForEach(activeGroupPages) { page in
                        pageButton(page: page)
                    }
                }
                .padding(.bottom, 8)
            }
            .frame(maxHeight: .infinity)

            Text("Global")
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.tertiaryTextColor)
                .padding(.horizontal, 16)

            ForEach(flatGlobalPageIDs, id: \.self) { pageID in
                if let page = registry.pages.first(where: { $0.id == pageID }) {
                    pageButton(page: page)
                }
            }

            if isPageVisible(Self.settingsHubRootId), let hubPage = registry.pages.first(where: { $0.id == Self.settingsHubRootId }) {
                settingsHubSection(hubPage: hubPage)
            }
        }
        .padding(.bottom, 14)
        .frame(width: sidebarWidth)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(.ultraThinMaterial)
        .background(colorScheme == .dark ? .white.opacity(0.05) : .white.opacity(0.3))
        .overlay {
            RoundedRectangle(cornerRadius: 0)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
        .shadow(color: theme.sidebarShadowColor, radius: 28, x: 8, y: 0)
        .ignoresSafeArea()
        .onChange(of: activePageID) { _, newVal in
            if newVal == Self.settingsHubRootId || registry.settingsHubPages.contains(newVal) {
                settingsHubExpanded = true
            }
        }
    }

    @ViewBuilder
    private func settingsHubSection(hubPage: PageDefinition) -> some View {
        let pal = theme.navAccentPalette(hubPage.accent)
        let hubHighlighted =
            activePageID == Self.settingsHubRootId || visibleSettingsHubPageIDs.contains(activePageID)
        VStack(alignment: .leading, spacing: 8) {
            Button {
                activePageID = Self.settingsHubRootId
                withAnimation(.easeInOut(duration: 0.2)) {
                    settingsHubExpanded.toggle()
                }
            } label: {
                HStack(spacing: 12) {
                    Image(systemName: hubPage.systemImage)
                        .font(.system(size: 16, weight: .semibold))
                        .frame(width: 20)
                    Text(hubPage.title)
                        .frame(maxWidth: .infinity, alignment: .leading)
                    Image(systemName: "chevron.right")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundStyle(theme.tertiaryTextColor)
                        .rotationEffect(.degrees(settingsHubExpanded ? 90 : 0))
                }
                .font(.body.weight(hubHighlighted ? .semibold : .medium))
                .foregroundStyle(hubHighlighted ? pal.iconActive : theme.primaryTextColor)
                .padding(.horizontal, 16)
                .frame(height: 50)
                .background(
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .fill(hubHighlighted ? pal.selectedFill : .clear)
                )
                .overlay {
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .stroke(hubHighlighted ? pal.iconActive.opacity(0.38) : .clear, lineWidth: 1)
                }
            }
            .buttonStyle(.plain)
            .padding(.horizontal, 14)

            if settingsHubExpanded {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(visibleSettingsHubPageIDs, id: \.self) { pageID in
                        if let page = registry.pages.first(where: { $0.id == pageID }) {
                            settingsHubChildRow(page: page)
                        }
                    }
                }
                .padding(.leading, 12)
                .overlay(alignment: .leading) {
                    Rectangle()
                        .fill(theme.cardStrokeColor.opacity(0.85))
                        .frame(width: 2)
                        .padding(.leading, 6)
                }
            }
        }
    }

    private func settingsHubChildRow(page: PageDefinition) -> some View {
        let isActive = activePageID == page.id
        let pal = theme.navAccentPalette(page.accent)
        return Button {
            activePageID = page.id
        } label: {
            HStack(spacing: 12) {
                Image(systemName: page.systemImage)
                    .font(.system(size: 15, weight: .semibold))
                    .frame(width: 20)
                Text(page.title)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .font(.callout.weight(isActive ? .semibold : .medium))
            .foregroundStyle(isActive ? pal.iconActive : theme.primaryTextColor)
            .padding(.horizontal, 14)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .fill(isActive ? pal.selectedFill : .clear)
            )
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 14)
    }

    private func groupTabButton(group: GroupDefinition) -> some View {
        let pal = theme.navAccentPalette(group.accent)
        let active = activeGroupID == group.id
        return Button {
            selectGroup(group.id)
        } label: {
            VStack(spacing: 6) {
                Image(systemName: group.systemImage)
                    .font(.system(size: 14, weight: .semibold))
                Text(group.label)
            }
            .font(.caption.weight(active ? .semibold : .medium))
            .foregroundStyle(active ? pal.iconActive : theme.primaryTextColor)
            .frame(width: 64, height: 64)
            .background(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .fill(active ? pal.selectedFill : .clear)
            )
            .overlay {
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(active ? pal.iconActive.opacity(0.42) : .clear, lineWidth: 1)
            }
        }
        .buttonStyle(.plain)
    }

    private func pageButton(page: PageDefinition) -> some View {
        let isActive = activePageID == page.id
        let pal = theme.navAccentPalette(page.accent)
        return Button {
            activePageID = page.id
        } label: {
            HStack(spacing: 12) {
                Image(systemName: page.systemImage)
                    .font(.system(size: 16, weight: .semibold))
                    .frame(width: 20)
                Text(page.title)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .font(.body.weight(isActive ? .semibold : .medium))
            .foregroundStyle(isActive ? pal.iconActive : theme.primaryTextColor)
            .padding(.horizontal, 16)
            .frame(height: 50)
            .background(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill(isActive ? pal.selectedFill : .clear)
            )
            .overlay {
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .stroke(isActive ? pal.iconActive.opacity(0.38) : .clear, lineWidth: 1)
            }
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 14)
    }
}
