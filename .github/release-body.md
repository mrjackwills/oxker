### 2025-06-18

### Chores
+ .devcontainer updated, [324f8268278081504d5357f2ed89b78ca2c25d04]
+ dependencies updated, [0ace9dd662144a589341779a64d7fcd8de7d9978], [a636007547280b3b3db69374601dbece4bc21eef]
+ Rust 1.87.0 linting, [395b1aa7e997a528e4f21e66f5f859001c1c3ec1], [67e5888e008cfd504c10e47f678f9351c838be99]

### Docs
+ example config files updated, [63ab7de72897de460f31181c5a42befbee2f91d3], [8fb5ac4a945b75f3fcd118c53be1202ccbc43c59]
+ README.md updated, link to directories crate, closes #65, [c2bfe3296563daf4b7f077469f3eeff6895720b0]

### Features
+ log panel size configurable, closes #50, use the `-` or `=` keys to change the height of the logs panel, or `\` to toggle visibility. Automatically hide the logs panel using a new config item `show_logs`, see `example_config/*` files for more details, [6edf99e0846bb4134d8ee5b646065b8cda8074d7]
+ build release binaries for aarch64-apple-darwin, closes #62, personally untested on MacOS - but others suggest it works as expected, [e7114d2f5e0ed8935943be64726fc2d90464a777], [2e8500902a515a246f9d9a503b4350849d634978]

### Fixes
+ merge args color/raw fix, [d198398795698a21d81d3fd20231c482cc346ab5]

### Refactors
+ reduce cloning of the logs text items, can expect 40-50% reduction in CPU and memory usage in certain common situations, [ecefa302b9ef5320ad4cce0b606aca70a7b459e2]
+ dead code removed, [b40b6b197e4e5fbdab083bc918d1a5d2750597f3]

### Tests
+ add more whole layout tests, [4b81c6caaf12028d7527c3f23cd2de6d1503e223]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
