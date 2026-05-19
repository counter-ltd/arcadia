# Hit List

Opinionated commentary. All products, companies, trademarks, and services belong to their
respective owners. Arcadia doesn't distribute proprietary software, bypass payment systems,
impersonate vendors, or encourage piracy.

What this is: a running record of software ecosystems, platforms, and products that do the
opposite of what Arcadia is trying to do — paywalls, artificial limits, rent extraction,
lock-in, or incentives structured against the user. Named for what they are, not for any
action taken against them.

---


## MacOS
### General UX/UI
[LiquidRadius](https://liquidradius.com) [NOT STARTED]

- Issues:
	- Requires SIP disabled
	- Requires FileVault disabled during installation
	- Closed-source
	- Paid utility ($6.99 lifetime)
	- Limited customization
- Notes:
	- Disabling core macOS security protections for a cosmetic utility is insane.
	- Closed-source software asking users to reduce platform security deserves scrutiny.
	- Good concept. Terrible tradeoff structure.
---
### Dock
[Docky](https://www.getdocky.com) [IN PROGRESS]

- Issues:
	- Closed-source
	- Paid “Pro” tier ($29.99 lifetime) layered onto core workflow features
	- Requires direct distribution outside the Mac App Store
	- Uses artificial feature segmentation (“Smart Stacks”, richer folders, custom icons, scripting) to push upgrades
	- Apple Silicon + macOS 26+ only
	- Early builds reported lag, onboarding issues, and installation quirks by users
	- Expands the Dock into a pseudo-workspace layer instead of fixing underlying macOS workflow fragmentation

- Notes:
	- “Native-feeling” is doing heavy marketing work here. It’s still another proprietary shell sitting on top of macOS UX debt.
	- Widgets, previews, stacks, launchers, and scripting bundled into the Dock risks recreating the same clutter desktop environments spent decades accumulating.
	- The product positions itself against Apple stagnation while recreating the exact lock-in dynamics of modern Mac utility ecosystems.
	- Direct distribution is understandable for system-level APIs, but it also removes App Store accountability and sandbox constraints.
	- Reddit launch feedback was positive overall, but multiple users immediately reported instability, lag, DNS problems, and onboarding friction during rollout.
	- Dock replacement apps are becoming an entire micro-industry of patching around Apple’s refusal to evolve the Dock meaningfully.
---
### Homebrew
[BrewStation](https://github.com/hreinssondev/BrewStation) [NOT STARTED]

- Issues:
	- Closed-source frontend branding layered on top of open package infrastructure
	- Reinvents package management UX instead of simplifying Homebrew itself
	- Encourages GUI dependency for workflows already solvable in terminal-native tooling
	- Adds another abstraction layer over Homebrew troubleshooting and state management
	- Electron/Tauri-style desktop utility trend continues bloating simple workflows into full applications
	- Relies heavily on visual polish and convenience features over transparency and composability
	- Creates additional maintenance surface for a package manager already sensitive to environment drift

- Notes:
	- Homebrew became dominant specifically because it stayed scriptable, inspectable, and terminal-first. Wrapping it in GUI orchestration slowly erodes that advantage.
	- “User-friendly package manager UI” keeps becoming shorthand for hiding system behavior until something breaks.
	- GUI package managers often age worse than CLI tooling because every upstream Homebrew change risks desyncing assumptions made by the frontend.
	- Projects like this unintentionally normalize the idea that users should fear terminals instead of learning minimal command literacy.
	- macOS utility ecosystems increasingly monetize friction caused by Apple neglecting power-user workflows.
	- The screenshots look clean, but package management is one of the last areas where visibility matters more than aesthetics.