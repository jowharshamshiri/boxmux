# Changelog

All notable changes to BoxMux will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.240.3373] - 2026-06-29

### Added

- Automatic light/dark theming with readable defaults rendered out of the box (no color configuration required)
- `--light` and `--dark` command-line flags to force a theme, overriding terminal auto-detection
- Hover highlighting on every clickable element (boxes, menu items, tabs, close buttons, scrollbar knobs) by default
- Mouse wheel now scrolls whichever box is under the cursor, not only the focused box
- Distinct focus indication on the selected box (focus-colored border and active tab)
- Render-on-change: the screen is redrawn only when content, layout, focus, or hover state changes
- Terminal self-recovery: raw mode and mouse tracking re-asserted each frame; spawned processes detached
- Documentation suite
- Getting started guide with tutorials
- configuration reference
- API documentation with examples
- Real-world examples and use cases
- Contributing guidelines
- Roadmap for future development
- Troubleshooting guide
- GitHub Pages documentation site

### Changed

- Selection, active-tab, and focus highlights use palette-safe fixed colors so they stay legible across terminal color schemes
- Color defaults are now theme-derived (light/dark) instead of fixed values; explicit YAML colors still override them
- Updated README
- Improved project structure with better organization
- Enhanced examples with more realistic use cases

### Fixed

- Dark-mode panels rendered as solid white because empty space was filled with a foreground block glyph instead of the background color (default fill character is now a space)
- The whole app background appeared dark in light mode because uncovered area used a hardcoded black default instead of the theme background
- Menu choices overdrew the tab/title bar row, causing the title tab's text and empty space to show different backgrounds (content now renders below the tab bar)
- White-on-cyan unreadable highlights caused by base ANSI colors being remapped by the terminal palette
- Title background rendering issue on initial launch
- Buffer synchronization problems in initial render cycle

## [0.76.71205] - 2024-01-15

### Added

- Test suite with 90%+ coverage
- Unit tests for all major components
- Integration tests for key workflows
- Test documentation and examples

### Changed

- Migrated from termion to crossterm for better cross-platform support
- Updated all dependencies to latest secure versions:
  - uuid 0.6.5 → 1.0
  - clap 3.2.0 → 4.0
  - diesel 1.4.8 → 2.0
- Improved error handling with anyhow and thiserror

### Fixed

- Thread manager todo!() implementation
- MuxBox selectability logic for empty tab orders
- Layout default value handling
- MuxBox bounds calculation accuracy
- Build errors with clap v4 API changes

### Security

- Updated all dependencies to address security vulnerabilities
- Improved input validation and sanitization

## [0.76.71204] - 2024-01-10

### Added

- Socket-based API for external control
- Real-time box updates via socket messaging
- Script execution in background threads
- Keyboard event handling system
- MuxBox focus management
- Tab navigation between boxes

### Changed

- Improved rendering performance
- Better memory management
- Enhanced error reporting

### Fixed

- Memory leaks in long-running applications
- Rendering glitches with rapid updates
- MuxBox overflow handling

## [0.76.71203] - 2024-01-05

### Added

- YAML-based configuration system
- Hierarchical box layouts
- Real-time data refresh intervals
- Interactive menu system
- Color and styling customization
- Script integration for dynamic content

### Changed

- Moved from hardcoded layouts to YAML configuration
- Improved terminal compatibility
- Better resource management

### Fixed

- Terminal restoration on exit
- Color rendering inconsistencies
- MuxBox positioning calculations

## [0.76.71202] - 2024-01-01

### Added

- Initial release of BoxMux
- Basic terminal UI framework
- MuxBox system with positioning
- Simple script execution
- Basic color support

### Changed

- rewrite from previous versions
- New architecture with better modularity
- Improved performance and stability

## [0.75.x] - 2023-12-xx

### Note

Previous versions were experimental and not publicly released.
This changelog starts from version 0.76.71202, which represents
the first stable release of BoxMux.

---

## Release Notes

### Version 0.76.71205 - "Documentation Release"

This release focuses on making BoxMux more accessible and user-friendly through documentation and improved developer experience.

**Key Highlights:**

- documentation suite with guides, references, and examples
- Fixed critical title background rendering issue
- Test coverage for better reliability
- Dependency updates for security and performance
- Better cross-platform support with crossterm migration

**Breaking Changes:**

- None in this release

**Migration Guide:**

- No migration required for existing configurations
- All existing YAML configurations remain compatible

**Known Issues:**

- Some charting features are still experimental
- Plugin system is not yet implemented
- Windows support is limited (planned for v0.8)

### Upcoming Releases

**v0.8 - "Enhanced Experience"** (Planned Q1 2024)

- Enhanced charting capabilities
- Configuration validation
- Performance optimizations
- Windows support improvements

**v0.9 - "Plugin System"** (Planned Q2 2024)

- Plugin architecture
- Custom components
- theming
- Multi-language scripting

**v1.0 - "Production Ready"** (Planned Q3 2024)

- Enterprise features
- networking
- Tooling
- Long-term support (LTS)

For more details, see our [Roadmap](docs/roadmap.md).

---

## Contributing

I welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to get started.

## Support

- **Documentation**: [docs/index.md](docs/index.md)
- **Issues**: [GitHub Issues](https://github.com/jowharshamshiri/boxmux/issues)

## License

BoxMux is released under the [MIT License](LICENSE).
