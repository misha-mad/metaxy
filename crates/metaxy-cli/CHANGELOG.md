# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.5.0...metaxy-cli-v0.5.1) - 2026-03-26

### Fixed

- *(metaxy-cli)* fire onRequest and onError lifecycle hooks in rpcStream
- *(metaxy-cli)* apply callOptions.timeout to rpcStream connection
- *(metaxy-cli)* merge callOptions.signal with internal AbortController in stream wrappers
- *(metaxy-cli)* handle SSE event:error in rpcStream parser

### Other

- Merge pull request #101 from misha-mad/test/rpc-fetch-coverage
- *(metaxy-cli)* add rpcFetch coverage for headers, retry, timeout, GET serialisation and response unwrapping
- *(metaxy-cli,metaxy-macro)* add missing stream coverage for timeout+input, init+timeout, and JsDoc overloads

## [0.5.0](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.4.1...metaxy-cli-v0.5.0) - 2026-03-21

### Added

- *(metaxy-cli)* add createStream/useStream to framework wrappers
- *(metaxy-cli)* add stream() method with SSE parsing to RPC client codegen
- *(metaxy-cli)* add streams section to Procedures type in TypeScript codegen
- *(metaxy-cli)* recognize #[rpc_stream] in parser and skip StreamSender param
- *(metaxy-cli)* add ProcedureKind::Stream variant

### Fixed

- make StreamSender generic for typed chunk extraction

### Other

- add comprehensive streaming test coverage
- *(metaxy-cli)* add React, Vue, SolidJS to architecture overview
- *(metaxy-cli)* replace mod.rs with new-style module file

## [0.4.1](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.4.0...metaxy-cli-v0.4.1) - 2026-02-28

### Added

- add demo lambdas for init, timeout, and idempotent macro pages

### Fixed

- *(metaxy-cli)* allow refetch() to work when enabled is false

### Other

- *(metaxy-cli)* update snapshots for refetch() codegen change
- trim crate READMEs, link to documentation site

## [0.4.0](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.3.0...metaxy-cli-v0.4.0) - 2026-02-24

### Added

- *(rpc)* implement idempotent attribute for mutations (RFC-12)
- *(rpc-macro, rpc-cli)* add `timeout` attribute for per-procedure server-side and client-side timeouts
- *(rpc-cli)* support bigint_types config for large integer mapping
- *(rpc-cli)* support external crate type overrides in codegen
- *(rpc-cli)* support #[serde(flatten)] in codegen
- *(rpc-cli)* support tuple structs and branded newtypes in codegen
- *(rpc-cli)* support generic structs and enums in codegen

### Other

- *(rpc)* address idempotent impl review feedback
- *(rpc-macro, rpc-cli)* document timeout attribute and validate empty string
- *(rpc-cli)* reformat test assertions for improved readability
- *(ROADMAP)* fix spacing issues in markdown tables
- *(docs, rpc-cli)* improve formatting for readability
- *(rpc-cli)* improve formatting for enum generation output

## [0.3.0](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.2.2...metaxy-cli-v0.3.0) - 2026-02-22

### Added

- *(rpc-cli)* support all 4 serde enum tagging strategies
- *(rpc-cli)* add reactive options and AbortController to query wrappers
- *(rpc-cli)* add SolidJS reactive wrapper codegen (RFC-10)
- *(rpc-cli)* add Vue 3 Composition API reactive wrapper codegen (RFC-9)
- *(rpc-cli)* add React hooks wrapper codegen (RFC-8)
- *(rpc-cli)* use createQuery/createMutation in demo examples page
- *(rpc-cli)* add Svelte 5 reactive wrapper codegen (RFC-7)

### Fixed

