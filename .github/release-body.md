### 2023-02-04

### Chores
+ devcontainer.json updated, typos-cli installed, temporary(?) buildkit fix, [3c6a8db6ef74d499b49fabe8912785cac16d9c4b]
+ create_release.sh check for typos, [310a63f4cabaa374797a7e4ed0d7fd1f5e79c8fe]

### Docs
+ AUR install instructions, thanks [orhun](https://github.com/orhun), [c5aa346bca139cc5ece1f4127293977924d16fca]
+ typos fixes, thanks [kianmeng](https://github.com/kianmeng), [5052d7ab0a156c43cadbd922c0019b284f24943a]
+ Readme.md styling tweak, [310a63f4cabaa374797a7e4ed0d7fd1f5e79c8fe]
+ Contributing guide, [5aaa00d6a3c58d98cb250b7b14584238df02961c], [a44b15f76088561a0e272d4e7456197c2aaabdb4]

### Features
+ Use a scratch container for the docker image, should reduce image size by around 60%. This checks for the ENV `OXKER_RUNTIME=container`, which is automatically set by the docker image, [17b71b6b41f6a98a0f92277f40a88f4b1b8a1328]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
