### 2022-09-07

### Chores
+ dependencies updated, [a3168daa3f769a6747dfbe61103073a7e80a1485], [78e59160bb6a978ee80e3a99eb72f051fb64e737]

### Features
+ containerize self, github action to build and push to [Docker Hub](https://hub.docker.com/r/mrjackwills/oxker), [07f972022a69f22bac57925e6ad84234381f7890]
+ gui_state is_loading use a HashSet to enable multiple things be loading at the same time, [66583e1b037b7e2f3e47948d70d8a4c6f6a2f2d5]
+ github action publish to crates.io, [90b2e3f6db0d5f63840cd80888a30da6ecc22f20]
+ derive Eq where appropriate, [d7c2601f959bc12a64cd25cef59c837e1e8c2b2a]
+ ignore containers 'oxker' containers, [1be9f52ad4a68f93142784e9df630c59cdec0a79]
+ update container info if container is either running OR restarting, [5f12362db7cb61ca68f75b99ecfc9725380d87d2]

### Fixes
+ devcontainer updated, [3bde4f5629539cab3dbb57556663ab81685f9d7a]
+ Use Binate enum to enable two cycles of cpu/mem update to be executed (for each container) at the same time, refactor hashmap spawn insertions, [7ec58e79a1316ad1f7e50a2781dea0fe8422c588]

### Refactors
+ improved way to remove leading '/' of container name, [832e9782d7765872cbb84df6b3703fc08cb353c9]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
