# Changelog

All notable changes to BoxMux will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

This release rebuilds BoxMux's runtime, renderer and terminal emulator. Configurations written
for earlier versions need the changes listed under "Migrating".

### Migrating

- `pty: true` and `thread: true` are gone: use `execution_mode: Pty` or `execution_mode: Thread`
  (the default is `Immediate`). The removed keys are now load errors, as is every unknown key.
- `bounds` is gone: use `position`.
- `tab_order` is an integer (`tab_order: 2`, not `"2"`); focus order is numeric.
- A position without `%` is a number of cells, as before; percentages may have up to two
  decimals. Positions beyond the parent are now clamped to it, and anything else (`10px`,
  `abc`) is a load error instead of 0.
- Colors are one of `default` and the 16 ANSI names (`red`, `bright_red`, ...); other names are
  load errors. Base and bright colors are now different colors.
- `overflow_behavior` is `scroll`, `wrap` or `truncate`; `title_position` is `left`, `center` or
  `right`.
- `table_config` accepts only the documented keys; `border_style: custom` is gone.
- Plugins use a new C ABI with JSON requests (see the plugin documentation) and are listed under
  `app.plugins`.
- Command line: `-d/--frame-delay` (was `--frame_delay`), `--log-file` and `--log-level` have no
  short forms, `update_box_content` is `replace_box_content` (with `--failed` instead of a
  true/false argument), `update_box_script` is `replace_box_script`, script arguments are one
  argument per line, `spawn_pty_process --libs` is gone (use `app.libs`), and the new `--socket`
  option chooses the control socket.
- The control socket speaks one JSON request and one JSON reply per connection; see the API
  documentation. Its default location is per user (`$XDG_RUNTIME_DIR/boxmux.sock`,
  `/tmp/boxmux-<uid>.sock`, or the `boxmux-<user>` pipe on Windows).

### Added

- Windows support (ConPTY terminals, PowerShell scripts by default) and the `app.shell` setting.
- Every control command is implemented and answers with a result or an error; `send_signal`,
  `query_pty_status` and box replacement work.
- Keyboard input, paste and (when the program asks for it) mouse input reach PTY programs;
  terminal queries (cursor position, device attributes) are answered.
- Charts and tables can show the output of the box's script; tables render.
- `min_*`/`max_*` sizes apply around the `anchor`; `next_focus_id` steers Tab.
- `fill: false` leaves a box's background unpainted; `highlighted_*` settings apply under the
  mouse pointer.
- Variables: defaults may contain references (`${A:${B}}`); `$${` writes a literal `${`.
- PTY programs can copy to the clipboard (OSC 52); 24-bit color in PTY boxes.
- The light or dark theme follows the terminal's background color when the terminal reports it.
- An example plugin (`examples/plugins/gauge`).
- Release and documentation scripts (`scripts/release.sh`, `scripts/deploy-docs.sh`) and
  cross-platform test plans for mg-test, replacing the GitHub workflows.

### Fixed

- Control commands no longer hang.
- A box without `refresh_interval` runs its script once instead of restarting it in a loop, and a
  refresh never starts while the previous run is still going.
- Quitting, closing a tab or removing a box stops the processes it started, including their
  children; termination signals and panics restore the terminal.
- Moving or resizing a box no longer writes keys the configuration rejects, keeps comments and
  units, and is not done for a click without movement or with `--lock`.
- Text with wide characters, tabs or color sequences is measured and clipped correctly; long lines
  wrap without panicking.
- Clicks land on what is drawn, for nested and overlapping boxes alike.
- The tab bar can always reach its last tab.
- Escape sequences split between two reads of a PTY are understood; deferred line wrapping,
  wide characters and backspace behave like a real terminal.
- Configuration errors name the file, line and setting, and the program exits with an error.

### Removed

- The message-bus architecture, live YAML sync, the visual testing framework and unused UI
  components.
- Homebrew formula and Debian packaging leftovers; `target-cpu=native` release builds.

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
