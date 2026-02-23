### 2026-02-23

*BREAKING CHANGES*
+ `log_scroll_forward`, `log_scroll_back` renamed to `scroll_forward`, `scroll_back`
+ Additional KeyMap entry, `inspect` defaults to `i`, enables Inspect mode
+ Docker Host priorities reordered, *should* now be, from high to low order,  `--host` cli argument, `config.toml` `host` value, `DOCKER_HOST` env, [Docker library](https://github.com/fussybeaver/bollard) default setting.
+ `config.toml` `host` value is now  commented out by default - this should help with invalid Docker connection errors and enable easy Podman support

### Chores
+ dependencies updated, [4658a8de264698b0c8092e1227f0683527219a0b], [8b5899ca238bcbff32519b376b920cd7b7509809], [bebb687c59f3b408e69b23d2e68fa69f006a3231]
+ GitHub workflow updated, [a0aa7918241ee8f702d6472c620287aa4be7d56c]

### Features
+ Network chart, closes #79, [99fcb8fedf01599ec346b65d435d4c301a7a8851]
+ Inspect mode & help panel redesign, [ae7f3f4a9472b451c37c0ab97b1756b41a3529f5]
+ set rust-version in Cargo.toml, closes #77, [0763a1024f44d98b8d9d65f57995da538e40963c]

### Fixes
+ Enable quit on Docker connect error screen, [5f942eb2e963660bd7fe9d80fa7ba8a83754803a]

### Refactors
+ dead code removed, [3e31a2a6bc02d6ef75bd6cbc18568e82e60e1ee3]
+ docker data spawns, [cd943f67e465fff9726b40570a089301a4a8f534]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
