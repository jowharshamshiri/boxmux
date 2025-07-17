# Changelog

All notable changes to BoxMux will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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

- Updated README
- Improved project structure with better organization
- Enhanced examples with more realistic use cases

### Fixed

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
- Panel selectability logic for empty tab orders
- Layout default value handling
- Panel bounds calculation accuracy
- Build errors with clap v4 API changes

### Security

- Updated all dependencies to address security vulnerabilities
- Improved input validation and sanitization

## [0.76.71204] - 2024-01-10

### Added

- Socket-based API for external control
- Real-time panel updates via socket messaging
- Script execution in background threads
- Keyboard event handling system
- Panel focus management
- Tab navigation between panels

### Changed

- Improved rendering performance
- Better memory management
- Enhanced error reporting

### Fixed

- Memory leaks in long-running applications
- Rendering glitches with rapid updates
- Panel overflow handling

## [0.76.71203] - 2024-01-05

### Added

- YAML-based configuration system
- Hierarchical panel layouts
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
- Panel positioning calculations

## [0.76.71202] - 2024-01-01

### Added

- Initial release of BoxMux
- Basic terminal UI framework
- Panel system with positioning
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
- **Discussions**: [GitHub Discussions](https://github.com/jowharshamshiri/boxmux/discussions)
- **Community**: [Discord](https://discord.gg/boxmux)

## License

BoxMux is released under the [MIT License](LICENSE).