- *(rpc-cli)* mutation isSuccess should persist through subsequent errors
- *(rpc-cli)* use controllerRef.current in interval to survive refetch()
- *(rpc-cli)* fix 6 bugs in React useQuery codegen
- *(rpc-cli)* prevent refetchInterval changes from triggering fetch in Solid
- *(rpc-cli)* fix 7 bugs in Solid createQuery codegen
- *(rpc-cli)* fix 6 bugs in Svelte createQuery codegen
- *(rpc-cli)* cache resolveEnabled() in refetch to avoid double read
- *(rpc-cli)* use generation counter to decouple UI state from AbortController
- *(rpc-cli)* isolate refetch controller and parameterize setupInterval
- *(rpc-cli)* reset polling interval on manual refetch and extract setupInterval helper
- *(rpc-cli)* fix 5 bugs in Vue useQuery codegen
- *(rpc-cli)* handle `has_default + Option<T>` in enum struct variant fields
- *(rpc-cli)* disambiguate void-query options getter and fix Vue controller safety
- *(rpc-cli)* improve SolidJS codegen correctness
- *(rpc-cli)* reuse computed input from watch source instead of re-calling getter
- *(rpc-cli)* type watch callback parameter to fix TS7031 implicit any
- *(rpc-cli)* use computed() for Vue reactive isSuccess/isError, serialize watch input
- *(rpc-cli)* use hasSucceeded flag for useMutation isSuccess
- *(rpc-cli)* fix Svelte createQuery double inputFn call and type safety
- *(rpc-cli)* fix React useQuery argument parsing and type safety
- *(rpc-cli)* adjust assertions formatting in svelte tests and fix README typo
- *(rpc-cli)* use status enum instead of data-based isSuccess in Svelte wrappers

### Other

- *(rpc-cli)* extract shared codegen logic and fix best practice violations
- *(rpc-cli)* isolate interval into separate createEffect in Solid codegen
- *(rpc-cli)* remove redundant prev* comparison in Svelte $effect
- *(rpc-cli)* replace runtime probe heuristic with VOID_QUERY_KEYS set in Vue codegen
- *(rpc-cli)* document enum tagging strategies and mark roadmap complete
- *(docs, rpc-cli)* enhance table formatting and argument alignment
- *(rpc-cli)* update RFC-10, ROADMAP, and READMEs with SolidJS fixes
- *(rpc-cli)* update all docs for SolidJS primitives (RFC-10)
- *(rpc-cli)* update all docs for Vue 3 composables (RFC-9)
- *(rpc-cli)* update all docs for React hooks and RFC directory restructure
- *(docs, rpc-cli)* enhance table formatting and argument alignment

### Added

- *(rpc-cli)* add Vue 3 composables codegen with `useQuery` and `useMutation` (RFC-9)
- *(rpc-cli)* add `--vue-output` CLI flag and `output.vue` config field
- *(rpc-cli)* add React hooks wrapper codegen with `useQuery` and `useMutation` (RFC-8)
- *(rpc-cli)* add `--react-output` CLI flag and `output.react` config field

### Fixed

- *(rpc-cli)* fix Svelte `createQuery` double `inputFn()` call in `$effect`
- *(rpc-cli)* fix React `useQuery` argument parsing with `isQueryOptions` type guard
- *(rpc-cli)* fix `isLoading` initial state when `enabled: false`
- *(rpc-cli)* fix `isSuccess` semantics with `hasFetched`/`hasSucceeded` flags
- *(rpc-cli)* replace unsafe `as Function` casts with typed signatures in both React and Svelte

## [0.2.2](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.2.1...metaxy-cli-v0.2.2) - 2026-02-21

### Added

- *(rpc-cli)* implement request deduplication for queries (RFC-6)
- *(rpc-cli)* implement per-call options for query/mutate (RFC-5)

### Other

- *(rpc-cli, rpc-macro)* apply architecture audit improvements
- *(rpc, rpc-cli)* document per-call options and request deduplication

## [0.2.1](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.2.0...metaxy-cli-v0.2.1) - 2026-02-20

### Added

