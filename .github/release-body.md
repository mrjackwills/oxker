### 2025-09-28

### Chores
+ create_release.sh updated, [d4af754ad245540db60177f7b202b3c64519c961]
+ dependencies updated, [03599b46657d38d0c9f25c2ccfd9510f2b98dd84], [aef0c9503e7045a256856aa887d8c8d7722b9936], [f0771eab5d07d141fe7a8997db650f0f65ffe0a7], [1596de8681ad6c0a7832eb922dd2dc36ab30eb41]
+ GitHub workflow updated, [66dae5e61ea294ac8ce134a6c32b27c04166b6eb]

### Docs
+ fix numerous typos, [618a43b501914fdf2659e171172ad180364cf87a]

### Features
+ *BREAKING CHANGE* - `scroll_down_many` & `scroll_up_many` removed, `scroll_down_one` `scroll_up_one` renamed `scroll_down`, `scroll_up`, see [example_config](https://github.com/mrjackwills/oxker/tree/main/example_config), [52a04ec1d0b9e4877e304f60a857ebc00f88b4fd]
+ log search feature, closes #72. Use `#` button, remappable via `log_search_mode`, to enter log search mode. Case-sensitive by default, editable in `config.toml` with `log_search_case_sensitive` entry. Customise colours via `[colors.log_search]` entries, again see see [example_config](https://github.com/mrjackwills/oxker/tree/main/example_config), [96d9469623a7c90b79aa8d82abf587290343ad37], [a2316a9cac270790920a1ebd1be6532d51aba77c]
+ `term` renamed `filter term`, tests updated, [487c3faf96f4c197c8b82644c02466ea40626a5e]

My 32-bit armhf armv6 hardware no longer seems to be able to run Docker. Future `oxker` releases won't be tested on real hardware but will continue to be built and published for armv6.

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
