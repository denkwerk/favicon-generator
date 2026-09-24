# Changelog

## [0.2.6](https://github.com/denkwerk/favicon-generator/compare/v0.2.5...v0.2.6) (2026-09-24)

### Features

* **core:** configure the manifest's icon sizes and purpose ([97c8729](https://github.com/denkwerk/favicon-generator/commit/97c8729e9b4c67dcc5e8c4af6ce3ef1448fe85f6))
* **nuxt,unplugin:** serve the files under more paths with mirrorPrefixes ([00d4a95](https://github.com/denkwerk/favicon-generator/commit/00d4a951fdd197a8db9eaaeb54543714bf2ed576))

## [0.2.5](https://github.com/denkwerk/favicon-generator/compare/v0.2.4...v0.2.5) (2026-09-24)

### Features

* **cli:** reuse generated files and Figma exports from a cache ([c45c6ad](https://github.com/denkwerk/favicon-generator/commit/c45c6adf9bc7bc30d014aaf06259b2359aa780ea))
* **core:** add the cache and cacheDir options to FaviconConfig ([ce47dd6](https://github.com/denkwerk/favicon-generator/commit/ce47dd6315292f4e07a52e3e0669b43456c036ca))
* **unplugin,nuxt:** cache in node_modules/.cache/favicon-generator and check Figma for changes ([9068b8c](https://github.com/denkwerk/favicon-generator/commit/9068b8cd01ec023aedc0209ef6a418abb6011125))

### Performance Improvements

* **cli:** request the Figma file version along with the first export ([62587e0](https://github.com/denkwerk/favicon-generator/commit/62587e0fb4a78b82e0a3bcc74614d65a47c0264c))

### Documentation

* **readme:** describe the cache and what it saves ([e26faad](https://github.com/denkwerk/favicon-generator/commit/e26faadeb78903a0476a228874c974c8da8d3e87))

## [0.2.4](https://github.com/denkwerk/favicon-generator/compare/v0.2.3...v0.2.4) (2026-09-24)

### Bug Fixes

* **npm:** resolve the types with moduleResolution node ([4c6c3e5](https://github.com/denkwerk/favicon-generator/commit/4c6c3e50d5c1f13983b00b96b52750675cbc3e95))

### Documentation

* describe the pinned tool versions and have Dependabot propose Rust and Actions updates ([949904f](https://github.com/denkwerk/favicon-generator/commit/949904fea82825aab585e65f15b78bd5dbe7ea9c))

## [0.2.3](https://github.com/denkwerk/favicon-generator/compare/v0.2.2...v0.2.3) (2026-09-24)

### Bug Fixes

* **npm:** add keywords, homepage, bugs and author to the packages ([5d0653d](https://github.com/denkwerk/favicon-generator/commit/5d0653d4f0e3156fd2650dc3c5063a2b46b2f9fd))
* **npm:** list the Nuxt module and bundler plugins in every package's keywords ([16df325](https://github.com/denkwerk/favicon-generator/commit/16df325a26407078a21a981d4da2cdfb82a17ed5))

## [0.2.2](https://github.com/denkwerk/favicon-generator/compare/v0.2.1...v0.2.2) (2026-09-23)

### Features

* **core:** export mergeConfig to layer options over a config file ([17adbc4](https://github.com/denkwerk/favicon-generator/commit/17adbc451d7c37b64a3f7fe6b7e28a8aad4a3e96))
* **unplugin:** add a Vite, Rollup, Rolldown, webpack and Rspack plugin ([ae6d66f](https://github.com/denkwerk/favicon-generator/commit/ae6d66f1bf6ec3d798533692e60a1da7fddef901))

### Bug Fixes

* **unplugin:** emit webpack and Rspack assets in processAssets ([2fc6633](https://github.com/denkwerk/favicon-generator/commit/2fc66332891a7423a02cc5a209b582636a643f6d))

### Documentation

* add Vite config file and tsdown library examples ([5a4edd5](https://github.com/denkwerk/favicon-generator/commit/5a4edd5323cba9447e1f6e5d9b17b2230cac6798))
* move the contributing guide to CONTRIBUTING.md ([d4cafd4](https://github.com/denkwerk/favicon-generator/commit/d4cafd4e3333206e3b77e1483d1c8b960a8d1bf2))

## [0.2.1](https://github.com/denkwerk/favicon-generator/compare/v0.2.0...v0.2.1) (2026-09-23)

### Documentation

* **readme:** describe the extras as additions ([6faa442](https://github.com/denkwerk/favicon-generator/commit/6faa4428de7c77abea7f6d8cb938c5a9f5301950))

## [0.2.0](https://github.com/denkwerk/favicon-generator/compare/v0.1.3...v0.2.0) (2026-09-23)

### ⚠ BREAKING CHANGES

* the default output is the minimal set; add `legacy: true`
for the old sizes, and configure `manifest`/`windows` for those files. The
app options moved into groups and the old names are rejected with a hint:
appName/appShortName/appDescription/startUrl/scope/display become
manifest.name/shortName/description/startUrl/scope/display,
manifestCrossorigin becomes manifest.crossorigin, backgroundColor becomes
manifest.backgroundColor or appleTouchIcon.background, tileColor becomes
windows.tileColor, iconPurpose becomes manifest.maskable, --app-name
becomes --name, and snippets default to ['html'].

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

### Features

* generate only what the image provides by default, with opt-in option groups ([401c7ef](https://github.com/denkwerk/favicon-generator/commit/401c7ef7614e39de765e8f3d414274b159cb5d3f))

### Documentation

* **readme:** minimal quick start, detailed configuration per option group ([a7693cc](https://github.com/denkwerk/favicon-generator/commit/a7693cce6b0573100480f38b3e500c6dd7c9a344))

## [0.1.3](https://github.com/denkwerk/favicon-generator/compare/v0.1.2...v0.1.3) (2026-09-23)

### Features

* **cli:** add -i/--input and -o/--output flags ([2de4868](https://github.com/denkwerk/favicon-generator/commit/2de48684b35a20dc7aada73cebedc2a779dffd04))

### Documentation

* **readme:** show flags and config file side by side in the quick start ([b24d64b](https://github.com/denkwerk/favicon-generator/commit/b24d64b239dd396c89b54648b2f8a900ec8ad172))

## [0.1.2](https://github.com/denkwerk/favicon-generator/compare/v0.1.1...v0.1.2) (2026-09-23)

### Features

* update logo ([756d8d9](https://github.com/denkwerk/favicon-generator/commit/756d8d9ec2f43b353a2ea899d1c6876385fe4a02))

## [0.1.1](https://github.com/denkwerk/favicon-generator/compare/v0.1.0...v0.1.1) (2026-09-23)

### Documentation

* **readme:** restructure the READMEs and showcase config formats and output ([7edab29](https://github.com/denkwerk/favicon-generator/commit/7edab296b2a7378ac8ebbd860617393262109128))

## [0.1.0](https://github.com/denkwerk/favicon-generator/compare/v0.0.0...v0.1.0) (2026-09-23)

### ⚠ BREAKING CHANGES

* **nuxt:** rename the config key to `favicon`

### Features

* add favicon ([845db1c](https://github.com/denkwerk/favicon-generator/commit/845db1c02c12f15f1de92b78252d8da2fbd878fd))
* add initial rust idea ([65f24e8](https://github.com/denkwerk/favicon-generator/commit/65f24e831f20bbfb8bacd77a9ba46e3285f8999b))
* add Nuxt module and examples/nuxt, rename demo to examples ([df37168](https://github.com/denkwerk/favicon-generator/commit/df37168e54d6a8a5e36ff2f62962a35d3cd289d2))
* favicon.config.* support in the Nuxt module, split CLI/TS examples ([e4b1e2a](https://github.com/denkwerk/favicon-generator/commit/e4b1e2a26d5528dfbe2bab0342f8780c3e3a29ef))
* **npm:** ship all platform binaries in @denkwerk/favicon-generator ([89285db](https://github.com/denkwerk/favicon-generator/commit/89285db605cc51d79dc1c5ef27ddf6a2f9bf7fcd))
* **nuxt:** rename the config key to `favicon` ([67babd2](https://github.com/denkwerk/favicon-generator/commit/67babd27b900531db424b62420a4cff6387e4d09))
* turborepo monorepo with npm package, config files and release CI ([e840df1](https://github.com/denkwerk/favicon-generator/commit/e840df1fa7ec92a5b536933d144cd9201acc875e))
