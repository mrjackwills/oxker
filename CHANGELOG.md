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
+ allow toggling of mouse caputre, to select & copy text with mouse, closes #2,  [aec184ea](https://github.com/mrjackwills/oxker/commit/aec184ea22b289e91942a4c3e6a415685884bc47),
+ show id column, [b10f9274](https://github.com/mrjackwills/oxker/commit/b10f927481c9e38a48c1d4b94e744ec48e8b6ba6),
+ draw_popup, using enum to draw in one of 9 areas, closes #6, [1017850a](https://github.com/mrjackwills/oxker/commit/1017850a6cc91328abc1127bdb117495f5e909d8),
+ use a message rx/sx for all docker commands, remove update loop, wait for update message from gui instead, [9b70fdfa](https://github.com/mrjackwills/oxker/commit/9b70fdfad7b38361ebee301bdc2545d3f0dfcf9e),

### Fixes
+ readme.md typo, [589501f9](https://github.com/mrjackwills/oxker/commit/589501f9a4a0bfabdb0654e68cc0c752c529d97a),
+ column heading mem > memory, [5e8e6b59](https://github.com/mrjackwills/oxker/commit/5e8e6b590b06f01a542fdd10bae8f14d303ab08a),
+ cargo fmt added to create_release.sh, [bb29c0eb](https://github.com/mrjackwills/oxker/commit/bb29c0ebfafd6a9a036eb317a240954d1405966e),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.1'>v0.0.1</a>
### 2022-04-25

+ init commit
