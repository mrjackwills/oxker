### 2023-08-28

### Chores
+ dependencies updated, [8ce5a1877a8c56d9bbab560c97e2596ea87cc4c0], [94a20584e6ef0701c9f36838b0dfbcd911698dbe], [29e02e0d1faae4a836c7e5cfd0d791338ff586e3], [8e4c2e686761df56920df2267b765ab1297c9972]
+ `_typos.toml` added, [84ba1020939606abf4a287cbd1de1f3a10d3f0c0]

### Features
+ Custom hostname. `oxker` will use `$DOCKER_HOST` env if set, or one can use the cli argument `--host`, which takes priority over the `$DOCKER_HOST`, closes #30, [10950787649d2b66fc1e8cd8b85526df51479857]

### Refactors
+ `set_error()` takes `gui_state` and error enum, to make sure app_data & gui_state is in sync [62c78dfaa50a8d8c084f7fbf7e203b50aaa731ae]
+ `fn loading_spin` doesn't need to be async, [2e27462d1b3f0bdb27d7646511e36d0c9af07f3e]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
