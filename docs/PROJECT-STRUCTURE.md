# Project Structure

```
metaxy/
├── crates/
│   ├── metaxy/                   # Facade crate (re-exports macros + runtime deps)
│   │   └── src/
│   │       ├── lib.rs            #   pub use metaxy_macro::{rpc_query, rpc_mutation, rpc_stream}
│   │       └── stream.rs         #   StreamSender<T>, SendError (SSE chunk sender)
│   ├── metaxy-macro/             # Proc-macro crate
│   │   └── src/
│   │       ├── lib.rs            #   #[rpc_query] / #[rpc_mutation] / #[rpc_stream]
│   │       ├── codegen_stream.rs #   Stream handler codegen (Axum + VercelLayer)
│   │       └── tests.rs          #   Macro expansion tests
│   └── metaxy-cli/               # CLI crate (library + binary: `metaxy`)
│       ├── src/
│       │   ├── lib.rs            #   Library root — public module declarations
│       │   ├── main.rs           #   CLI entry (clap arg parsing)
│       │   ├── commands.rs       #   scan / generate command implementations
│       │   ├── config.rs         #   metaxy.config.toml loading & merging
│       │   ├── model.rs          #   Manifest, Procedure, RustType, StructDef, EnumDef, FieldDef
│       │   ├── parser.rs         #   Module declarations for parser/
│       │   ├── parser/           #   Rust source → Manifest (via syn)
│       │   │   ├── extract.rs    #     File scanning & procedure extraction
│       │   │   ├── serde.rs      #     #[serde(...)] attribute parsing
│       │   │   └── types.rs      #     syn::Type → RustType conversion
│       │   ├── codegen.rs        #   Module declarations for codegen/
│       │   ├── codegen/          #   Manifest → TypeScript
│       │   │   ├── common.rs     #     Shared codegen logic (header, framework helpers)
│       │   │   ├── typescript.rs #     RustType → TS type mapping + rpc-types.ts
│       │   │   ├── client.rs     #     RpcClient interface + rpc-client.ts
│       │   │   ├── overrides.rs  #     Type override resolution
│       │   │   ├── svelte.rs     #     Svelte 5 reactive wrappers (opt-in)
│       │   │   ├── react.rs      #     React hooks wrappers (opt-in)
│       │   │   ├── vue.rs        #     Vue 3 composable wrappers (opt-in)
│       │   │   └── solid.rs      #     SolidJS reactive primitives (opt-in)
│       │   └── watch.rs          #   File watcher with debounce
│       └── tests/                # Integration tests
│           ├── common/mod.rs     #   Shared test helpers
│           ├── commands.rs       #   scan / generate / write_file / bytecount
│           ├── config.rs         #   Config parsing, discovery, CLI overrides
│           ├── extract.rs        #   Parser extraction from Rust source
│           ├── types.rs          #   syn::Type → RustType + RenameRule conversion
│           ├── typescript.rs     #   TypeScript codegen (type mapping, JSDoc, serde)
│           ├── client.rs         #   Client codegen (RpcClient, overloads, stream)
│           ├── overrides.rs      #   Type override tests
│           ├── svelte.rs         #   Svelte codegen (createQuery, createMutation, createStream)
│           ├── react.rs          #   React codegen (useQuery, useMutation, useStream)
│           ├── vue.rs            #   Vue codegen (useQuery, useMutation, useStream)
│           ├── solid.rs          #   SolidJS codegen (createQuery, createMutation, createStream)
│           └── snapshots/        #   insta snapshot files (auto-generated)
├── demo/                         # Demo application (SvelteKit) + Rust lambdas
│   ├── api/                      # Rust lambdas (each file = one endpoint)
│   │   ├── hello.rs              #   GET  /api/hello?input="name"
│   │   ├── time.rs               #   GET  /api/time
│   │   ├── math.rs               #   GET  /api/math?input={a,b,op}
│   │   ├── types.rs              #   GET  /api/types (expanded type mappings demo)
│   │   ├── secret.rs             #   GET  /api/secret (headers demo)
│   │   ├── echo.rs               #   POST /api/echo (mutation)
│   │   ├── bigint_demo.rs        #   GET  /api/bigint_demo (BigInt types)
│   │   ├── cached_time.rs        #   GET  /api/cached_time (server-side caching)
│   │   ├── cached_time_private.rs #  GET  /api/cached_time_private (private cache)
│   │   ├── cached_time_stale.rs  #   GET  /api/cached_time_stale (stale-while-revalidate)
│   │   ├── cookie_demo.rs        #   GET  /api/cookie_demo (custom fetch / cookies)
│   │   ├── dedup_demo.rs         #   GET  /api/dedup_demo (request deduplication)
│   │   ├── idempotent_demo.rs    #   POST /api/idempotent_demo (idempotent mutation)
│   │   ├── init_demo.rs          #   GET  /api/init_demo (cold-start init)
│   │   ├── timeout_demo.rs       #   GET  /api/timeout_demo (timeout / abort)
│   │   ├── countdown.rs          #   POST /api/countdown (streaming countdown)
│   │   └── token_stream.rs       #   POST /api/token_stream (LLM-style streaming)
│   ├── Cargo.toml                # Rust package for demo lambdas
│   ├── metaxy.config.toml        # CLI config file
│   ├── src/
│   │   ├── app.d.ts              # SvelteKit type declarations
│   │   ├── app.css               # Global styles (Tailwind)
│   │   ├── lib/
│   │   │   ├── rpc-types.ts      # ← auto-generated types
│   │   │   ├── rpc-client.ts     # ← auto-generated client
│   │   │   ├── rpc.svelte.ts     # ← auto-generated Svelte 5 wrappers
│   │   │   ├── client.ts         #   RPC client instance (manual)
│   │   │   ├── highlight.server.ts  # Shiki syntax highlighting (build-time)
│   │   │   ├── images/
│   │   │   │   └── github.svg    #   GitHub logo for header
│   │   │   └── components/
│   │   │       ├── CodeBlock.svelte      # Syntax-highlighted code block
│   │   │       ├── Code.svelte           # Inline code badge
│   │   │       ├── Button.svelte         # Primary action button (ts/rust variants)
│   │   │       ├── DemoCard.svelte       # Interactive demo container
│   │   │       ├── CollapsibleCode.svelte # Show/hide code toggle
│   │   │       ├── OutputBox.svelte      # Result display (neutral/success/error)
│   │   │       ├── PageHeader.svelte     # Page h1 + lead paragraph
│   │   │       ├── SectionHeading.svelte # Section h2 (normal/large)
│   │   │       └── FeatureRow.svelte     # Landing page code+text row
│   │   └── routes/               # SvelteKit pages (docs site, fully prerendered)
│   ├── tests/
│   │   ├── integration/
│   │   │   └── codegen.test.ts   # Vitest: codegen pipeline tests
│   │   └── e2e/
│   │       └── rpc.test.ts       # Playwright: UI + API tests
│   ├── package.json              # Node scripts
│   ├── svelte.config.js          # SvelteKit config
│   ├── vite.config.ts            # Vite config
│   ├── vitest.config.ts          # Vitest config
│   ├── playwright.config.ts      # Playwright config
│   ├── eslint.config.js          # ESLint config
│   ├── tsconfig.json             # TypeScript config
│   └── vercel.json               # Vercel deployment config
├── docs/                         # Design documents & references
│   ├── ROADMAP.md                #   Feature roadmap by phase
│   ├── PROJECT-STRUCTURE.md      #   This file
│   ├── plans/                    #   Implementation plans & design docs
│   └── RFC/                      #   Individual RFC design documents
│       └── RFC-NNN-*.md
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                # CI: build, test, lint
│   │   ├── coverage.yml          # Code coverage (tarpaulin + codecov)
│   │   └── release-plz.yml      # Automated releases (release-plz)
│   ├── ISSUE_TEMPLATE/           # Bug report & feature request templates
│   └── DISCUSSION_TEMPLATE/      # Discussion category templates
├── Cargo.toml                    # Rust workspace (crates + demo)
├── Cargo.lock                    # Dependency lock file
├── README.md
├── CONTRIBUTING.md               # Development setup & contributing guide
├── CLAUDE.md                     # AI assistant instructions
├── release-plz.toml              # release-plz configuration
├── codecov.yml                   # Codecov configuration
└── rustfmt.toml                  # Rust formatter configuration
```
