### 2024-05-07

### Chores
+ Dependencies updated, [07e293ac2ce2e7deb5735154fcdb24ef83a19b67], [27d72c547e738f6816cd4b353ac881e454a0be70]

### Features
+ Allow closing dialogs with `Escape`, thanks [JCQuintas](https://github.com/JCQuintas), [0e4c3ceab933458d40b54d5fcff7e6cf7a3ab315]

### Fixes
+ correct header display when terminal width changes, [4628803b2b9fe63522d033b192763ed6ff5b57dd]

### Refactors
+ use tokio CancellationToken, [0631a73ec27530f8fcc88988a0a02ca75e32c5ba]
+ impl AsyncTTY, [bf33776e9a61684032a80d22d995ba7e0446620e]

### Tests
+ reduced header section test, [aa0947405393db2c306e86986183514cbc0f5a75]
+ test_draw_blocks_help() with add esc text, [ff839af4ef68193149d6456e70fee189228c4a44]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
