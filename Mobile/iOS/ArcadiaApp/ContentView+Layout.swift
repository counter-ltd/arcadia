import SwiftUI

extension ContentView {
    var sidebarOffset: CGFloat {
        isSidebarOpen
            ? min(0, sidebarDragOffset)
            : max(-sidebarWidth, -sidebarWidth + max(0, sidebarDragOffset))
    }

    var closeSidebarGesture: some Gesture {
        DragGesture(minimumDistance: 12)
            .onChanged { value in
                guard isSidebarOpen else { return }
                sidebarDragOffset = min(0, value.translation.width)
            }
            .onEnded { value in
                guard isSidebarOpen else { return }
                let closing = value.translation.width < -sidebarSwipeThreshold
                    || value.predictedEndTranslation.width < -sidebarSwipeThreshold
                withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                    isSidebarOpen = !closing
                    sidebarDragOffset = 0
                }
            }
    }

    var openSidebarGesture: some Gesture {
        DragGesture(minimumDistance: 12)
            .onChanged { value in
                guard !isSidebarOpen else { return }
                sidebarDragOffset = max(0, value.translation.width)
            }
            .onEnded { value in
                guard !isSidebarOpen else { return }
                let opening = value.translation.width > sidebarSwipeThreshold
                    || value.predictedEndTranslation.width > sidebarSwipeThreshold
                withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                    isSidebarOpen = opening
                    sidebarDragOffset = 0
                }
            }
    }

    var edgeSwipeHandle: some View {
        Rectangle()
            .fill(.clear)
            .frame(width: 24)
            .contentShape(Rectangle())
            .ignoresSafeArea()
            .gesture(openSidebarGesture)
    }

    var mainContent: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 24) {
                if activePage.id != "utility.shell" {
                    HStack(alignment: .top) {
                        VStack(alignment: .leading, spacing: 10) {
                            Text(activePage.title)
                                .font(.system(size: 40, weight: .semibold, design: .rounded))
                                .foregroundStyle(theme.primaryTextColor)

                            Text(activePage.description)
                                .font(.body)
                                .foregroundStyle(theme.secondaryTextColor)
                                .fixedSize(horizontal: false, vertical: true)
                        }

                        Spacer()

                        Image(systemName: activePage.systemImage)
                            .font(.system(size: 28, weight: .medium))
                            .foregroundStyle(theme.accentTextColor)
                            .frame(width: 58, height: 58)
                            .background(theme.cardFillColor, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
                            .overlay {
                                RoundedRectangle(cornerRadius: 18, style: .continuous)
                                    .stroke(theme.cardStrokeColor, lineWidth: 1)
                            }
                    }
                }

                contentBody

                Spacer(minLength: 0)
            }
            .padding(24)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .background {
                RoundedRectangle(cornerRadius: 30, style: .continuous)
                    .fill(theme.cardFillColor)
                    .background(
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .fill(.ultraThinMaterial)
                    )
                    .overlay {
                        RoundedRectangle(cornerRadius: 30, style: .continuous)
                            .stroke(theme.cardStrokeColor, lineWidth: 1)
                    }
                    .shadow(color: theme.contentShadowColor, radius: 40, x: 0, y: 18)
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 10)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button {
                        withAnimation(.spring(response: 0.42, dampingFraction: 0.88)) {
                            isSidebarOpen.toggle()
                        }
                    } label: {
                        Image(systemName: "sidebar.leading")
                            .font(.system(size: 18, weight: .semibold))
                            .foregroundStyle(theme.accentTextColor)
                            .frame(width: 52, height: 52)
                            .background(.ultraThinMaterial, in: Circle())
                            .overlay {
                                Circle()
                                    .fill(colorScheme == .dark ? .white.opacity(0.04) : .white.opacity(0.35))
                            }
                            .overlay {
                                Circle()
                                    .stroke(theme.cardStrokeColor, lineWidth: 1)
                            }
                            .shadow(color: theme.contentShadowColor, radius: 14, x: 0, y: 8)
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(isSidebarOpen ? "Close sidebar" : "Open sidebar")
                }
                ToolbarItemGroup(placement: .topBarTrailing) {
                    ForEach(visibleTopBarPages) { page in
                        topBarPageButton(page: page)
                    }
                }
            }
            .toolbarBackground(.hidden, for: .navigationBar)
        }
    }

    @ViewBuilder
    var contentBody: some View {
        if activePage.id == "global.modules" {
            ModulesView(modules: modules, onToggle: updateModule, onAppear: reloadModules)
        } else if activePage.id == "network.overview" {
            NetworkOverviewView(theme: theme, modules: modules)
        } else if activePage.id == "network.nodes" {
            LanNodesView(theme: theme)
        } else if activePage.id == "utility.shell" {
            ShellView(shellHistory: $shellHistory, shellCommandInput: $shellCommandInput, onRun: runShellCommand)
        } else if activePage.id == "late.now_playing" {
            LateView(theme: theme)
        } else {
            VStack(spacing: 16) {
                GlassCard(title: "Primary Surface", subtitle: "This page is rendered from shared page definitions.")
                HStack(spacing: 16) {
                    GlassMetric(title: "Sidebar", value: isSidebarOpen ? "Open" : "Closed")
                    GlassMetric(title: "Selection", value: activePage.title)
                }
            }
        }
    }

    var visibleTopBarPages: [PageDefinition] {
        navigationRegistry.topBarPages
            .filter { isPageVisible($0) }
            .compactMap { id in navigationRegistry.pages.first(where: { $0.id == id }) }
    }

    @ViewBuilder
    func topBarPageButton(page: PageDefinition) -> some View {
        let isActive = activePageID == page.id
        Button {
            activePageID = page.id
        } label: {
            Image(systemName: page.systemImage)
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(isActive ? theme.primaryTextColor : theme.accentTextColor)
                .frame(width: 52, height: 52)
                .background(
                    isActive
                        ? AnyShapeStyle(theme.cardFillColor)
                        : AnyShapeStyle(.ultraThinMaterial),
                    in: Circle()
                )
                .overlay {
                    Circle()
                        .fill(
                            isActive
                                ? (colorScheme == .dark ? .white.opacity(0.08) : .white.opacity(0.45))
                                : (colorScheme == .dark ? .white.opacity(0.04) : .white.opacity(0.35))
                        )
                }
                .overlay {
                    Circle()
                        .stroke(
                            isActive ? theme.accentTextColor.opacity(0.4) : theme.cardStrokeColor,
                            lineWidth: 1
                        )
                }
                .shadow(color: theme.contentShadowColor, radius: 14, x: 0, y: 8)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Open \(page.title)")
    }

    var glassBackground: some View {
        ZStack {
            if colorScheme == .dark {
                LinearGradient(
                    colors: [
                        Color(red: 0.05, green: 0.08, blue: 0.16),
                        Color(red: 0.07, green: 0.14, blue: 0.25),
                        Color(red: 0.03, green: 0.06, blue: 0.12)
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            } else {
                LinearGradient(
                    colors: [
                        Color(red: 0.95, green: 0.97, blue: 1.0),
                        Color(red: 0.92, green: 0.96, blue: 0.99),
                        Color(red: 0.97, green: 0.98, blue: 1.0)
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
            }

            Circle()
                .fill(colorScheme == .dark ? Color.white.opacity(0.18) : Color.white.opacity(0.6))
                .frame(width: 320)
                .blur(radius: 70)
                .offset(x: -120, y: -250)

            Circle()
                .fill(colorScheme == .dark ? Color.cyan.opacity(0.16) : Color.cyan.opacity(0.2))
                .frame(width: 360)
                .blur(radius: 90)
                .offset(x: 150, y: -160)

            Circle()
                .fill(colorScheme == .dark ? Color.blue.opacity(0.22) : Color.blue.opacity(0.16))
                .frame(width: 380)
                .blur(radius: 110)
                .offset(x: 130, y: 260)
        }
        .ignoresSafeArea()
    }
}
