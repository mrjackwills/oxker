### 2023-11-21

### Chores
+ workflow dependencies updated, [6a4cf6490d08b976734e2bc8186d94c095700558]
+ dependencies updated, [e301b51891e03ea40b2f904583119da3bc4daf53], [81d5b326db8881263f2c9072e1426948e41b4a0f], [294cc2684f42daab9d51601e235a384f55617678]
+ lints moved from main.rs to Cargo.toml, [2de76e2f358be9c1500ca3dc4f9df0979ed8ed28]
+ .devcontainer updated, [37d2ee915625806dd11c2cc816a892aae12a777c]

### Features
+ Docker exec mode - you are now able to attempt to exec into a container by pressing the `e` key, closes #28, [c8077bca0b673478cfbb417e677a885136ba9eff], [0e5ee143b008c9d0ee0b681231a1568be227150b], [0e5ee143b008c9d0ee0b681231a1568be227150b]
+ Export logs feature, press `s` to save logs, use `--save-dir` cli-arg to customise output location, closes #1, [a15da5ed43d07852504a4dd1884a189e3f5b9d84]

### Fixes
+ GitHub workflow, cargo publish before create release, [ae4ce3b549c40cc8bd713f375f030b185179a6e2]
+ sorted created_at clash, closes #22, [3a6489396e87702ce94b349a7f47028ece7922f6]
+ `as_ref()` fixed, thanks [Daniel-Boll](https://github.com/Daniel-Boll), [77fbaa8b1669286369b6ec1edd80220c808b628f]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
