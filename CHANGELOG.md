# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
 - Cleanup `build.rs`

### Fixed
 - Add `links=snappy` to `Cargo.toml`
 - Add `cargo:root` and `cargo:include`
   - Dependent crates get `DEP_SNAPPY_ROOT` and `DEP_SNAPPY_INCLUDE` in their build environment

## [0.1.1] - 2024-06-22

### Added
 - Add a `CHANGELOG.md`

### Changed
 - Changed `repository` and `description` in `Cargo.toml`

## [0.1.0] - 2024-06-22

### Added
- Initial release
- Snappy: **1.2.1** (2024/05/22)

[unreleased]: https://github.com/LDeakin/rust_snappy_src/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/LDeakin/rust_snappy_src/releases/tag/v0.1.1
[0.1.0]: https://github.com/LDeakin/rust_snappy_src/releases/tag/v0.1.0
