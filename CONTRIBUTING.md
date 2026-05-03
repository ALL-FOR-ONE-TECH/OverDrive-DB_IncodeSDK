# Contributing to OverDrive InCode SDK

Thank you for your interest in contributing! 🎉

## How to Contribute

### Reporting Bugs

1. Check [existing issues](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/issues) first
2. Create a new issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - Platform (Windows/Linux/macOS) and SDK version

### Feature Requests

Open an issue with the `enhancement` label describing:
- What problem it solves
- Proposed API design
- Example usage

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Submit a PR with a clear description

## Development Setup

```bash
# Clone
git clone https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK.git
cd OverDrive-DB_SDK

# Build
cargo build

# Test
cargo test

# Build release binary
cargo build --release
```

## Code Style

- Follow standard Rust formatting: `cargo fmt`
- No clippy warnings: `cargo clippy`
- Document all public APIs with `///` doc comments
- Add tests for new features

## License

By contributing, you agree that your contributions will be licensed under the MIT/Apache-2.0 dual license.
