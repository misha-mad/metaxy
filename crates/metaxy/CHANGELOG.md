# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/misha-mad/metaxy/compare/metaxy-v0.1.4...metaxy-v0.1.5) - 2026-03-21

### Added

- *(metaxy-macro)* add #[rpc_stream] attribute macro with Axum-based codegen
- *(metaxy)* add StreamSender type and streaming dependencies

### Fixed

- make StreamSender generic for typed chunk extraction
- *(metaxy)* use std::io::Error in StreamSender channel type

### Other

- add comprehensive streaming test coverage

## [0.1.4](https://github.com/misha-mad/metaxy/compare/metaxy-v0.1.3...metaxy-v0.1.4) - 2026-02-28

### Other

- trim crate READMEs, link to documentation site

## [0.1.3](https://github.com/misha-mad/metaxy/compare/metaxy-v0.1.2...metaxy-v0.1.3) - 2026-02-24

### Added

- *(rpc-macro, rpc-cli)* add `timeout` attribute for per-procedure server-side and client-side timeouts

## [0.1.2](https://github.com/misha-mad/metaxy/compare/metaxy-v0.1.1...metaxy-v0.1.2) - 2026-02-20

### Added

- *(rpc)* add Headers support for accessing request headers in handlers

### Other

- *(rpc)* workspace-wide lint config and audit fixes

## [0.1.1](https://github.com/misha-mad/metaxy/compare/metaxy-v0.1.0...metaxy-v0.1.1) - 2026-02-19

### Other

- release v0.1.4

## [0.1.0](https://github.com/misha-mad/metaxy/releases/tag/metaxy-v0.1.0) - 2026-02-18

### Added

- *(rpc)* add metaxy facade crate

### Other

- *(rpc)* add README for facade crate, update rpc-macro README
