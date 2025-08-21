### 2025-08-21

### Chores
+ Dependencies updated, [ced885e0128b6d5d3a3c7cb97d7e53bc2da64893], [f9b40ea03d0e70e235c28646ff3f9ebb468a904d]
+ Rust 1.89 linting, [79d19ceeb81ae60bc5562683e405d6e74e6f2578]
+ GitHub workflow updated, [08384200558fa1b9d378ea62ea832708caebaa91], [6573af1ed7d382a81c1305397e904066bb8395a8]

### Features
+ Horizontally scroll across logs. By default use `←` & `→` keys to traverse horizontally across the lines when logs panel selected. Updated `config.toml` with `log_scroll_forward` and `log_scroll_back`  [c190f0206cc55b8e45b8373f9be954e828c18b3b], [8939ac0345326633e794cc10a981a1f3c5c07549]
+ Force clear screen & redraw of UI. By default uses `f` key, `config.toml` updated with `force_redraw`  [50edbc0cc09db864835fe81a03cba8eadafe548b]
+ Increase scroll speed using the `ctrl` key in conjuction with a scroll key, `config.toml` updated with `scroll_modifier`. The next release will remove `scroll_down_many` & `scroll_down_up` keys, [c5bbffdb5f9e800951e4060aa6aee8e00db589aa]

### Refactors
+ remove macos cfg none-const functions, Zigbuild now uses Rust 1.87.0, [eb686e2c952e04da74b3e12c0bfa015ec4615e1d]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
