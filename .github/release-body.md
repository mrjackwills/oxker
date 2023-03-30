### 2023-03-30

### Chores
+ dependencies updated, [7a9bdc9699594532e17a33e044ca0678693c8d3f], [58e03a750fe89b914b9069cb0c6c02a3d0929439], [b246e8c25af0c5136953afca7c694cda66550d9b]

### Docs
+ README.md and screenshot updated, [73ab7580c61dd59c59f10872629111360afb9033]

### Features
+ Ability to delete a container, be warned, as this will force delete, closes #27, [937202fe34d1692693c62dd1a7ad19db37651233], [b25f8b18f4f2acd5c9af4a1d40655761d1bd720e]
+ Publish images to `ghcr.io` as well as Docker Hub, and correctly tag images with `latest` and the current sermver, [cb1271cf7f21c898020481ad85914a3dcc83ec93]
+ Replace `tui-rs` with [ratatui](https://github.com/tui-rs-revival/ratatui), [d431f850219b28af2bc45f3b6917377604596a40]

### Fixes
+ out of bound bug in `heading_bar()`, [b9c125da46fe0eb4aae15c354d87ac824e9cb83a]
+ `-d` arg error text updated, [e0b49be84062abdfcb636418f57043fad37d06ec]

### Refactors
+ `popup()` use `saturating_x()` rather than `checked_x()`, [d628e8029942916053b3b7e72d363b1290fc5711]
+ button_item() include brackets, [7c92ffef7da20143a31706a310b5e6f2c3e0554f]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
