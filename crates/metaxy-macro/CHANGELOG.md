# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.11](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.10...metaxy-macro-v0.1.11) - 2026-03-26

### Other

- *(metaxy-cli,metaxy-macro)* rustfmt formatting
- *(metaxy-cli,metaxy-macro)* verify streaming option combinations and exclusions
- *(metaxy-cli,metaxy-macro)* add missing stream coverage for timeout+input, init+timeout, and JsDoc overloads

## [0.1.10](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.9...metaxy-macro-v0.1.10) - 2026-03-21

### Added

- *(metaxy-macro)* add #[rpc_stream] attribute macro with Axum-based codegen

### Fixed

- *(metaxy-macro)* use fallback route for stream handler
- make StreamSender generic for typed chunk extraction
- *(metaxy)* use std::io::Error in StreamSender channel type

### Other

- add comprehensive streaming test coverage
- *(metaxy-macro)* fix CORS Allow-Headers and compile errors wording
- *(metaxy-macro)* document state and headers parameters in supported signatures
- *(metaxy-macro)* split lib.rs and tests.rs into focused modules

## [0.1.9](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.8...metaxy-macro-v0.1.9) - 2026-02-28

### Other

- fix grammar issues in macro doc comments
- trim crate READMEs, link to documentation site

## [0.1.8](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.7...metaxy-macro-v0.1.8) - 2026-02-24

### Added

- *(rpc)* implement idempotent attribute for mutations (RFC-12)
- *(rpc-macro, rpc-cli)* add `timeout` attribute for per-procedure server-side and client-side timeouts
- *(rpc-macro)* add `init` attribute for cold-start initialization (RFC-11)
- *(rpc-macro)* support cache attribute for server-side caching

### Fixed

- *(rpc-macro)* replace panic with syn::Error for invalid init path

### Other

- *(rpc)* address idempotent impl review feedback
- *(rpc-macro)* simplify error handling in metadata parsing
- *(rpc-macro, rpc-cli)* document timeout attribute and validate empty string
- *(rpc-macro)* address review feedback from rust best practices
- *(rpc-macro)* add parse_cache_attrs and error response tests

## [0.1.7](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.6...metaxy-macro-v0.1.7) - 2026-02-22

### Other

- *(rpc-cli)* extract shared codegen logic and fix best practice violations

## [0.1.6](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.5...metaxy-macro-v0.1.6) - 2026-02-21

### Other

- *(rpc-cli, rpc-macro)* apply architecture audit improvements

## [0.1.5](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.4...metaxy-macro-v0.1.5) - 2026-02-20

### Added

- *(rpc)* add Headers support for accessing request headers in handlers

### Other

- *(rpc)* workspace-wide lint config and audit fixes
- *(rpc-macro)* extract tests to separate file
- *(rpc-macro)* remove unnecessary newline in `lib.rs`

## [0.1.4](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.3...metaxy-macro-v0.1.4) - 2026-02-18

### Added

- *(rpc)* add metaxy facade crate

### Fixed

- *(rpc-macro)* apply best practices from code audit
- *(rpc-macro)* remove needless borrow in new_spanned call
- resolve all clippy warnings across workspace
- *(rpc-macro)* remove invalid tarpaulin skip attribute

### Other

- *(rpc)* add README for facade crate, update rpc-macro README
- *(rpc-macro)* apply best practices from code audit
- remove SvelteKit coupling from rustdoc, RFCs, and roadmap
- center sponsor line, remove codecov badges from crate READMEs
- *(rpc-macro)* skip proc-macro entry points from tarpaulin coverage
- *(rpc-macro)* inline generate_handler into proc-macro entry points
- *(rpc-macro)* cover FnArg::Receiver filter branch
- *(rpc-macro)* extract build_handler for testable code generation
- *(rpc-macro)* add unit tests for proc-macro code generation

## [0.1.3](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.2...metaxy-macro-v0.1.3) - 2026-02-16

### Other

- add test coverage with cargo-tarpaulin and Codecov

## [0.1.2](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.1...metaxy-macro-v0.1.2) - 2026-02-15

### Other

- Merge pull request #5 from misha-mad/docs/rpc-macro
- *(rpc-macro)* add crate-level documentation and README

## [0.1.1](https://github.com/misha-mad/metaxy/compare/metaxy-macro-v0.1.0...metaxy-macro-v0.1.1) - 2026-02-15

### Other

- release v0.1.0

## [0.1.0](https://github.com/misha-mad/metaxy/releases/tag/metaxy-macro-v0.1.0) - 2026-02-15

### Other

- add release-plz setup, improve metadata in Cargo.toml files
- Rename project references from "svelte-rust" to "metaxy" across files.
- Add contributing guidelines and update README to reference them
- Update README: Fix incorrect div alignment attribute
- Remove "Tech Stack" section from README.
- Update README formatting and links for `demo` directory
- Add dual licensing (MIT OR Apache-2.0) to crates and update README
- Add new RPC endpoints and enhance example app integration
- Update README diagram alignment for improved formatting
- Add live demo link and sponsorship section to README
- Add CI workflow and badge to README
- Fix inconsistent box widths in README diagrams
- Normalize table formatting in README.md
- Move demo app to `demo` dir
- Add TypeScript code generation and parsing support for Rust enums
- Improve error handling in `rpc-macro` and remove unused `vercel.json` configuration
- Replace `rspc` with `metaxy`, introducing a new TypeScript client, query macro, and CLI framework.
- Init
