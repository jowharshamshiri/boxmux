# Contributing to BoxMux

Welcome to BoxMux! I'm excited that you're interested in contributing to making terminal interfaces more beautiful and functional. This document will guide you through the process of contributing to BoxMux.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Types of Contributions](#types-of-contributions)
- [Development Process](#development-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Review Process](#review-process)

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read and follow these guidelines:

### Our Pledge

I pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, sex characteristics, gender identity and expression, level of experience, education, socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

**Positive behavior includes:**

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism

**Unacceptable behavior includes:**

- Harassment, trolling, or derogatory comments
- Public or private harassment
- Publishing others' private information without permission
- Other conduct which could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project team. All complaints will be reviewed and investigated promptly and fairly.

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust** (latest stable version) - [Install Rust](https://rustup.rs/)
- **Git** for version control
- **A Unix-like system** (Linux, macOS, or WSL)
- **Basic terminal knowledge**

### First Steps

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:

   ```bash
   git clone https://github.com/jowharshamshiri/boxmux.git
   cd boxmux
   ```

3. **Set up the upstream remote**:

   ```bash
   git remote add upstream https://github.com/original-owner/boxmux.git
   ```

4. **Build the project**:

   ```bash
   cargo build
   ```

5. **Run the tests**:

   ```bash
   cargo test
   ```

## Development Setup

### Environment Setup

1. **Install development dependencies**:

   ```bash
   # For linting and formatting
   rustup component add rustfmt clippy
   
   # For documentation generation
   cargo install cargo-doc
   
   # For test coverage (optional)
   cargo install cargo-tarpaulin
   ```

2. **Set up your editor** with Rust support:
   - **VS Code**: Install the Rust Analyzer extension
   - **Vim/Neovim**: Install rust.vim and ale/coc-rust-analyzer
   - **Emacs**: Install rust-mode and lsp-mode

3. **Configure Git hooks** (optional but recommended):

   ```bash
   # Pre-commit hook for formatting
   echo '#!/bin/bash
   cargo fmt -- --check
   cargo clippy -- -D warnings' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

### Project Structure

```
boxmux/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── draw_utils.rs        # Drawing and rendering utilities
│   ├── draw_loop.rs         # Main drawing loop
│   ├── input_loop.rs        # Input handling
│   ├── thread_manager.rs    # Thread management
│   ├── socket_loop.rs       # Socket communication
│   ├── utils.rs             # Utility functions
│   └── model/              # Data structures
│       ├── mod.rs
│       ├── app.rs           # Application state
│       ├── layout.rs        # Layout definitions
│       ├── panel.rs         # Panel definitions
│       └── common.rs        # Common types
├── layouts/                 # Example configurations
├── docs/                    # Documentation
├── tests/                   # Integration tests
└── examples/               # Example applications
```

## Contributing Guidelines

### Issues and Bug Reports

When reporting bugs or requesting features:

1. **Search existing issues** to avoid duplicates
2. **Use issue templates** when available
3. **Provide detailed information**:
   - BoxMux version
   - Operating system
   - Terminal emulator
   - Steps to reproduce
   - Expected vs. actual behavior
   - Configuration files (if relevant)

### Feature Requests

For new features:

1. **Describe the problem** the feature would solve
2. **Explain the proposed solution**
3. **Provide examples** of how it would be used
4. **Consider alternatives** and their trade-offs
5. **Discuss implementation** approach if you have ideas

## Types of Contributions

### Code Contributions

- **Bug fixes**: Fix reported issues
- **New features**: Implement requested functionality
- **Performance improvements**: Optimize existing code
- **Refactoring**: Improve code structure and maintainability

### Documentation Contributions

- **API documentation**: Improve code comments and docs
- **User guides**: Create tutorials and examples
- **Configuration reference**: Document YAML options
- **Troubleshooting**: Add solutions to common problems

### Testing Contributions

- **Unit tests**: Test individual components
- **Integration tests**: Test feature interactions
- **Performance tests**: Benchmark critical paths
- **Configuration tests**: Test various YAML configurations

### Contributions

- **Issue triage**: Help categorize and prioritize issues
- **User support**: Answer questions in discussions
- **Examples**: Create real-world use cases
- **Tutorials**: Write learning materials

## Development Process

### Branching Strategy

I use a simple Git flow:

1. **main**: Stable release branch
2. **develop**: Integration branch for new features
3. **feature/***: Feature development branches
4. **hotfix/***: Critical bug fixes

### Development Workflow

1. **Create a feature branch**:

   ```bash
   git checkout -b feature/awesome-feature
   ```

2. **Make your changes**:
   - Write code following our style guidelines
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**:

   ```bash
   cargo test
   cargo clippy
   cargo fmt -- --check
   ```

4. **Commit your changes**:

   ```bash
   git add .
   git commit -m "Add awesome feature
   
   - Implement feature X
   - Add tests for feature X
   - Update documentation"
   ```

5. **Push to your fork**:

   ```bash
   git push origin feature/awesome-feature
   ```

6. **Open a pull request**

### Commit Messages

Follow conventional commit format:

```
type(scope): short description

Longer description explaining the change in detail.
Include motivation and context.

- List specific changes
- Include breaking changes if any
- Reference issues: Fixes #123
```

**Types:**

- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation updates
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

## Code Style

### Rust Style Guidelines

I follow the standard Rust style guidelines:

1. **Use `rustfmt`** for consistent formatting:

   ```bash
   cargo fmt
   ```

2. **Follow Rust naming conventions**:
   - `snake_case` for functions and variables
   - `PascalCase` for types and traits
   - `SCREAMING_SNAKE_CASE` for constants

3. **Use `clippy`** for linting:

   ```bash
   cargo clippy -- -D warnings
   ```

4. **Write idiomatic Rust**:
   - Use `match` instead of nested `if let`
   - Prefer `?` operator for error handling
   - Use iterator methods instead of loops when appropriate

### Code Organization

1. **Module structure**:
   - Keep modules focused and cohesive
   - Use `pub(crate)` for internal APIs
   - Document public interfaces

2. **Error handling**:
   - Use `anyhow` for application errors
   - Use `thiserror` for library errors
   - Provide meaningful error messages

3. **Testing**:
   - Write unit tests for all public functions
   - Use descriptive test names
   - Include edge cases and error conditions

### Documentation Standards

1. **Code comments**:

   ```rust
   /// Renders a panel with the given configuration.
   /// 
   /// # Arguments
   /// 
   /// * `panel` - The panel configuration to render
   /// * `bounds` - The screen bounds for rendering
   /// 
   /// # Returns
   /// 
   /// Returns `Ok(())` on success, or an error if rendering fails.
   pub fn render_panel(panel: &Panel, bounds: &Bounds) -> Result<()> {
       // Implementation
   }
   ```

2. **Module documentation**:

   ```rust
   //! This module handles panel rendering and drawing utilities.
   //! 
   //! The main entry point is the `render_panel` function, which
   //! handles the rendering pipeline for a panel.
   ```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a specific module
cargo test model::panel
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_panel_creation() {
        let panel = Panel::new("test_panel");
        assert_eq!(panel.id(), "test_panel");
    }
    
    #[test]
    fn test_panel_rendering() {
        // Test rendering logic
    }
    
    #[test]
    #[should_panic]
    fn test_invalid_configuration() {
        // Test error conditions
    }
}
```

### Test Coverage

I aim for high test coverage:

```bash
# Generate coverage report
cargo tarpaulin --out html

# View coverage
open tarpaulin-report.html
```

## Documentation

### Writing Documentation

1. **API documentation**: Use `///` for public APIs
2. **Examples**: Include usage examples in docs
3. **Configuration**: Document all YAML options
4. **Troubleshooting**: Include common issues and solutions

### Building Documentation

```bash
# Build documentation
cargo doc --open

# Build with private items
cargo doc --document-private-items --open
```

### Documentation Standards

- Use clear, concise language
- Include examples for complex features
- Link to related documentation
- Keep documentation up-to-date with code changes

## Submitting Changes

### Pull Request Process

1. **Ensure your branch is up-to-date**:

   ```bash
   git checkout main
   git pull upstream main
   git checkout feature/awesome-feature
   git rebase main
   ```

2. **Create a pull request** with:
   - Clear title and description
   - Link to related issues
   - Description of changes
   - Testing instructions
   - Screenshots (if UI changes)

3. **PR template**:

   ```markdown
   ## Description
   Brief description of the changes.
   
   ## Related Issues
   Fixes #123
   
   ## Changes Made
   - Added feature X
   - Fixed bug Y
   - Updated documentation
   
   ## Testing
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Manual testing completed
   
   ## Screenshots
   (If applicable)
   ```

### PR Requirements

Before submitting a PR:

- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] CHANGELOG updated (if applicable)
- [ ] No merge conflicts
- [ ] Descriptive commit messages

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas

### Getting Help

If you need help:

1. **Check existing documentation**
2. **Search GitHub issues**

### Recognition

I value all contributions:

- Contributors are listed in CONTRIBUTORS.md
- Significant contributions are highlighted in releases
- I provide feedback and mentoring for new contributors

### Maintainer Responsibilities

Maintainers will:

- Review PRs promptly
- Provide constructive feedback
- Help onboard new contributors
- Maintain project quality standards
- Communicate project direction

## Development Tips

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with specific module logging
RUST_LOG=boxmux::draw_utils=debug cargo run

# Use debugger
rust-gdb target/debug/boxmux
```

### Performance Profiling

```bash
# Profile with perf
cargo build --release
perf record -g ./target/release/boxmux
perf report
```

### Memory Debugging

```bash
# Check for memory leaks
valgrind --leak-check=full ./target/debug/boxmux
```

## Release Process

### Version Numbering

I use semantic versioning (SemVer):

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Steps

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. Tag release after merge
5. Publish to crates.io (maintainers)
