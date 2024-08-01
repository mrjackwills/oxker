### 2024-08-01

### Chores
+ .devcontainer extensions updated, [0288cbc8146cde1dd40ceaec9550198b635bb8f5]
+ dependencies updated, [1df4f78dc41013c33d901925933b1ccb29ad4bc8], [5ae253b8734ba0495e4e8149b17d5228b3d86f8d], [7a517db9f7c14c35e56ff70cf76ffb608fd30e17], [9c291cd9c81b6d9a02085878588ed3b845fd0046], [0e90f4eb55ac5fb5d45e7d212c3686027dd3913e], [fe71cbfb00f166b7c02a6e28e64650ed1b47d15d]
+ docker-compose alpine version bump, [51ceab3ebdb09356cd401d2f268840239255126f]
+ Rust 1.80 linting, [93e1279b1fc77019442a385e2e36be2fe438e828]
+ create_release v0.5.6, [f408acfe9a9f5a976735b8a8a51500fd7b865daf]

### Docs
+ screenshot updated, [6975ebe70f7058229c232e4a56b090f55247d2a2]

### Features
+ left align all text, [e0d421c4918a17c9e0e21fd214edb99d71281c9d]
+ place image name in logs panel title, [12f24357a68abe871f44d871d95b6e2ef062181e]
+ distinguish between unhealthy & healthy running containers, closes #43, [de8768181631c6d961ce0e4dacb50c2ed02abc36]
+ filter containers, use `F1` or `/` to enter filter mode, closes #37, thanks to [MohammadShabaniSBU](https://github.com/MohammadShabaniSBU) for the original PR, [d5d8a0dbc5437ff3b17f34b9dbb9589bb56b4a3e], [[7ee1f06f804683e3395953a02138d4e9da115ea9]]
+ place image name in logs panel title, [ef19b9cf89a881d0a7ac818885317ce2bd683dfc]

### Fixes
+ log_sanitizer `raw()` & `remove_ansi()` now functioning as intended, [0dc98dfc8113869b81be9d697ca77418c919e4bf]
+ Dockerfile command use uppercase, [068e4025a5d6049a9a6951a0480a6bdef7379f88]
+ heading section help margin, [0e927aae178c1d8f60561b93607a26d45a1d9331]
+ install.sh use curl, [197a031b8cf356f49f08e04472d0d1c489699415]

### Tests
+ fix layout tests with new left alignment, [dfced564278eafdbb8a5b95badbae3a7c4bf87b3]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
