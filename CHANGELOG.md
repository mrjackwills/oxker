### Chores
+ create_release.sh updated, [d4af754ad245540db60177f7b202b3c64519c961]
+ dependencies updated, [03599b46657d38d0c9f25c2ccfd9510f2b98dd84], [aef0c9503e7045a256856aa887d8c8d7722b9936], [f0771eab5d07d141fe7a8997db650f0f65ffe0a7], [1596de8681ad6c0a7832eb922dd2dc36ab30eb41]
+ GitHub workflow updated, [66dae5e61ea294ac8ce134a6c32b27c04166b6eb]

### Features
+ *BREAKING CHANGE* - `scroll_down_many` & `scroll_up_many` removed, `scroll_down_one` `scroll_up_one` renamed `scroll_down`, `scroll_up`, see [example_config](https://github.com/mrjackwills/oxker/tree/main/example_config), [52a04ec1d0b9e4877e304f60a857ebc00f88b4fd]
+ log search feature, closes #72. Use `#` button, remappable via `log_search_mode`, to enter log search mode. Case-sensitive by default, editable in `config.toml` with `log_search_case_sensitive` entry. Customise colours via `[colors.log_search]` entries, again see see [example_config](https://github.com/mrjackwills/oxker/tree/main/example_config), [96d9469623a7c90b79aa8d82abf587290343ad37], [a2316a9cac270790920a1ebd1be6532d51aba77c]
+ `term` renamed `filter term`, tests updated, [487c3faf96f4c197c8b82644c02466ea40626a5e]

My 32-bit armhf armv6 hardware no longer seems to be able to run Docker. Future `oxker` releases won't be tested on real hardware but will continue to be built and published for armv6.

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.11.1'>v0.11.1</a>
### 2025-08-21

### Reverts
+ GitHub workflow reverted, [d1b69858](https://github.com/mrjackwills/oxker/commit/d1b69858c622636afbc46db7d37cf91e58d61212)

see [v0.11.0 release notes](https://github.com/mrjackwills/oxker/releases/tag/v0.11.0) for more information about v0.11 release

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.11.0'>v0.11.0</a>
### 2025-08-21

### Chores
+ Dependencies updated, [ced885e0](https://github.com/mrjackwills/oxker/commit/ced885e0128b6d5d3a3c7cb97d7e53bc2da64893), [f9b40ea0](https://github.com/mrjackwills/oxker/commit/f9b40ea03d0e70e235c28646ff3f9ebb468a904d)
+ Rust 1.89 linting, [79d19cee](https://github.com/mrjackwills/oxker/commit/79d19ceeb81ae60bc5562683e405d6e74e6f2578)
+ GitHub workflow updated, [08384200](https://github.com/mrjackwills/oxker/commit/08384200558fa1b9d378ea62ea832708caebaa91), [6573af1e](https://github.com/mrjackwills/oxker/commit/6573af1ed7d382a81c1305397e904066bb8395a8)

### Features
+ Horizontally scroll across logs. By default use `←` & `→` keys to traverse horizontally across the lines when logs panel selected. Updated `config.toml` with `log_scroll_forward` and `log_scroll_back`  [c190f020](https://github.com/mrjackwills/oxker/commit/c190f0206cc55b8e45b8373f9be954e828c18b3b), [8939ac03](https://github.com/mrjackwills/oxker/commit/8939ac0345326633e794cc10a981a1f3c5c07549)
+ Force clear screen & redraw of UI. By default uses `f` key, `config.toml` updated with `force_redraw`  [50edbc0c](https://github.com/mrjackwills/oxker/commit/50edbc0cc09db864835fe81a03cba8eadafe548b)
+ Increase scroll speed using the `ctrl` key in conjunction with a scroll key, `config.toml` updated with `scroll_modifier`. The next release will remove `scroll_down_many` & `scroll_down_up` keys, [c5bbffdb](https://github.com/mrjackwills/oxker/commit/c5bbffdb5f9e800951e4060aa6aee8e00db589aa)

### Refactors
+ remove macos cfg none-const functions, Zigbuild now uses Rust 1.87.0, [eb686e2c](https://github.com/mrjackwills/oxker/commit/eb686e2c952e04da74b3e12c0bfa015ec4615e1d)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.10.5'>v0.10.5</a>
### 2025-06-19

### Reverts
+ Bollard update rolled back, closes [#66](https://github.com/mrjackwills/oxker/issues/66), [aac9c6b5](https://github.com/mrjackwills/oxker/commit/aac9c6b598ce6c23b14f5a8b0116e662b18074d2)

 # <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.10.4'>v0.10.4</a>
### 2025-06-18

### Chores
+ .devcontainer updated, [324f8268](https://github.com/mrjackwills/oxker/commit/324f8268278081504d5357f2ed89b78ca2c25d04)
+ dependencies updated, [0ace9dd6](https://github.com/mrjackwills/oxker/commit/0ace9dd662144a589341779a64d7fcd8de7d9978), [a6360075](https://github.com/mrjackwills/oxker/commit/a636007547280b3b3db69374601dbece4bc21eef)
+ Rust 1.87.0 linting, [395b1aa7](https://github.com/mrjackwills/oxker/commit/395b1aa7e997a528e4f21e66f5f859001c1c3ec1), [67e5888e](https://github.com/mrjackwills/oxker/commit/67e5888e008cfd504c10e47f678f9351c838be99)
back
### Docs
+ example config files updated, [63ab7de7](https://github.com/mrjackwills/oxker/commit/63ab7de72897de460f31181c5a42befbee2f91d3), [8fb5ac4a](https://github.com/mrjackwills/oxker/commit/8fb5ac4a945b75f3fcd118c53be1202ccbc43c59)
+ README.md updated, link to directories crate, closes [#65](https://github.com/mrjackwills/oxker/issues/65), [c2bfe329](https://github.com/mrjackwills/oxker/commit/c2bfe3296563daf4b7f077469f3eeff6895720b0)

### Features
+ log panel size configurable, closes [#50](https://github.com/mrjackwills/oxker/issues/50), use the `-` or `=` keys to change the height of the logs panel, or `\` to toggle visibility. Automatically hide the logs panel using a new config item `show_logs`, see `example_config/*` files for more details, [6edf99e0](https://github.com/mrjackwills/oxker/commit/6edf99e0846bb4134d8ee5b646065b8cda8074d7)
+ build release binaries for aarch64-apple-darwin, closes [#62](https://github.com/mrjackwills/oxker/issues/62), personally untested on MacOS - but others suggest it works as expected, [e7114d2f](https://github.com/mrjackwills/oxker/commit/e7114d2f5e0ed8935943be64726fc2d90464a777), [2e850090](https://github.com/mrjackwills/oxker/commit/2e8500902a515a246f9d9a503b4350849d634978)

### Fixes
+ merge args color/raw fix, [d1983987](https://github.com/mrjackwills/oxker/commit/d198398795698a21d81d3fd20231c482cc346ab5)

### Refactors
+ reduce cloning of the logs text items, can expect 40-50% reduction in CPU and memory usage in certain common situations, [ecefa302](https://github.com/mrjackwills/oxker/commit/ecefa302b9ef5320ad4cce0b606aca70a7b459e2)
+ dead code removed, [b40b6b19](https://github.com/mrjackwills/oxker/commit/b40b6b197e4e5fbdab083bc918d1a5d2750597f3)

### Tests
+ add more whole layout tests, [4b81c6ca](https://github.com/mrjackwills/oxker/commit/4b81c6caaf12028d7527c3f23cd2de6d1503e223)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.10.3'>v0.10.3</a>
### 2025-04-22

### Chores
+ dependencies updated, [bbfd2462](https://github.com/mrjackwills/oxker/commit/bbfd2462a1f45008587b488e8c6049ee76da72f2)

### Tests
+ fix tests for MacOS, closes [#61](https://github.com/mrjackwills/oxker/issues/61), [cfc2decd](https://github.com/mrjackwills/oxker/commit/cfc2decd8d237f1ac3f0bdb2b3d5581684064448)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.10.2'>v0.10.2</a>
### 2025-04-19

### Chores
+ dependencies updated, [1345ecb1](https://github.com/mrjackwills/oxker/commit/1345ecb1a2b17ad3d288f2de2058c0777b84f93b)

### Tests
+ use a fixed version String, `0.00.000`, in tests, [230174b3](https://github.com/mrjackwills/oxker/commit/230174b3c327c3f217cdcf8fce07d5d9ddea1033)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.10.1'>v0.10.1</a>
### 2025-04-18

### Chores
+ dependencies updated, [8f959c54](https://github.com/mrjackwills/oxker/commit/8f959c5408995527485e817514c3f4a10bca31bd), [69d1801e](https://github.com/mrjackwills/oxker/commit/69d1801ea1e71e7d84c00fb2142ba27577e3bb73), [188490e1](https://github.com/mrjackwills/oxker/commit/188490e13fd1255eecb305ac0a99a7a1913e0294)
+ Rust 1.86.0 linting, [9acf6033](https://github.com/mrjackwills/oxker/commit/9acf60334c5224faa9ee4ecf7030b2e9b13b7d67)

### Docs
+ comment typo, [723b220c](https://github.com/mrjackwills/oxker/commit/723b220c6aa6393b8eebd84a6ddba69f35dc18b8)

### Fixes
+ github workflow update, [997eebca](https://github.com/mrjackwills/oxker/commit/997eebca20a2883dcfcb35009dd4d7b0d438dec6)
+ config merging, [a468827f](https://github.com/mrjackwills/oxker/commit/a468827f02c5243b9bd4e0183fed16854ae6c851)

### Refactors
+ rename ChartType to ChartVariant, [bca67116](https://github.com/mrjackwills/oxker/commit/bca67116f3b71451156a39a7b0957568b26fa183), [d0caa927](https://github.com/mrjackwills/oxker/commit/d0caa9271b6f92f52ccfe3dec69708efe54e5170)
+ rename FileType to FileFormat, [848f64d0](https://github.com/mrjackwills/oxker/commit/848f64d0da429e547a2f6c8de62e4da5f5c9a187)

### Tests
+ Use insta, closes [#57](https://github.com/mrjackwills/oxker/issues/57), [9362d7b4](https://github.com/mrjackwills/oxker/commit/9362d7b481ea22eab6f902dc7f3c10150c7ddf22)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.10.0'>v0.10.0</a>
### 2025-02-23

### Chores
+ dependencies updated, [e5f355a1](https://github.com/mrjackwills/oxker/commit/e5f355a1928f78abdb64e4c5617d6fac06340016), [4539d8ad](https://github.com/mrjackwills/oxker/commit/4539d8ad0705b46d7c89c51c7be482b696d26e5f), [6aee6181](https://github.com/mrjackwills/oxker/commit/6aee6181136235a1a4f79af9b9748c1801be8bf8), [64d1bdf2](https://github.com/mrjackwills/oxker/commit/64d1bdf2bf88407e02f0eded1e03fcfc5ee2d8e3)
+ .devcontainer dependencies updated, [5c8e76e7](https://github.com/mrjackwills/oxker/commit/5c8e76e7bb4d7aab8543c9be09fdbc4ffa446b10)
+ example docker-compose.yml updated, [2354b0b9](https://github.com/mrjackwills/oxker/commit/2354b0b9be1ab3795a421512594b2650b9cbdd74)
+ Rust 1.84 linting, [3065265e](https://github.com/mrjackwills/oxker/commit/3065265e26c30d78ba738cfe731d3901ec1948d0)

### Features
+ Config file introduced, including customizing color scheme of application, closes [#47](https://github.com/mrjackwills/oxker/issues/47), [f4d54e1b](https://github.com/mrjackwills/oxker/commit/f4d54e1ba8ea1516394aef19511a63e6271f27bf)
+ Enable log timestamps to be set to any given timezone, plus custom timestamp format via the config file, closes [#56](https://github.com/mrjackwills/oxker/issues/56), [7a5e7a25873d2c270e5808730721ebb5427a051]
+ update Rust edition to 2024, [7e4a960b](https://github.com/mrjackwills/oxker/commit/7e4a960b888f1dab524d6045504162cea1171d20)

### Fixes
+ Only draw screen if data or layout has changed, drastically reduces CPU usage, [bfc295c5](https://github.com/mrjackwills/oxker/commit/bfc295c50e982886ccaa5e60b57f10d3690b3f09)
 
 # <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.9.0'>v0.9.0</a>
### 2024-12-05

### Chores
+ dependencies updated, [b7871357](https://github.com/mrjackwills/oxker/commit/b78713579c4706d605e5b35fcd832610a0152294), [c6200e8f](https://github.com/mrjackwills/oxker/commit/c6200e8f77f8bb1f0152cb9374029d15cc45df9d)
+ Rust 1.83 linting, [751d997a](https://github.com/mrjackwills/oxker/commit/751d997a3dac823e144ae62e6c1455676e50ddb8)

### Features
+ `--no-stderr` cli arg, removes Standard error output from logs, closes [#52](https://github.com/mrjackwills/oxker/issues/52), [c739637b](https://github.com/mrjackwills/oxker/commit/c739637b91c8fa742a69f4d888678d7b3964678c)
+ ContainerPorts use ipaddr, [1b26997d](https://github.com/mrjackwills/oxker/commit/1b26997d25f748e0d452f41fe41791533046ecdf)

### Fixes
+ update containerised Dockerfile, [0c6f5322](https://github.com/mrjackwills/oxker/commit/0c6f53228f01196e352c2069383ba1e7a10950a8)
+ calculate_usage overflow, [5106a01f](https://github.com/mrjackwills/oxker/commit/5106a01f3dcb87ce5a8f1fb7bf49dc6b3c25d03e)
+ DockerData spawns insertion error, [d4906d33](https://github.com/mrjackwills/oxker/commit/d4906d33c26b75d92e7d80040c488faa90a257c6)

### Refactors
+ speed up docker logs init process, [8b9fe424](https://github.com/mrjackwills/oxker/commit/8b9fe4246865441704ae12dff0938868a4fe6f81)
+ remove docker sleep, [f1562d10](https://github.com/mrjackwills/oxker/commit/f1562d1084336fe5be39894c93cb49107f0a4a6d)
+ dead code removed, [5ee48d57](https://github.com/mrjackwills/oxker/commit/5ee48d5708fa6de0206c021db0bb611196e66fba), [ba6a9524](https://github.com/mrjackwills/oxker/commit/ba6a95241389f99d504ee4bf3e87e19006f12e49), [f0b11456](https://github.com/mrjackwills/oxker/commit/f0b1145651625ad4e577d79baaf902d4d3bc0579)
+ input_handler, [7f423834](https://github.com/mrjackwills/oxker/commit/7f4238349525c01ae9fb8b1f6c0946e5364dd55e)
+ statefulList get_state_title, [2d540b0e](https://github.com/mrjackwills/oxker/commit/2d540b0e2210cc04d73035ec59211ffc739174f6)
+ statefulList next/previous, [7bb2bef2](https://github.com/mrjackwills/oxker/commit/7bb2bef28d90ebc58da86a0365a1904a0c32dffe)
+ help_box closure fn, [2860426d](https://github.com/mrjackwills/oxker/commit/2860426d57a4458fcee49a2fd20e8e7bb9e71fb5)
+ use check_sub for sleep calculations, [fe3696e5](https://github.com/mrjackwills/oxker/commit/fe3696e5576739d8b033d9e748b5ea696c4b4e4f)
+ rename scheduler to heartbeat, [68a6551e](https://github.com/mrjackwills/oxker/commit/68a6551ed038a36330b2f098112829465a1c3c7a)
+ remove unnecessary is_running load, [76ccf7c0](https://github.com/mrjackwills/oxker/commit/76ccf7c00691f815c3ab0bede838c99252ba84f0)
+ execute_command(), [2a834d6c](https://github.com/mrjackwills/oxker/commit/2a834d6c2fa4a15124d24ddbd12f667829e148ad)
+ Remove numerous clones(), [e5927f78](https://github.com/mrjackwills/oxker/commit/e5927f781a7e9517b9fa00a2d1a835d2774a9d26)
+ remove app_data param from generate_lock(), [1a8dab65](https://github.com/mrjackwills/oxker/commit/1a8dab654a1fdbf351a72dc54fe3d1943355bba6)
+ combine get_filter methods, [356ea554](https://github.com/mrjackwills/oxker/commit/356ea5549bb4877e9893fe0e1053e73c5a62e806)
+ FrameData refactors, [57781701](https://github.com/mrjackwills/oxker/commit/57781701ff14c553dfbafb965ee8a33ab44dd36f), [6e2f82db](https://github.com/mrjackwills/oxker/commit/6e2f82db81caaa98ce4781fa15928eb9e246ace6)
+ update_container_stat combine is_alive(), [55cc7467](https://github.com/mrjackwills/oxker/commit/55cc746736f6863aedc5ad838744a983796244d8)
+ remove `input_poll_rate` from `Ui`, instead use const `POLL_RATE`, [69f6c96b](https://github.com/mrjackwills/oxker/commit/69f6c96b700b9fde5578ae204992a67986d456ab)
+ pass `&FrameDate` into `draw_frame()`, [35aec506](https://github.com/mrjackwills/oxker/commit/35aec5060fdbe606267be26656b4aeee43d50c02)
+ dead code removed, [caf23be4](https://github.com/mrjackwills/oxker/commit/caf23be4a7faff99aaca80b081a02e4e0a372009)
+ input_handler, [9c4f8910](https://github.com/mrjackwills/oxker/commit/9c4f8910381b90b563da12eaba4b79cb60c40129)
+ draw_block, [de76bc22](https://github.com/mrjackwills/oxker/commit/de76bc22936b124dcb9646f302f6cc14691dbb63)

### Tests
+ fix logs tests, [9b22f5da](https://github.com/mrjackwills/oxker/commit/9b22f5da18e4bf92766a68a7f4cd61ad72724cfd)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.8.0'>v0.8.0</a>
### 2024-10-22

### Chores
+ dependencies updated, [ea877d23](https://github.com/mrjackwills/oxker/commit/ea877d23711b98ffd1108a74206d93d43482d44d), [af609c0d](https://github.com/mrjackwills/oxker/commit/af609c0dbf0caab4a073f822166de34999afb41b)
+ .devcontainer updated, [a9844436](https://github.com/mrjackwills/oxker/commit/a9844436d003b84a3e9d8b600ea029b232566f3a)
+ create_release.sh updated, [c4943370](https://github.com/mrjackwills/oxker/commit/c4943370f4a67f6c01c75a8a7f825912427666a2), [1389d8ad](https://github.com/mrjackwills/oxker/commit/1389d8adbba75fef480eb1de09337eb7beb10ba3)

### Features
+ Add Stderr output to logs, thanks [vincentmasse](https://github.com/vincentmasse), closes [#48](https://github.com/mrjackwills/oxker/issues/48), merges [#49](https://github.com/mrjackwills/oxker/pull/49), [b95c9311](https://github.com/mrjackwills/oxker/commit/b95c9311416cd0dbcfa5de90c23f3065bc2d6b17), [9936ad45](https://github.com/mrjackwills/oxker/commit/9936ad45e186ee431aade920674a2dc283937355), [289ede3f](https://github.com/mrjackwills/oxker/commit/289ede3f2531feeec56094a76bf34f4c69431bbe)

### Refactors
+ Rust 1.82 linting, [c058c5a3](https://github.com/mrjackwills/oxker/commit/c058c5a301cfd4e8d7a0079c4c3f8fdeae2803e5)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.7.2'>v0.7.2</a>
### 2024-09-07

### Reverts
+ Expect lint was causing issues with crates/docker builds, revert until fix is found, [578ed9f0](https://github.com/mrjackwills/oxker/commit/578ed9f085df0d97c451c06dab4ecbfccd894c52)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.7.1'>v0.7.1</a>
### 2024-09-07

### Chores
+ dependencies updated, [d6238587](https://github.com/mrjackwills/oxker/commit/d6238587ffc536f1ea93a47dd4d2ee69c36f35e3), [a564ef80](https://github.com/mrjackwills/oxker/commit/a564ef80318adbde9f188dd2cf38626a98793c75), [1d82ff13](https://github.com/mrjackwills/oxker/commit/1d82ff1368f7b43e6df8478d5e7b8d682320010a), [99f05f2e](https://github.com/mrjackwills/oxker/commit/99f05f2e5b511d039804159c92ade6c77ff360b7)
+ Rust 1.81.0 linting, [372f759c](https://github.com/mrjackwills/oxker/commit/372f759ca467e47c373a086c6a247c150b87d4bc)
+ .devcontainer updated, [5d77f1e0](https://github.com/mrjackwills/oxker/commit/5d77f1e02a428b4e5ee1bf5466055ad17f6e05af)

### Docs
+ CHANGELOG.md duplicate removed, [16ecc5a5](https://github.com/mrjackwills/oxker/commit/16ecc5a51f7defcc9dd4f4c6e34fa5bbfc7fea78)
+ Readme raspberry pi fix, [baf68783](https://github.com/mrjackwills/oxker/commit/baf68783929e5d6ac111a39dc62388cd24133da6)
+ Add installation guide to README for macOS installation via `brew install oxker`, thanks [miketheman](https://github.com/miketheman), [59817311](https://github.com/mrjackwills/oxker/commit/59817311baea628d2691765ff9387e055dce3307), [895ec620](https://github.com/mrjackwills/oxker/commit/895ec6204cc8220be64be7622df6edb6ef6d795b)

### Refactors
+ switch lints from `allow(x)` to `expect(x)`, [2a0ab6d8](https://github.com/mrjackwills/oxker/commit/2a0ab6d81ce4062de053a92b85f8b25ea23412b6)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.7.0'>v0.7.0</a>
### 2024-08-01

### Chores
+ .devcontainer extensions updated, [0288cbc8](https://github.com/mrjackwills/oxker/commit/0288cbc8146cde1dd40ceaec9550198b635bb8f5)
+ dependencies updated, [1df4f78d](https://github.com/mrjackwills/oxker/commit/1df4f78dc41013c33d901925933b1ccb29ad4bc8), [5ae253b8](https://github.com/mrjackwills/oxker/commit/5ae253b8734ba0495e4e8149b17d5228b3d86f8d), [7a517db9](https://github.com/mrjackwills/oxker/commit/7a517db9f7c14c35e56ff70cf76ffb608fd30e17), [9c291cd9](https://github.com/mrjackwills/oxker/commit/9c291cd9c81b6d9a02085878588ed3b845fd0046), [0e90f4eb](https://github.com/mrjackwills/oxker/commit/0e90f4eb55ac5fb5d45e7d212c3686027dd3913e), [fe71cbfb](https://github.com/mrjackwills/oxker/commit/fe71cbfb00f166b7c02a6e28e64650ed1b47d15d)
+ docker-compose alpine version bump, [51ceab3e](https://github.com/mrjackwills/oxker/commit/51ceab3ebdb09356cd401d2f268840239255126f)
+ Rust 1.80 linting, [93e1279b](https://github.com/mrjackwills/oxker/commit/93e1279b1fc77019442a385e2e36be2fe438e828)
+ create_release v0.5.6, [f408acfe](https://github.com/mrjackwills/oxker/commit/f408acfe9a9f5a976735b8a8a51500fd7b865daf)

### Docs
+ screenshot updated, [6975ebe7](https://github.com/mrjackwills/oxker/commit/6975ebe70f7058229c232e4a56b090f55247d2a2)

### Features
+ left align all text, [e0d421c4](https://github.com/mrjackwills/oxker/commit/e0d421c4918a17c9e0e21fd214edb99d71281c9d)
+ place image name in logs panel title, [12f24357](https://github.com/mrjackwills/oxker/commit/12f24357a68abe871f44d871d95b6e2ef062181e)
+ distinguish between unhealthy & healthy running containers, closes [#43](https://github.com/mrjackwills/oxker/issues/43), [de876818](https://github.com/mrjackwills/oxker/commit/de8768181631c6d961ce0e4dacb50c2ed02abc36)
+ filter containers, use `F1` or `/` to enter filter mode, closes [#37](https://github.com/mrjackwills/oxker/issues/37), thanks to [MohammadShabaniSBU](https://github.com/MohammadShabaniSBU) for the original PR, [d5d8a0db](https://github.com/mrjackwills/oxker/commit/d5d8a0dbc5437ff3b17f34b9dbb9589bb56b4a3e), [7ee1f06f](https://github.com/mrjackwills/oxker/commit/7ee1f06f804683e3395953a02138d4e9da115ea9)

### Fixes
+ log_sanitizer `raw()` & `remove_ansi()` now functioning as intended, [0dc98dfc](https://github.com/mrjackwills/oxker/commit/0dc98dfc8113869b81be9d697ca77418c919e4bf)
+ Dockerfile command use uppercase, [068e4025](https://github.com/mrjackwills/oxker/commit/068e4025a5d6049a9a6951a0480a6bdef7379f88)
+ heading section help margin, [0e927aae](https://github.com/mrjackwills/oxker/commit/0e927aae178c1d8f60561b93607a26d45a1d9331)
+ install.sh use curl, [197a031b](https://github.com/mrjackwills/oxker/commit/197a031b8cf356f49f08e04472d0d1c489699415)

### Tests
+ fix layout tests with new left alignment, [dfced564](https://github.com/mrjackwills/oxker/commit/dfced564278eafdbb8a5b95badbae3a7c4bf87b3)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.6.4'>v0.6.4</a>
### 2024-05-25

### Chores
+ Dependencies updated, [51fdd26b](https://github.com/mrjackwills/oxker/commit/51fdd26be5b3166bcff5c26ece6d6ec0d893381e), [c1be658b](https://github.com/mrjackwills/oxker/commit/c1be658b8cc4786a9a7f2e0a88568019b3995c14)

### Docs
+ exec mode "not available on Windows", in both README.md and help panel, [df449a85](https://github.com/mrjackwills/oxker/commit/df449a85376bbeec87215952d6a9196721f7132e)

### Fixes
+ closes [#36](https://github.com/mrjackwills/oxker/issues/36) Double key strokes on Windows, [9b7d575a](https://github.com/mrjackwills/oxker/commit/9b7d575a76398cbe19e17f6494baf802dbb512b9)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.6.3'>v0.6.3</a>
### 2024-05-07

### Chores
+ Dependencies updated, [07e293ac](https://github.com/mrjackwills/oxker/commit/07e293ac2ce2e7deb5735154fcdb24ef83a19b67), [27d72c54](https://github.com/mrjackwills/oxker/commit/27d72c547e738f6816cd4b353ac881e454a0be70)

### Features
+ Allow closing dialogs with `Escape`, thanks [JCQuintas](https://github.com/JCQuintas), [0e4c3cea](https://github.com/mrjackwills/oxker/commit/0e4c3ceab933458d40b54d5fcff7e6cf7a3ab315)

### Fixes
+ correct header display when terminal width changes, [4628803b](https://github.com/mrjackwills/oxker/commit/4628803b2b9fe63522d033b192763ed6ff5b57dd)

### Refactors
+ use tokio CancellationToken, [0631a73e](https://github.com/mrjackwills/oxker/commit/0631a73ec27530f8fcc88988a0a02ca75e32c5ba)
+ impl AsyncTTY, [bf33776e](https://github.com/mrjackwills/oxker/commit/bf33776e9a61684032a80d22d995ba7e0446620e)

### Tests
+ reduced header section test, [aa094740](https://github.com/mrjackwills/oxker/commit/aa0947405393db2c306e86986183514cbc0f5a75)
+ test_draw_blocks_help() with add esc text, [ff839af4](https://github.com/mrjackwills/oxker/commit/ff839af4ef68193149d6456e70fee189228c4a44)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.6.2'>v0.6.2</a>
### 2024-03-31

### Chores
+ .devcontainer updated, [82a7f84e](https://github.com/mrjackwills/oxker/commit/82a7f84ed94a9bad6cc5fe078bfadbda65c8ea8f)
+ Rust 1.77 linting, [dfd4948d](https://github.com/mrjackwills/oxker/commit/dfd4948d9c43cfbffb091c303d3cb2a05522047a)
+ platform.sh formatted, [7953e68f](https://github.com/mrjackwills/oxker/commit/7953e68f3067ac3c9d4fe67e9f934c25036c0833)
+ dependencies updated, [8e6c3ca6](https://github.com/mrjackwills/oxker/commit/8e6c3ca6d83768f043923ccf6836397beaaae639)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.6.1'>v0.6.1</a>
### 2024-02-14

### Chores
+ create_release v0.5.5, [616338b7](https://github.com/mrjackwills/oxker/commit/616338b7107036e968f51c3ff80739f9ffb40fbd)
+ update dependencies, [10180d2e](https://github.com/mrjackwills/oxker/commit/10180d2e0817c00a198e27f7d71080c502639a6b)
+ update to ratatui v0.26.0, [d33dce3e](https://github.com/mrjackwills/oxker/commit/d33dce3eec4c19cc3c3668dab77f7d25d6970c3c)
+ GitHub workflow dependency bump, [0314eac9](https://github.com/mrjackwills/oxker/commit/0314eac9df6cf9fea1943dcd06bd6a0b27131c16)

### Docs
+ screenshot updated, [fe5ec4f5](https://github.com/mrjackwills/oxker/commit/fe5ec4f5dd25f11817be37f3f1867a6a2b0afc42)

### Fixes
+ ports all listed in white, [d3b23585](https://github.com/mrjackwills/oxker/commit/d3b23585b38045eb3bc827367eca90eb7f7a7dd5)
+ use long container name in delete popup, [6202b7bb](https://github.com/mrjackwills/oxker/commit/6202b7bbfdfb04a94959b5143dac3f1aa59cd336)
+ memory display, closes [#33](https://github.com/mrjackwills/oxker/issues/33), [a182d40a](https://github.com/mrjackwills/oxker/commit/a182d40a7463164ef5dcac379d1a1768d77209d2)

### Refactors
+ use &[T] instead of &Vec<T>, [76cd08ab](https://github.com/mrjackwills/oxker/commit/76cd08ab2f98687a866a6bbb4fa93bbdedaa7699), [1f62bb50](https://github.com/mrjackwills/oxker/commit/1f62bb50210f2d66bb7215e42e8b21a3c1a6ec06)
+ draw_block constraints into consts, [0436ff1b](https://github.com/mrjackwills/oxker/commit/0436ff1b7356c80532048c7d497c66d331092b01)

### Tests
+ update port test with new colour, [f74ae3f5](https://github.com/mrjackwills/oxker/commit/f74ae3f5c34d74b78822078291fed401427c4cba)
+ color match tests updated, [5b287416](https://github.com/mrjackwills/oxker/commit/5b287416315942b19c62f8c66348ce28462d894c)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.6.0'>v0.6.0</a>
### 2024-01-18

### Chores
+ dependencies updated, [53b4bafb](https://github.com/mrjackwills/oxker/commit/53b4bafbe53312fe41608ddf33e865d474222aaa), [58ef1516](https://github.com/mrjackwills/oxker/commit/58ef151600e362048a607c8ae61a5edfe80ab1dd), [b6fd3502](https://github.com/mrjackwills/oxker/commit/b6fd35022a99ec0e982ddb154b0450d49c4840e9), [0438c108](https://github.com/mrjackwills/oxker/commit/0438c108bdd9815d7eae1b89c47c4e6438f358d6)
+ files formatted, [1806165c](https://github.com/mrjackwills/oxker/commit/1806165c3e266876b2d1806f7b662d09705f3aad)
+ create_release.sh check for unused lint, [d0b27211](https://github.com/mrjackwills/oxker/commit/d0b27211928f93f8455e1ee5a6a6485c6a21d382)

### Docs
= Readme updated, screenshot added, [7561a934](https://github.com/mrjackwills/oxker/commit/7561a93415c1e1f596b15edba95e7b32a939cd90), [4069e557](https://github.com/mrjackwills/oxker/commit/4069e5572f81cb689dbb9f735db919e4636cdccc)

### Features
+ Ports section added, closes [#21](https://github.com/mrjackwills/oxker/issues/21), [65a1afcb](https://github.com/mrjackwills/oxker/commit/65a1afcb0605604ede350a5630c775f94ebb74ee), [7a096a65](https://github.com/mrjackwills/oxker/commit/7a096a65c40924021fe643fe0aa1067095832df9)

### Fixes
+ sort arrow now on left of header, [40ddcb72](https://github.com/mrjackwills/oxker/commit/40ddcb727d2c1758d6dd26a58507b85b219f51e2)

### Refactors
+ rename string_wrapper > unit_struct, [27cf53e4](https://github.com/mrjackwills/oxker/commit/27cf53e41f8b379f606c1c27620ee08e79bac57e)

### Tests
+ Finally have tests, currently for layout and associated methods, at the moment running the tests will not interfere with any running Docker containers, [4bcf77db](https://github.com/mrjackwills/oxker/commit/4bcf77db776a36e0a8151ecfbda722a66c4ba46c)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.5.0'>v0.5.0</a>
### 2024-01-05

### Chores
+ .devcontainer updated, [2313618e](https://github.com/mrjackwills/oxker/commit/2313618eb1493ce41d70847b888c32b65fdc40ea), [5af6b8bc](https://github.com/mrjackwills/oxker/commit/5af6b8bcd31c3c38ff5a5799c76dc1cbe1167763), [9b0b6b10](https://github.com/mrjackwills/oxker/commit/9b0b6b10c3a0c1d5095490cfd3cda18d252f38f5)
+ alpine version bump, [061de032](https://github.com/mrjackwills/oxker/commit/061de032dad935c56c6caab419ecb5c9bbac4c7e)
+ dependencies updated, [0890991f](https://github.com/mrjackwills/oxker/commit/0890991ff1a239fe2d556a0c4eac6ae05beb9b50), [0a7b266b](https://github.com/mrjackwills/oxker/commit/0a7b266b2a358a4788ae877ca8a97f08eac4eef2), [333621f1](https://github.com/mrjackwills/oxker/commit/333621f1a7321c1fdf73fd35dd7f3ab165a9dc64), [3e51889c](https://github.com/mrjackwills/oxker/commit/3e51889cd8a552b1da463ae6a40d5de6eec188f5), [a179bb6f](https://github.com/mrjackwills/oxker/commit/a179bb6f6a7e076269fa830f56c0d4a31cf8488a)
+ file formatting, [eb5e74ae](https://github.com/mrjackwills/oxker/commit/eb5e74ae67d815bf49f241d2baf319e41cf9adf8)
+ Rust 1.75.0 linting, [81be75f2](https://github.com/mrjackwills/oxker/commit/81be75f27fd32a59ebff57e44c5022ff862df84b)

### Docs
+ screenshot updated, [0231d1bd](https://github.com/mrjackwills/oxker/commit/0231d1bdcda304300d289243a95044ab3bdce85c)
+ comment typo, [0ad1ec9d](https://github.com/mrjackwills/oxker/commit/0ad1ec9d85d6f0cac743b4421d0ad03432c9d717)

### Features
+ re-arrange columns, container name is now the first column, added a ContainerName & ContainerImage struct via `string_wrapper` macro, closes [#32](https://github.com/mrjackwills/oxker/issues/32), [e936bb4b](https://github.com/mrjackwills/oxker/commit/e936bb4b78980d0e34a1ef5e9f6f82a9ed0ddc7f)

### Fixes
+ Docker Commands hidden, [4301e470](https://github.com/mrjackwills/oxker/commit/4301e4709f99fc23ee438bf345b0dc698a05dc4e)
+ .gitattributes, [1234ea53](https://github.com/mrjackwills/oxker/commit/1234ea53897b2ed6ada0eb18cd81b8783a5dc5f5)

### Refactors
+ GitHub workflow action improved, [04b66af2](https://github.com/mrjackwills/oxker/commit/04b66af2b60c96cfbece0b13109e30b08ef35cc4)
+ sort_containers, [ccf8b55a](https://github.com/mrjackwills/oxker/commit/ccf8b55a7495982f72b4fb3af6e11a9bd7465216)
+ string_wrapper .get() return `&str`, [a722731c](https://github.com/mrjackwills/oxker/commit/a722731c6a77e00d1fb13967b51400aa34e72213)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.4.0'>v0.4.0</a>
### 2023-11-21

### Chores
+ workflow dependencies updated, [6a4cf649](https://github.com/mrjackwills/oxker/commit/6a4cf6490d08b976734e2bc8186d94c095700558)
+ dependencies updated, [e301b518](https://github.com/mrjackwills/oxker/commit/e301b51891e03ea40b2f904583119da3bc4daf53), [81d5b326](https://github.com/mrjackwills/oxker/commit/81d5b326db8881263f2c9072e1426948e41b4a0f), [294cc268](https://github.com/mrjackwills/oxker/commit/294cc2684f42daab9d51601e235a384f55617678)
+ lints moved from main.rs to Cargo.toml, [2de76e2f](https://github.com/mrjackwills/oxker/commit/2de76e2f358be9c1500ca3dc4f9df0979ed8ed28)
+ .devcontainer updated, [37d2ee91](https://github.com/mrjackwills/oxker/commit/37d2ee915625806dd11c2cc816a892aae12a777c)

### Features
+ Docker exec mode - you are now able to attempt to exec into a container by pressing the `e` key, closes [#28](https://github.com/mrjackwills/oxker/issues/28), [c8077bca](https://github.com/mrjackwills/oxker/commit/c8077bca0b673478cfbb417e677a885136ba9eff), [0e5ee143](https://github.com/mrjackwills/oxker/commit/0e5ee143b008c9d0ee0b681231a1568be227150b), [0e5ee143](https://github.com/mrjackwills/oxker/commit/0e5ee143b008c9d0ee0b681231a1568be227150b)
+ Export logs feature, press `s` to save logs, use `--save-dir` cli-arg to customise output location, closes [#1](https://github.com/mrjackwills/oxker/issues/1), [a15da5ed](https://github.com/mrjackwills/oxker/commit/a15da5ed43d07852504a4dd1884a189e3f5b9d84)

### Fixes
+ GitHub workflow, cargo publish before create release, [ae4ce3b5](https://github.com/mrjackwills/oxker/commit/ae4ce3b549c40cc8bd713f375f030b185179a6e2)
+ sorted created_at clash, closes [#22](https://github.com/mrjackwills/oxker/issues/22), [3a648939](https://github.com/mrjackwills/oxker/commit/3a6489396e87702ce94b349a7f47028ece7922f6)
+ `as_ref()` fixed, thanks [Daniel-Boll](https://github.com/Daniel-Boll), [77fbaa8b](https://github.com/mrjackwills/oxker/commit/77fbaa8b1669286369b6ec1edd80220c808b628f)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.3.3'>v0.3.3</a>
### 2023-10-21

### Chores
+ docker-compose Alpine bump, [d46c425f](https://github.com/mrjackwills/oxker/commit/d46c425fa29f3c1d27bd57764748bae7e0b82f69)
+ dependencies updated, [e6eecbbd](https://github.com/mrjackwills/oxker/commit/e6eecbbdce9c0ccff42aa8806dddb6e3364f990c), [ec93115e](https://github.com/mrjackwills/oxker/commit/ec93115ece83002fa127f3358f573319e29357e1), [b36daa5a](https://github.com/mrjackwills/oxker/commit/b36daa5aeaa354b6c4f45b7ae67ac1a6345ea1c0), [9c0de1f0](https://github.com/mrjackwills/oxker/commit/9c0de1f0feff3165d0f5b6cb5dda843c124bcfa4), [6dd953df](https://github.com/mrjackwills/oxker/commit/6dd953df458096aee5914411ce40e46c3f600ede)
+ Rust 1.73 linting, [21234c66](https://github.com/mrjackwills/oxker/commit/21234c66c3935330ccd58543dd3a915a293ac776)

### Docs
+ README.md updated, [3fd3915b](https://github.com/mrjackwills/oxker/commit/3fd3915b3e929742d8007109fd4c7b4a345eb0fa)

### Refactors
+ LogsTZ from `&str`, [44f581f5](https://github.com/mrjackwills/oxker/commit/44f581f5b3652cc4e623fe145141878754dca292)
+ from string impl, [ca79893d](https://github.com/mrjackwills/oxker/commit/ca79893df5f05ebf445ce194d578cb8213c9755e)
+ env handling, [18c3ed43](https://github.com/mrjackwills/oxker/commit/18c3ed43376a8b5e2d285d1b34a9f96843357d53)
+ `parse_args/mod.rs` > `parse_args.rs`, [a6ff4124](https://github.com/mrjackwills/oxker/commit/a6ff4124319ed17d3f1c46c916418f850ef1d3b0)
+ set_info_box take `&str`, [faeaca0c](https://github.com/mrjackwills/oxker/commit/faeaca0cd1bb243c7f4a7112b928be776b877ca1)
+ GitHub action use concurrency matrix, re-roder workflow, [85f1982f](https://github.com/mrjackwills/oxker/commit/85f1982f4066bfdbc764ab7b88588eded6a17f96)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.3.2'>v0.3.2</a>
### 2023-08-28

### Chores
+ dependencies updated, [8ce5a187](https://github.com/mrjackwills/oxker/commit/8ce5a1877a8c56d9bbab560c97e2596ea87cc4c0), [94a20584](https://github.com/mrjackwills/oxker/commit/94a20584e6ef0701c9f36838b0dfbcd911698dbe), [29e02e0d](https://github.com/mrjackwills/oxker/commit/29e02e0d1faae4a836c7e5cfd0d791338ff586e3), [8e4c2e68](https://github.com/mrjackwills/oxker/commit/8e4c2e686761df56920df2267b765ab1297c9972)
+ `_typos.toml` added, [84ba1020](https://github.com/mrjackwills/oxker/commit/84ba1020939606abf4a287cbd1de1f3a10d3f0c0)

### Features
+ Custom hostname. `oxker` will use `$DOCKER_HOST` env if set, or one can use the cli argument `--host`, which takes priority over the `$DOCKER_HOST`, closes [#30](https://github.com/mrjackwills/oxker/issues/30), [10950787](https://github.com/mrjackwills/oxker/commit/10950787649d2b66fc1e8cd8b85526df51479857)

### Refactors
+ `set_error()` takes `gui_state` and error enum, to make sure app_data & gui_state is in sync [62c78dfa](https://github.com/mrjackwills/oxker/commit/62c78dfaa50a8d8c084f7fbf7e203b50aaa731ae)
+ `fn loading_spin` doesn't need to be async, [2e27462d](https://github.com/mrjackwills/oxker/commit/2e27462d1b3f0bdb27d7646511e36d0c9af07f3e)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.3.1'>v0.3.1</a>
### 2023-06-04

### Chores
+ github workflow ubuntu latest, build for x86 musl, [4fa841e6](https://github.com/mrjackwills/oxker/commit/4fa841e6e74e3e10e3d3e82eac1a1ca1338814cf)
+ dependencies updated, [0caa92f6](https://github.com/mrjackwills/oxker/commit/0caa92f6a4728d50d8b2d8f15d96a21112732ec5), [1fd1dfc7](https://github.com/mrjackwills/oxker/commit/1fd1dfc75d6fa4e84451ebc845b9e1c730381f41)
+ `Spans` -> `Line`, ratatui 0.21 update, [4679ddc8](https://github.com/mrjackwills/oxker/commit/4679ddc885a9b35c901f3600b63fd9e86118264c), [0d37ac55](https://github.com/mrjackwills/oxker/commit/0d37ac55018038363e5f92dc4215996f8cff7b2e)
+ `create_release.sh` updated, [7dec5f14](https://github.com/mrjackwills/oxker/commit/7dec5f14a381d237c5e72fbf9551bcf398f93f3e)

### Fixes
+ workflow additional image fix, closes [#29](https://github.com/mrjackwills/oxker/issues/29), [47cda44b](https://github.com/mrjackwills/oxker/commit/47cda44b8213cfb8c3807df6c43e3f5dc2452b57)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.3.0'>v0.3.0</a>
### 2023-03-30

### Chores
+ dependencies updated, [7a9bdc96](https://github.com/mrjackwills/oxker/commit/7a9bdc9699594532e17a33e044ca0678693c8d3f), [58e03a75](https://github.com/mrjackwills/oxker/commit/58e03a750fe89b914b9069cb0c6c02a3d0929439), [b246e8c2](https://github.com/mrjackwills/oxker/commit/b246e8c25af0c5136953afca7c694cda66550d9b)

### Docs
+ README.md and screenshot updated, [73ab7580](https://github.com/mrjackwills/oxker/commit/73ab7580c61dd59c59f10872629111360afb9033)

### Features
+ Ability to delete a container, be warned, as this will force delete, closes [#27](https://github.com/mrjackwills/oxker/issues/27), [937202fe](https://github.com/mrjackwills/oxker/commit/937202fe34d1692693c62dd1a7ad19db37651233), [b25f8b18](https://github.com/mrjackwills/oxker/commit/b25f8b18f4f2acd5c9af4a1d40655761d1bd720e)
+ Publish images to `ghcr.io` as well as Docker Hub, and correctly tag images with `latest` and the current sermver, [cb1271cf](https://github.com/mrjackwills/oxker/commit/cb1271cf7f21c898020481ad85914a3dcc83ec93)
+ Replace `tui-rs` with [ratatui](https://github.com/tui-rs-revival/ratatui), [d431f850](https://github.com/mrjackwills/oxker/commit/d431f850219b28af2bc45f3b6917377604596a40)

### Fixes
+ out of bound bug in `heading_bar()`, [b9c125da](https://github.com/mrjackwills/oxker/commit/b9c125da46fe0eb4aae15c354d87ac824e9cb83a)
+ `-d` arg error text updated, [e0b49be8](https://github.com/mrjackwills/oxker/commit/e0b49be84062abdfcb636418f57043fad37d06ec)

### Refactors
+ `popup()` use `saturating_x()` rather than `checked_x()`, [d628e802](https://github.com/mrjackwills/oxker/commit/d628e8029942916053b3b7e72d363b1290fc5711)
+ button_item() include brackets, [7c92ffef](https://github.com/mrjackwills/oxker/commit/7c92ffef7da20143a31706a310b5e6f2c3e0554f)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.5'>v0.2.5</a>
### 2023-03-13

### Chores
+ Rust 1.68.0 clippy linting, [5582c454](https://github.com/mrjackwills/oxker/commit/5582c45403413d3355bbcd629cfad559296f5e5b)
+ devcontainer use sparse protocol index, [20b79e9c](https://github.com/mrjackwills/oxker/commit/20b79e9cd5bf75bb253158c0b590284139e0291d)
+ dependencies updated, [0c07d4b4](https://github.com/mrjackwills/oxker/commit/0c07d4b40607a0eba003b6dcd0345ec0543c6264), [601a73d2](https://github.com/mrjackwills/oxker/commit/601a73d2c830043a25d64922c4d4aa38f8801912), [5aaa3c1a](https://github.com/mrjackwills/oxker/commit/5aaa3c1ab08b0c85df9bfce18a3e60206556fa58), [7a156303](https://github.com/mrjackwills/oxker/commit/7a1563030e48499da7f41033673c70deefe3de8a), [45715775](https://github.com/mrjackwills/oxker/commit/457157755baa1f9e9cfef9315a7940c357b0953d)

### Features
+ increase mpsc channel size from 16 to 32 messages, [924f14e9](https://github.com/mrjackwills/oxker/commit/924f14e998f79f731447a2eded038eab51f2e932)
+ KeyEvents send modifier, so can quit on `ctrl + c`, [598f67c6](https://github.com/mrjackwills/oxker/commit/598f67c6f6a8713102bcc415f0409911763bb914)
+ only send relevant mouse events to input handler, [507660d8](https://github.com/mrjackwills/oxker/commit/507660d835d0beaa8cd021110401ecc58c0613c6)

### Fixes
+ GitHub workflow on SEMEVR tag only, [14077386](https://github.com/mrjackwills/oxker/commit/140773865165bf006e74f9d436fc744220f5eae7)

### Refactors
+ replace `unwrap_or(())` with `.ok()`, [8ba37a16](https://github.com/mrjackwills/oxker/commit/8ba37a165bb89277ab957194da6464bdb35be2e6)
+ use `unwrap_or_default()`, [79de92c3](https://github.com/mrjackwills/oxker/commit/79de92c3921702417bb2df1f44939a7b09cb7fa0)
+ Result return, [d9f0bd55](https://github.com/mrjackwills/oxker/commit/d9f0bd5566e27218b8c8eaba6ece237907771c1d)

### Reverts
+ temporary devcontainer buildkit fix removed, [d1497a44](https://github.com/mrjackwills/oxker/commit/d1497a4451f4de54d3cc26c5a3957cd636c29118)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.4'>v0.2.4</a>
### 2023-03-02

### Chores
+ dependencies updated, [aac3ef2b](https://github.com/mrjackwills/oxker/commit/aac3ef2b1def3345d749d813d9b76020d6b5e5ca), [4723be7f](https://github.com/mrjackwills/oxker/commit/4723be7fb2eb101024bb9d5a514e2c6cc51eb6f6), [c69ab4f7](https://github.com/mrjackwills/oxker/commit/c69ab4f7c3b873f25ea46958add37be78d23e9cf), [ba643786](https://github.com/mrjackwills/oxker/commit/ba6437862dae0f422660a602aeabd6217d023fac), [2bb4c338](https://github.com/mrjackwills/oxker/commit/2bb4c338903e09856053894d9646307e31d32f1c)
+ dev container install x86 musl toolchain, [e650034d](https://github.com/mrjackwills/oxker/commit/e650034d50f01a7598876d4f2887df691700e06a)

### Docs
+ typos removed, [23ad9a5f](https://github.com/mrjackwills/oxker/commit/23ad9a5fb3cacf3fb8cb70c65ca9133ed9949e45), [cebb975c](https://github.com/mrjackwills/oxker/commit/cebb975cb82f653407ec801fd8c726ca6ed68289), [fdc67c92](https://github.com/mrjackwills/oxker/commit/fdc67c9249a239bac97a78b20c9378472865209c)
+ comments improved, [ec962295](https://github.com/mrjackwills/oxker/commit/ec962295a8789ff8010604e974969bf618ea7108)

### Features
+ mouse capture is now more specific, should have substantial performance impact, 10x reduction in cpu usage when mouse is moved observed, as well as fixing intermittent mouse events output bug, [0a1b5311](https://github.com/mrjackwills/oxker/commit/0a1b53111627206cc7436589e5b7212e1b72edb8), [93f7c07f](https://github.com/mrjackwills/oxker/commit/93f7c07f708885f8870da5dfb6d57c62f93c9c78), [c74f6c11](https://github.com/mrjackwills/oxker/commit/c74f6c1179b5f62989eb74f395a56b43a8781b03)
+ improve the styling of the help information popup, [28de74b8](https://github.com/mrjackwills/oxker/commit/28de74b866f07c8543e46be3cab929eff28953fd)
+ use checked_sub & checked_div for bounds checks, [72279e26](https://github.com/mrjackwills/oxker/commit/72279e26ae996353c95a75527f704bac1e4bcf4d)

### Fixes
+ correctly set gui error, [340893a8](https://github.com/mrjackwills/oxker/commit/340893a860e99ec4029d12613f2a6de3cb7b47e2)

### Refactors
+ dead code removed, [b8f5792d](https://github.com/mrjackwills/oxker/commit/b8f5792d1865d3a398cd7f23aa9473a55dc6ea44)
+ improve the get_width function, [04c26fe8](https://github.com/mrjackwills/oxker/commit/04c26fe8fc7c79506921b9cff42825b1ee132737)
+ place ui methods into a Ui struct, [3437df59](https://github.com/mrjackwills/oxker/commit/3437df59884f084624031fceb34ea3012a8e2251)
+ get_horizontal/vertical constraints into single method, [e8f5cf9c](https://github.com/mrjackwills/oxker/commit/e8f5cf9c6f8cd5f807a05fb61e31d7cd1426486f)
+ docker update_everything variables, [074cb957](https://github.com/mrjackwills/oxker/commit/074cb957f274675a468f08fecb1c43ff7453217d)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.3'>v0.2.3</a>
### 2023-02-04

### Fixes
+ Container runner `FROM scratch` (missing from v0.2.2 D'oh), this now should actually reduce Docker image size by ~60%, [0bd317b7](https://github.com/mrjackwills/oxker/commit/0bd317b7ce6f9f42a614c488099b5fc7a14d91c7)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.2'>v0.2.2</a>
### 2023-02-04

### Chores
+ devcontainer.json updated, typos-cli installed, temporary(?) buildkit fix, [3c6a8db6](https://github.com/mrjackwills/oxker/commit/3c6a8db6ef74d499b49fabe8912785cac16d9c4b)
+ create_release.sh check for typos, [310a63f4](https://github.com/mrjackwills/oxker/commit/310a63f4cabaa374797a7e4ed0d7fd1f5e79c8fe)

### Docs
+ AUR install instructions, thanks [orhun](https://github.com/orhun), [c5aa346b](https://github.com/mrjackwills/oxker/commit/c5aa346bca139cc5ece1f4127293977924d16fca)
+ typos fixes, thanks [kianmeng](https://github.com/kianmeng), [5052d7ab](https://github.com/mrjackwills/oxker/commit/5052d7ab0a156c43cadbd922c0019b284f24943a)
+ Readme.md styling tweak, [310a63f4](https://github.com/mrjackwills/oxker/commit/310a63f4cabaa374797a7e4ed0d7fd1f5e79c8fe)
+ Contributing guide, [5aaa00d6](https://github.com/mrjackwills/oxker/commit/5aaa00d6a3c58d98cb250b7b14584238df02961c), [a44b15f7](https://github.com/mrjackwills/oxker/commit/a44b15f76088561a0e272d4e7456197c2aaabdb4)

### Features
+ Use a scratch container for the docker image, should reduce image size by around 60%. This checks for the ENV `OXKER_RUNTIME=container`, which is automatically set by the docker image, [17b71b6b](https://github.com/mrjackwills/oxker/commit/17b71b6b41f6a98a0f92277f40a88f4b1b8a1328)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.1'>v0.2.1</a>
### 2023-01-29

### Chores
+ dependencies updated, [c129f474](https://github.com/mrjackwills/oxker/commit/c129f474fe2976454b1868d00e8d7d99b87ec23b), [9788b8af](https://github.com/mrjackwills/oxker/commit/9788b8afd98e59b1d4412a8adc54b34d2c5671fd), [2ab88eb2](https://github.com/mrjackwills/oxker/commit/2ab88eb26e9bbbc4dad4651256d8d9b044ea3272)

### Docs
+ comment typo, [10255791](https://github.com/mrjackwills/oxker/commit/1025579138f11e4987263c7bfe936c4c8542f8b3)

### Fixes
+ deadlock on draw logs when no containers found, [68e444bf](https://github.com/mrjackwills/oxker/commit/68e444bfc393eb46bac2b99eb57697bb9b0451af)
+ github workflow release on main only (with semver tag), [e4ca41df](https://github.com/mrjackwills/oxker/commit/e4ca41dfd8ec3acae202a2d2464b8e18f5c5bdd5), [749ec712](https://github.com/mrjackwills/oxker/commit/749ec712f07cff2c941aed6726c56bdbd5cb8d2c)

### Refactors
+ major refactor of internal data handling, [b4488e4b](https://github.com/mrjackwills/oxker/commit/b4488e4bdb0252f5c5680cee6a46427f22a282ab)
+ needless (double) referencing removed, [a174dafe](https://github.com/mrjackwills/oxker/commit/a174dafe1b05908735680a874dc551a86da24777)
+ app_data methods re-ordered & renamed, [c0bb5355](https://github.com/mrjackwills/oxker/commit/c0bb5355d6a5d352260655110ce3d5ab695acda9)

### Reverts
+ is_running AtomicBool back to SeqCst, [c4d80061](https://github.com/mrjackwills/oxker/commit/c4d80061dab94afd08d4d793dc147f878c965ad6)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.0'>v0.2.0</a>
### 2023-01-21

### Chores
+ dependencies updated, [8cd199db](https://github.com/mrjackwills/oxker/commit/8cd199db49186fad6ce432bb277e3a10f0a08d34), [d880b829](https://github.com/mrjackwills/oxker/commit/d880b829c123dbe57deccadef97810e45c083737), [66d57c99](https://github.com/mrjackwills/oxker/commit/66d57c99558ca14d9593d6dbfd5b0e8e5d59055d), [33f93749](https://github.com/mrjackwills/oxker/commit/33f9374908942f4a3b90be227fad94ca353cf351), [007d5d83](https://github.com/mrjackwills/oxker/commit/007d5d83d7f1b93e1e78777a4417b2740db706bd)
+ create_release.sh typos, [9a27d46a](https://github.com/mrjackwills/oxker/commit/9a27d46a044452080144ee1367dc95886b10abf8)
+ dev container post create install cross, [2d253f03](https://github.com/mrjackwills/oxker/commit/2d253f034182741d434e4bac12317f24221d0d4a)

### Features
**all potentially considered breaking changes**
+ store Logs in own struct, use a hashset to track timestamps, hopefully closes [#11](https://github.com/mrjackwills/oxker/issues/11), [657ea2d7](https://github.com/mrjackwills/oxker/commit/657ea2d751a71f05b17547b47c492d5676817336)
+ Spawn docker commands into own thread, can now execute multiple docker commands at the same time, [9ec43e12](https://github.com/mrjackwills/oxker/commit/9ec43e124a62a80f4e78acba85fc3af5980ce260)
+ align memory columns correctly, minimum byte display value now `0.00 kB`, rather than `0 B`, closes [#20](https://github.com/mrjackwills/oxker/issues/20), [bd7dfcd2](https://github.com/mrjackwills/oxker/commit/bd7dfcd2c512a527d66a1388f90006988a487186), [51c58001](https://github.com/mrjackwills/oxker/commit/51c580010a24de2427373795803936d498dc8cee)

### Refactors
+ main.rs tidy up, [97b89349](https://github.com/mrjackwills/oxker/commit/97b89349dc2de275ca514a1e6420255a63d775e8)
+ derive Default for GuiState, [9dcd0509](https://github.com/mrjackwills/oxker/commit/9dcd0509efeb464f58fb53d813bd78de2447949d)
+ param reduction, AtomicBool to Relaxed, [0350293d](https://github.com/mrjackwills/oxker/commit/0350293de3c00c6e5e5d787b7596bb3413d1cda1)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.11'>v0.1.11</a>
### 2023-01-03

### Chores
+ dependencies updated, [9b09146a](https://github.com/mrjackwills/oxker/commit/9b09146aadae5727a5fee4de5fe0c1d70c581c22)

### Features
+ `install.sh` script added, for automated platform selection, download, and installation, [7a42eba6](https://github.com/mrjackwills/oxker/commit/7a42eba6b0968314af40ff87bcc42d288f6860bc), [e0703b76](https://github.com/mrjackwills/oxker/commit/e0703b76a1a28cfe266f130a7f7dec92f1b5ad58)

### Fixes
+ If a sort order is set, sort containers on every `update_stats()` execution, [cfdea775](https://github.com/mrjackwills/oxker/commit/cfdea77594e48c8c20a4d6e6c7ea31c9181361a1)

### Refactors
+ input sort executed in app_data struct `sort_by_header()`, [3cdc5fae](https://github.com/mrjackwills/oxker/commit/3cdc5fae02097628799209f371ae9292e513e76c)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.10'>v0.1.10</a>
### 2022-12-25

### Chores
+ dependencies updated, [1525b315](https://github.com/mrjackwills/oxker/commit/1525b3150293015c0fb2f2161da463b21ac2694c), [8d539ab1](https://github.com/mrjackwills/oxker/commit/8d539ab14809136d743c49d60779687fc8eeef6d), [1774217a](https://github.com/mrjackwills/oxker/commit/1774217a8a657d261397d213e5ecee667cf3b6b1)
+ Rust 1.66 linting, [bf9dcac7](https://github.com/mrjackwills/oxker/commit/bf9dcac7045c0d2314df147ec2744a3ad886564b)

### Features
+ Caching on github action, [a91c9aa4](https://github.com/mrjackwills/oxker/commit/a91c9aa45ffd5c998cd1b83d8e90d0912893c31f)

### Fixes
+ comment typo, [7899b773](https://github.com/mrjackwills/oxker/commit/7899b773569fed86343a035d3023bf34297fdabb)

### Refactors
+ remove_ansi() to single liner, [57c3a6c1](https://github.com/mrjackwills/oxker/commit/57c3a6c186b916faba24bf7b5cdbbda31d636a7e)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.9'>v0.1.9</a>
### 2022-12-05

### Fixes
+ disallow commands to be sent to a dockerised oxker container, closes [#19](https://github.com/mrjackwills/oxker/issues/19), [160b8021](https://github.com/mrjackwills/oxker/commit/160b8021b1de898064756b53c127d49b8096ce4d)
+ if no container created time, use 0, instead of system_time(), [1adb61ce](https://github.com/mrjackwills/oxker/commit/1adb61ce3b029d4fcf51961958d483b2fae8825a)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.8'>v0.1.8</a>
### 2022-12-05

### Chores
+ dependencies updated, [e3aa4420](https://github.com/mrjackwills/oxker/commit/e3aa4420cb510df0381e311d37e768937070387a)
+ docker-compose.yml alpine bump, [911c6596](https://github.com/mrjackwills/oxker/commit/911c6596684db4ccbe7a55aadd6f595a95f89bb0)
+ github workflow use dtolnay/rust-toolchain, [57c18878](https://github.com/mrjackwills/oxker/commit/57c18878690477a05d7330112a65d1d58a07901e)

### Features
+ Clicking a header now toggles between Ascending -> Descending -> Default. Use the containers created_time as the default order - maybe add created column in future version, closes [#18](https://github.com/mrjackwills/oxker/issues/18), [cf14ba49](https://github.com/mrjackwills/oxker/commit/cf14ba498987db587c0f5bef8a67cf4113ffcb1e), [d1de2914](https://github.com/mrjackwills/oxker/commit/d1de291473d8a1028f1936429832d3820d75df54)
+ `-s` flag for showing the oxker container when executing the docker image, [c93870e5](https://github.com/mrjackwills/oxker/commit/c93870e5fbbc7df35c69d32e4460d2104e521e33)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.7'>v0.1.7</a>
### 2022-11-13

### Chores
+ update dependencies, closes [#17](https://github.com/mrjackwills/oxker/issues/17), [8a5d0ef8](https://github.com/mrjackwills/oxker/commit/8a5d0ef8376e3739dda5b0ed4c3e75e565deed45), [eadfc3d6](https://github.com/mrjackwills/oxker/commit/eadfc3d6c6896ecc8cff88c6a9e9c8b3e477c0cd)
+ aggressive linting with Rust 1.65.0, [8f3a1513](https://github.com/mrjackwills/oxker/commit/8f3a15137155dc374e6b2822c9155c07d05d5e28)

### Docs
+ README.md improved Download & Install section, and now available on [NixPkg](https://search.nixos.org/packages?channel=unstable&show=oxker&from=0&size=50&sort=relevance&type=packages&query=oxker), thanks [siph](https://github.com/siph), [67a9e183](https://github.com/mrjackwills/oxker/commit/67a9e183ca04199da758255075ff7e73061eb850)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.6'>v0.1.6</a>
### 2022-10-16

### Chores
+ Cargo update, [c3e72ae7](https://github.com/mrjackwills/oxker/commit/c3e72ae7369a25d903f39e55a4349cb005671dd4),
+ create_release.sh v0.1.0, [3c8d59c6](https://github.com/mrjackwills/oxker/commit/3c8d59c666bd4cda9ca54989b2f1b48bba17bc57),
+ uuid updated to version 1.2, [438ad770](https://github.com/mrjackwills/oxker/commit/438ad770f4a5ecb5f4bbc308066ad9e808f66514),

### Fixes
+ loading icon shifting error fix, also make icon white, closes [#15](https://github.com/mrjackwills/oxker/issues/15), [59797685](https://github.com/mrjackwills/oxker/commit/59797685dffa29752a48c98e6cf465884d6d9df6),

### Features
+ Show container name in log panel title, closes [#16](https://github.com/mrjackwills/oxker/issues/16), [9cb0c414](https://github.com/mrjackwills/oxker/commit/9cb0c414afc284947fc2b8494504387e4e7edd87),
+ use gui_state HashSet to keep track of application gui state, [9e9d5155](https://github.com/mrjackwills/oxker/commit/9e9d51559a13944622abf4fcbd3bd63766d11467),
+ terminal.clear() after run_app finished, [67c49575](https://github.com/mrjackwills/oxker/commit/67c49575682cb271fac0998ff377a6504cd0bc86),

### Refactors
+ CpuStats & MemStats use tuple struct, [a060d032](https://github.com/mrjackwills/oxker/commit/a060d032586a0707ac91cb13d922aae0850449c5),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.5'>v0.1.5</a>
### 2022-10-07

### Chores
+ Update clap to v4, [15597dbe](https://github.com/mrjackwills/oxker/commit/15597dbe6942ec053541398ce0e9dedc10a4d3ea),

### Docs
+ readme.md updated, [a05bf561](https://github.com/mrjackwills/oxker/commit/a05bf561cc6d96237f683ab0b3c782d6841974d9),

### Features
+ use newtype construct for container id, [41cbb84f](https://github.com/mrjackwills/oxker/commit/41cbb84f2896f8be2c37eba87e390d998aff7382),

### Refactors
+ Impl Copy where able to, [e76878f4](https://github.com/mrjackwills/oxker/commit/e76878f424d72b943713ef84e95e25fada77d79e),
+ replace async fn with just fn, [17dc604b](https://github.com/mrjackwills/oxker/commit/17dc604befac75cb9dc0311a0e43f9927fe0ca30),
+ remove pointless clone()'s & variable declarations, [6731002e](https://github.com/mrjackwills/oxker/commit/6731002ee42c9460042c2c38aff5101b1bcebbe6),
+ replace String::from("") with String::new(), [62fb2247](https://github.com/mrjackwills/oxker/commit/62fb22478697cc9a7ab9fb562a724965b437233a),
+ replace map_or_else with map_or, [3e26f292](https://github.com/mrjackwills/oxker/commit/3e26f292c7dc5e13af4580952767ebe821aa5183), [5660b34d](https://github.com/mrjackwills/oxker/commit/5660b34d5149dce27706ff6daa90b854e6f84e14),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.4'>v0.1.4</a>
### 2022-09-07

### Chores
+ dependencies updated, [a3168daa](https://github.com/mrjackwills/oxker/commit/a3168daa3f769a6747dfbe61103073a7e80a1485),[78e59160](https://github.com/mrjackwills/oxker/commit/78e59160bb6a978ee80e3a99eb72f051fb64e737),

### Features
+ containerize self, github action to build and push to [Docker Hub](https://hub.docker.com/r/mrjackwills/oxker), [07f97202](https://github.com/mrjackwills/oxker/commit/07f972022a69f22bac57925e6ad84234381f7890),
+ gui_state is_loading use a HashSet to enable multiple things be loading at the same time, [66583e1b](https://github.com/mrjackwills/oxker/commit/66583e1b037b7e2f3e47948d70d8a4c6f6a2f2d5),
+ github action publish to crates.io, [90b2e3f6](https://github.com/mrjackwills/oxker/commit/90b2e3f6db0d5f63840cd80888a30da6ecc22f20),
+ derive Eq where appropriate, [d7c2601f](https://github.com/mrjackwills/oxker/commit/d7c2601f959bc12a64cd25cef59c837e1e8c2b2a),
+ ignore containers 'oxker' containers, [1be9f52a](https://github.com/mrjackwills/oxker/commit/1be9f52ad4a68f93142784e9df630c59cdec0a79),
+ update container info if container is either running OR restarting, [5f12362d](https://github.com/mrjackwills/oxker/commit/5f12362db7cb61ca68f75b99ecfc9725380d87d2),

### Fixes
+ devcontainer updated, [3bde4f56](https://github.com/mrjackwills/oxker/commit/3bde4f5629539cab3dbb57556663ab81685f9d7a),
+ Use Binate enum to enable two cycles of cpu/mem update to be executed (for each container) at the same time, refactor hashmap spawn insertions, [7ec58e79](https://github.com/mrjackwills/oxker/commit/7ec58e79a1316ad1f7e50a2781dea0fe8422c588),

### Refactors
+ improved way to remove leading '/' of container name, [832e9782](https://github.com/mrjackwills/oxker/commit/832e9782d7765872cbb84df6b3703fc08cb353c9),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.3'>v0.1.3</a>
### 2022-08-04

### Chores
+ dependencies updated, [d9801cdf](https://github.com/mrjackwills/oxker/commit/d9801cdf372521fe5624a8d68fac83ed39ef81f4),
+ linting: nursery, pedantic, unused_unwraps, [1bd61d4c](https://github.com/mrjackwills/oxker/commit/1bd61d4ce8b369d6d078201add3eea0f59fe0dea), [1263662b](https://github.com/mrjackwills/oxker/commit/1263662bd9412afacddbc10721bf216ae3a843f1), [ca3315a6](https://github.com/mrjackwills/oxker/commit/ca3315a69f593ad705eb637f227f195edd7781b2),

### Features
+ build all production targets on release, [44f8140e](https://github.com/mrjackwills/oxker/commit/44f8140eaec330abe5a94f3ddae9e8b223688aa8),

### Fixes
+ toml keywords, [dd2d82d1](https://github.com/mrjackwills/oxker/commit/dd2d82d114537e09dbeb12f360157f0e68e7846e),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.2'>v0.1.2</a>
### 2022-07-23

### Fixes
+ remove reqwest dependency, [10ff8bab](https://github.com/mrjackwills/oxker/commit/10ff8bab5f01f097fd6cdec60b2be947f238197b),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.1'>v0.1.1</a>
### 2022-07-23

### Chores
+ update Cargo.toml, in preparation for crates.io publishing, [fdc6898e](https://github.com/mrjackwills/oxker/commit/fdc6898e20c41415f03e310d7b84af4b6c39ab62),

### Docs
+ added cargo install instructions, [c774b10d](https://github.com/mrjackwills/oxker/commit/c774b10d557b10885b9d3a0b3612330a8ecb1cd5),

### Fixes
+ use SpawnId for docker hashmap JoinHandle mapping, [1ae95d58](https://github.com/mrjackwills/oxker/commit/1ae95d58c3302a95d5a0a2f0b61b126c72b6e166),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.0'>v0.1.0</a>
### 2022-07-23

### Chores
+ dependencies updated, [cf7e02dd](https://github.com/mrjackwills/oxker/commit/cf7e02dde94f69832a2e485b99785afc66a5bc15),

### Features
+ Enable sorting of containers by each, and every, heading. Either via keyboard or mouse, closes [#3](https://github.com/mrjackwills/oxker/issues/3), [a6c296f2](https://github.com/mrjackwills/oxker/commit/a6c296f2cde56cf241bcd696cab8bd477270e5f4),
+ Spawn & track docker information update requests, multiple identical requests cannot be executed, [740c059b](https://github.com/mrjackwills/oxker/commit/740c059b276f35acd1cb03f1030134646bf8a07d),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.6'>v0.0.6</a>
### 2022-07-06

### Docs
+ readme update, [f29e29ad](https://github.com/mrjackwills/oxker/commit/f29e29ad151ddf424ba630e6d33edf19acfd7636),
+ comments improved, [1674db8a](https://github.com/mrjackwills/oxker/commit/1674db8a20aafa447732deb2e44ac8b97cf0471b),
+ readme logo size, [a733efa6](https://github.com/mrjackwills/oxker/commit/a733efa65865e04d9ec86c7ca8785dfbae635695),

### Fixes
+ Remove unwraps(), [61db81ec](https://github.com/mrjackwills/oxker/commit/61db81ecfe5684ddb8a360715f43357a042162c0),
+ Help menu alt+tab > shift+tab typo, thanks [siph](https://github.com/siph), [04466803](https://github.com/mrjackwills/oxker/commit/04466803481b75feb7d7f275248279fdb8729862),

### Refactors
+ tokio spawns, [1fd230f2](https://github.com/mrjackwills/oxker/commit/1fd230f2f3cf4e376058359515e76f4fa6e425c2),
+ max_line_width(), [a5d7dabb](https://github.com/mrjackwills/oxker/commit/a5d7dabbd68dc15a081df33352ce3b55d9a9891c),
+ create_release dead code removed, [297979c1](https://github.com/mrjackwills/oxker/commit/297979c197c2defd409053d8da724f922b0bba1b),


# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.5'>v0.0.5</a>
### 2022-05-30

### Docs
+ Readme one-liner to download & install latest version, [11d5ba36](https://github.com/mrjackwills/oxker/commit/11d5ba361ee4c11d080f1c3c14d8bb677cbfd1fc),
+ Example docker-compose.yml bump alpine version to 3.16, [98c83f2f](https://github.com/mrjackwills/oxker/commit/98c83f2f68f59e78f0c78270c59886630d98913c),

### Fixes
+ use Some() checks to make sure that container item indexes are still valid, else can create out-of-bounds errors, closes [#8](https://github.com/mrjackwills/oxker/issues/8), [4cf02e3f](https://github.com/mrjackwills/oxker/commit/4cf02e3f04426ef44ec5a7421687f2104ac5102f),
+ Remove + replace as many unwrap()'s as possible, [d8e22d74](https://github.com/mrjackwills/oxker/commit/d8e22d7444965f1874d7367259310440a889432b),
+ Help panel typo, [e497f3f2](https://github.com/mrjackwills/oxker/commit/e497f3f2d9e1dca99469860c2e728c99e29353ad),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.4'>v0.0.4</a>
### 2022-05-08

### Fixes
+ Help menu logo corrected, [2f545202](https://github.com/mrjackwills/oxker/commit/2f5452027e86f714729b804d4bf65306e755df7f),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.3'>v0.0.3</a>
### 2022-05-08

### Docs
+ slight readme tweaks, [eb9184a1](https://github.com/mrjackwills/oxker/commit/eb9184a1aee64be1c20fabd482bfcbe676bed049),

### Features
+ vim movement keys, 'j' & 'k', to move through menus, thanks [siph](https://github.com/siph), [77eb33c0](https://github.com/mrjackwills/oxker/commit/77eb33c008e36965d84d1eafbbc3733af19fd262),

### Fixes
+ create_release.sh correctly link to closed issues, [5820d0a9](https://github.com/mrjackwills/oxker/commit/5820d0a9b68ead71d031377c5d22138675d7dfa8),

### Refactors
+ generate_block reduce params, insert into area hashmap from inside generate_block function, [32705a60](https://github.com/mrjackwills/oxker/commit/32705a60c4f865eb829cc460b2ac82db79107c1a),
+ dead code removed, [d20e1bcd](https://github.com/mrjackwills/oxker/commit/d20e1bcd47965859a04f8e080509a5afb2de36d9),
+ create_release.sh improved flow & comments, [4283a285](https://github.com/mrjackwills/oxker/commit/4283a285e2e60907e432294e3b97a759ec06a23d),


# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.2'>v0.0.2</a>
### 2022-04-29

### Features
+ allow toggling of mouse capture, to select & copy text with mouse, closes [#2](https://github.com/mrjackwills/oxker/issues/2), [aec184ea](https://github.com/mrjackwills/oxker/commit/aec184ea22b289e91942a4c3e6a415685884bc47),
+ show id column, [b10f9274](https://github.com/mrjackwills/oxker/commit/b10f927481c9e38a48c1d4b94e744ec48e8b6ba6),
+ draw_popup, using enum to draw in one of 9 areas, closes [#6](https://github.com/mrjackwills/oxker/issues/6), [1017850a](https://github.com/mrjackwills/oxker/commit/1017850a6cc91328abc1127bdb117495f5e909d8),
+ use a message rx/sx for all docker commands, remove update loop, wait for update message from gui instead, [9b70fdfa](https://github.com/mrjackwills/oxker/commit/9b70fdfad7b38361ebee301bdc2545d3f0dfcf9e),

### Fixes
+ readme.md typo, [589501f9](https://github.com/mrjackwills/oxker/commit/589501f9a4a0bfabdb0654e68cc0c752c529d97a),
+ column heading mem > memory, [5e8e6b59](https://github.com/mrjackwills/oxker/commit/5e8e6b590b06f01a542fdd10bae8f14d303ab08a),
+ cargo fmt added to create_release.sh, [bb29c0eb](https://github.com/mrjackwills/oxker/commit/bb29c0ebfafd6a9a036eb317a240954d1405966e),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.1'>v0.0.1</a>
### 2022-04-25

+ init commit