- *(rpc)* add signal option to RpcClientConfig
- *(rpc)* add custom serialize/deserialize to RpcClientConfig
- *(rpc)* add retry policy and timeout to RpcClientConfig
- *(rpc)* add lifecycle hooks to RpcClientConfig
- *(rpc)* add Headers support for accessing request headers in handlers
- *(rpc-cli)* add protected endpoint demo for RpcClientConfig.headers
- *(rpc-cli)* add RpcClientConfig with fetch and headers options
- *(rpc-cli)* add type mappings for sets and smart pointers

### Other

- *(rpc-cli)* add insta snapshot tests for codegen output
- *(rpc)* workspace-wide lint config and audit fixes
- *(rpc-cli)* document lifecycle hooks in README
- update type mapping tables and mark expanded types as done
- Merge branch 'main' of misha-mad:misha-mad/metaxy into feat/expanded-type-mappings

## [0.2.0](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.1.4...metaxy-cli-v0.2.0) - 2026-02-19

### Added

- *(rpc-cli)* support serde attributes in codegen (RFC-3)

### Other

- *(rpc-cli)* address PR review findings

## [0.1.4](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.1.3...metaxy-cli-v0.1.4) - 2026-02-18

### Fixed

- resolve all clippy warnings across workspace
- *(rpc-cli)* remove flaky config tests using set_current_dir

### Other

- *(rpc-cli)* apply best practices from code audit
- *(rpc-cli)* reformat and align code for improved readability
- *(rpc-cli)* move inline tests to integration tests
- *(rpc-cli)* replace mod.rs with named module files (Rust 2018 style)
- center sponsor line, remove codecov badges from crate READMEs
- *(rpc-cli)* remove dead-code branches, exclude rpc-macro from coverage
- *(rpc-cli)* cover remaining uncovered lines across all modules
- *(rpc-cli)* exclude untestable code from tarpaulin coverage
- *(rpc-cli)* add unit tests for main.rs functions

## [0.1.3](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.1.2...metaxy-cli-v0.1.3) - 2026-02-16

### Added

- *(rpc-cli)* add CLI flags for all config-only options
- *(rpc-cli)* add `clear_screen` option to watch mode
- *(rpc-cli)* add [codegen.naming] config with fields option
- *(rpc-cli)* add preserve_docs option to [codegen] config
- *(rpc-cli)* add extension field to [output.imports] config
- *(rpc-cli)* add include/exclude glob patterns to [input] config
- *(rpc-cli)* add metaxy.config.toml support (RFC-2 Phase 1)

### Other

- add test coverage with cargo-tarpaulin and Codecov
- *(rpc-cli)* update README with `--clear-screen` flag details
- *(rpc-cli)* document metaxy.config.toml support in READMEs

## [0.1.2](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.1.1...metaxy-cli-v0.1.2) - 2026-02-15

### Other

- *(rpc-cli)* fix alignment in architecture ASCII diagram
- *(rpc-cli)* add crate-level documentation and README

## [0.1.1](https://github.com/misha-mad/metaxy/compare/metaxy-cli-v0.1.0...metaxy-cli-v0.1.1) - 2026-02-15

### Other

- release v0.1.0

## [0.1.0](https://github.com/misha-mad/metaxy/releases/tag/metaxy-cli-v0.1.0) - 2026-02-15

### Other

- add release-plz setup, improve metadata in Cargo.toml files
- Rename project references from "svelte-rust" to "metaxy" across files.
- Add contributing guidelines and update README to reference them
- Update README: Fix incorrect div alignment attribute
- Remove "Tech Stack" section from README.
- Update README formatting and links for `demo` directory
- Add dual licensing (MIT OR Apache-2.0) to crates and update README
- Update README diagram alignment for improved formatting
- Add live demo link and sponsorship section to README
- Add CI workflow and badge to README
- Fix inconsistent box widths in README diagrams
- Normalize table formatting in README.md
- Move demo app to `demo` dir
- Add TypeScript code generation and parsing support for Rust enums
- Replace `rspc` with `metaxy`, introducing a new TypeScript client, query macro, and CLI framework.
- Init
