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

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.1'>v0.2.1</a>
### 2023-01-29

### Chores
+ dependencies updated, [c129f474](https://github.com/mrjackwills/oxker/commit/c129f474fe2976454b1868d00e8d7d99b87ec23b), [9788b8af](https://github.com/mrjackwills/oxker/commit/9788b8afd98e59b1d4412a8adc54b34d2c5671fd), [2ab88eb2](https://github.com/mrjackwills/oxker/commit/2ab88eb26e9bbbc4dad4651256d8d9b044ea3272)

### Docs
+ comment typo, [10255791](https://github.com/mrjackwills/oxker/commit/1025579138f11e4987263c7bfe936c4c8542f8b3)

### Fixes
+ deadlock on draw logs when no containers found, [68e444bf](https://github.com/mrjackwills/oxker/commit/68e444bfc393eb46bac2b99eb57697bb9b0451af)
+ github workflow release on main only (with semver tag), [e4ca41df](https://github.com/mrjackwills/oxker/commit/e4ca41dfd8ec3acae202a2d2464b8e18f5c5bdd5), [749ec712](https://github.com/mrjackwills/oxker/commit/749ec712f07cff2c941aed6726c56bdbd5cb8d2c)

### Refactors
+ major refactor of internal data handling, [b4488e4b](https://github.com/mrjackwills/oxker/commit/b4488e4bdb0252f5c5680cee6a46427f22a282ab)
+ needless (double) referencing removed, [a174dafe](https://github.com/mrjackwills/oxker/commit/a174dafe1b05908735680a874dc551a86da24777)
+ app_data methods re-ordered & renamed, [c0bb5355](https://github.com/mrjackwills/oxker/commit/c0bb5355d6a5d352260655110ce3d5ab695acda9)

### Reverts
+ is_running AtomicBool back to SeqCst, [c4d80061](https://github.com/mrjackwills/oxker/commit/c4d80061dab94afd08d4d793dc147f878c965ad6)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.2.0'>v0.2.0</a>
### 2023-01-21

### Chores
+ dependencies updated, [8cd199db](https://github.com/mrjackwills/oxker/commit/8cd199db49186fad6ce432bb277e3a10f0a08d34), [d880b829](https://github.com/mrjackwills/oxker/commit/d880b829c123dbe57deccadef97810e45c083737), [66d57c99](https://github.com/mrjackwills/oxker/commit/66d57c99558ca14d9593d6dbfd5b0e8e5d59055d), [33f93749](https://github.com/mrjackwills/oxker/commit/33f9374908942f4a3b90be227fad94ca353cf351), [007d5d83](https://github.com/mrjackwills/oxker/commit/007d5d83d7f1b93e1e78777a4417b2740db706bd)
+ create_release.sh typos, [9a27d46a](https://github.com/mrjackwills/oxker/commit/9a27d46a044452080144ee1367dc95886b10abf8)
+ dev container post create install cross, [2d253f03](https://github.com/mrjackwills/oxker/commit/2d253f034182741d434e4bac12317f24221d0d4a)

### Features
**all potentially considered breaking changes**
+ store Logs in own struct, use a hashset to track timestamps, hopefully closes [#11](https://github.com/mrjackwills/oxker/issues/11), [657ea2d7](https://github.com/mrjackwills/oxker/commit/657ea2d751a71f05b17547b47c492d5676817336)
+ Spawn docker commands into own thread, can now execute multiple docker commands at the same time, [9ec43e12](https://github.com/mrjackwills/oxker/commit/9ec43e124a62a80f4e78acba85fc3af5980ce260)
+ align memory columns correctly, minimum byte display value now `0.00 kB`, rather than `0 B`, closes [#20](https://github.com/mrjackwills/oxker/issues/20), [bd7dfcd2](https://github.com/mrjackwills/oxker/commit/bd7dfcd2c512a527d66a1388f90006988a487186), [51c58001](https://github.com/mrjackwills/oxker/commit/51c580010a24de2427373795803936d498dc8cee)

### Refactors
+ main.rs tidy up, [97b89349](https://github.com/mrjackwills/oxker/commit/97b89349dc2de275ca514a1e6420255a63d775e8)
+ derive Default for GuiState, [9dcd0509](https://github.com/mrjackwills/oxker/commit/9dcd0509efeb464f58fb53d813bd78de2447949d)
+ param reduction, AtomicBool to Relaxed, [0350293d](https://github.com/mrjackwills/oxker/commit/0350293de3c00c6e5e5d787b7596bb3413d1cda1)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.11'>v0.1.11</a>
### 2023-01-03

### Chores
+ dependencies updated, [9b09146a](https://github.com/mrjackwills/oxker/commit/9b09146aadae5727a5fee4de5fe0c1d70c581c22)

### Features
+ `install.sh` script added, for automated platform selection, download, and installation, [7a42eba6](https://github.com/mrjackwills/oxker/commit/7a42eba6b0968314af40ff87bcc42d288f6860bc), [e0703b76](https://github.com/mrjackwills/oxker/commit/e0703b76a1a28cfe266f130a7f7dec92f1b5ad58)

### Fixes
+ If a sort order is set, sort containers on every `update_stats()` execution, [cfdea775](https://github.com/mrjackwills/oxker/commit/cfdea77594e48c8c20a4d6e6c7ea31c9181361a1)

### Refactors
+ input sort executed in app_data struct `sort_by_header()`, [3cdc5fae](https://github.com/mrjackwills/oxker/commit/3cdc5fae02097628799209f371ae9292e513e76c)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.10'>v0.1.10</a>
### 2022-12-25

### Chores
+ dependencies updated, [1525b315](https://github.com/mrjackwills/oxker/commit/1525b3150293015c0fb2f2161da463b21ac2694c), [8d539ab1](https://github.com/mrjackwills/oxker/commit/8d539ab14809136d743c49d60779687fc8eeef6d), [1774217a](https://github.com/mrjackwills/oxker/commit/1774217a8a657d261397d213e5ecee667cf3b6b1)
+ Rust 1.66 linting, [bf9dcac7](https://github.com/mrjackwills/oxker/commit/bf9dcac7045c0d2314df147ec2744a3ad886564b)

### Features
+ Caching on github action, [a91c9aa4](https://github.com/mrjackwills/oxker/commit/a91c9aa45ffd5c998cd1b83d8e90d0912893c31f)

### Fixes
+ comment typo, [7899b773](https://github.com/mrjackwills/oxker/commit/7899b773569fed86343a035d3023bf34297fdabb)

### Refactors
+ remove_ansi() to single liner, [57c3a6c1](https://github.com/mrjackwills/oxker/commit/57c3a6c186b916faba24bf7b5cdbbda31d636a7e)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.9'>v0.1.9</a>
### 2022-12-05

### Fixes
+ disallow commands to be sent to a dockerised oxker container, closes [#19](https://github.com/mrjackwills/oxker/issues/19), [160b8021](https://github.com/mrjackwills/oxker/commit/160b8021b1de898064756b53c127d49b8096ce4d)
+ if no container created time, use 0, instead of system_time(), [1adb61ce](https://github.com/mrjackwills/oxker/commit/1adb61ce3b029d4fcf51961958d483b2fae8825a)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.8'>v0.1.8</a>
### 2022-12-05

### Chores
+ dependencies updated, [e3aa4420](https://github.com/mrjackwills/oxker/commit/e3aa4420cb510df0381e311d37e768937070387a)
+ docker-compose.yml alpine bump, [911c6596](https://github.com/mrjackwills/oxker/commit/911c6596684db4ccbe7a55aadd6f595a95f89bb0)
+ github workflow use dtolnay/rust-toolchain, [57c18878](https://github.com/mrjackwills/oxker/commit/57c18878690477a05d7330112a65d1d58a07901e)

### Features
+ Clicking a header now toggles between Ascending -> Descending -> Default. Use the containers created_time as the default order - maybe add created column in future version, closes [#18](https://github.com/mrjackwills/oxker/issues/18), [cf14ba49](https://github.com/mrjackwills/oxker/commit/cf14ba498987db587c0f5bef8a67cf4113ffcb1e), [d1de2914](https://github.com/mrjackwills/oxker/commit/d1de291473d8a1028f1936429832d3820d75df54)
+ `-s` flag for showing the oxker container when executing the docker image, [c93870e5](https://github.com/mrjackwills/oxker/commit/c93870e5fbbc7df35c69d32e4460d2104e521e33)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.7'>v0.1.7</a>
### 2022-11-13

### Chores
+ update dependencies, closes [#17](https://github.com/mrjackwills/oxker/issues/17), [8a5d0ef8](https://github.com/mrjackwills/oxker/commit/8a5d0ef8376e3739dda5b0ed4c3e75e565deed45), [eadfc3d6](https://github.com/mrjackwills/oxker/commit/eadfc3d6c6896ecc8cff88c6a9e9c8b3e477c0cd)
+ aggressive linting with Rust 1.65.0, [8f3a1513](https://github.com/mrjackwills/oxker/commit/8f3a15137155dc374e6b2822c9155c07d05d5e28)

### Docs
+ README.md improved Download & Install section, and now available on [NixPkg](https://search.nixos.org/packages?channel=unstable&show=oxker&from=0&size=50&sort=relevance&type=packages&query=oxker), thanks [siph](https://github.com/siph), [67a9e183](https://github.com/mrjackwills/oxker/commit/67a9e183ca04199da758255075ff7e73061eb850)

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.6'>v0.1.6</a>
### 2022-10-16

### Chores
+ Cargo update, [c3e72ae7](https://github.com/mrjackwills/oxker/commit/c3e72ae7369a25d903f39e55a4349cb005671dd4),
+ create_release.sh v0.1.0, [3c8d59c6](https://github.com/mrjackwills/oxker/commit/3c8d59c666bd4cda9ca54989b2f1b48bba17bc57),
+ uuid updated to version 1.2, [438ad770](https://github.com/mrjackwills/oxker/commit/438ad770f4a5ecb5f4bbc308066ad9e808f66514),

### Fixes
+ loading icon shifting error fix, also make icon white, closes [#15](https://github.com/mrjackwills/oxker/issues/15), [59797685](https://github.com/mrjackwills/oxker/commit/59797685dffa29752a48c98e6cf465884d6d9df6),

### Features
+ Show container name in log panel title, closes [#16](https://github.com/mrjackwills/oxker/issues/16), [9cb0c414](https://github.com/mrjackwills/oxker/commit/9cb0c414afc284947fc2b8494504387e4e7edd87),
+ use gui_state HashSet to keep track of application gui state, [9e9d5155](https://github.com/mrjackwills/oxker/commit/9e9d51559a13944622abf4fcbd3bd63766d11467),
+ terminal.clear() after run_app finished, [67c49575](https://github.com/mrjackwills/oxker/commit/67c49575682cb271fac0998ff377a6504cd0bc86),

### Refactors
+ CpuStats & MemStats use tuple struct, [a060d032](https://github.com/mrjackwills/oxker/commit/a060d032586a0707ac91cb13d922aae0850449c5),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.5'>v0.1.5</a>
### 2022-10-07

### Chores
+ Update clap to v4, [15597dbe](https://github.com/mrjackwills/oxker/commit/15597dbe6942ec053541398ce0e9dedc10a4d3ea),

### Docs
+ readme.md updated, [a05bf561](https://github.com/mrjackwills/oxker/commit/a05bf561cc6d96237f683ab0b3c782d6841974d9),

### Features
+ use newtype construct for container id, [41cbb84f](https://github.com/mrjackwills/oxker/commit/41cbb84f2896f8be2c37eba87e390d998aff7382),

### Refactors
+ Impl Copy where able to, [e76878f4](https://github.com/mrjackwills/oxker/commit/e76878f424d72b943713ef84e95e25fada77d79e),
+ replace async fn with just fn, [17dc604b](https://github.com/mrjackwills/oxker/commit/17dc604befac75cb9dc0311a0e43f9927fe0ca30),
+ remove pointless clone()'s & variable declarations, [6731002e](https://github.com/mrjackwills/oxker/commit/6731002ee42c9460042c2c38aff5101b1bcebbe6),
+ replace String::from("") with String::new(), [62fb2247](https://github.com/mrjackwills/oxker/commit/62fb22478697cc9a7ab9fb562a724965b437233a),
+ replace map_or_else with map_or, [3e26f292](https://github.com/mrjackwills/oxker/commit/3e26f292c7dc5e13af4580952767ebe821aa5183), [5660b34d](https://github.com/mrjackwills/oxker/commit/5660b34d5149dce27706ff6daa90b854e6f84e14),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.4'>v0.1.4</a>
### 2022-09-07

### Chores
+ dependencies updated, [a3168daa](https://github.com/mrjackwills/oxker/commit/a3168daa3f769a6747dfbe61103073a7e80a1485),[78e59160](https://github.com/mrjackwills/oxker/commit/78e59160bb6a978ee80e3a99eb72f051fb64e737),

### Features
+ containerize self, github action to build and push to [Docker Hub](https://hub.docker.com/r/mrjackwills/oxker), [07f97202](https://github.com/mrjackwills/oxker/commit/07f972022a69f22bac57925e6ad84234381f7890),
+ gui_state is_loading use a HashSet to enable multiple things be loading at the same time, [66583e1b](https://github.com/mrjackwills/oxker/commit/66583e1b037b7e2f3e47948d70d8a4c6f6a2f2d5),
+ github action publish to crates.io, [90b2e3f6](https://github.com/mrjackwills/oxker/commit/90b2e3f6db0d5f63840cd80888a30da6ecc22f20),
+ derive Eq where appropriate, [d7c2601f](https://github.com/mrjackwills/oxker/commit/d7c2601f959bc12a64cd25cef59c837e1e8c2b2a),
+ ignore containers 'oxker' containers, [1be9f52a](https://github.com/mrjackwills/oxker/commit/1be9f52ad4a68f93142784e9df630c59cdec0a79),
+ update container info if container is either running OR restarting, [5f12362d](https://github.com/mrjackwills/oxker/commit/5f12362db7cb61ca68f75b99ecfc9725380d87d2),

### Fixes
+ devcontainer updated, [3bde4f56](https://github.com/mrjackwills/oxker/commit/3bde4f5629539cab3dbb57556663ab81685f9d7a),
+ Use Binate enum to enable two cycles of cpu/mem update to be executed (for each container) at the same time, refactor hashmap spawn insertions, [7ec58e79](https://github.com/mrjackwills/oxker/commit/7ec58e79a1316ad1f7e50a2781dea0fe8422c588),

### Refactors
+ improved way to remove leading '/' of container name, [832e9782](https://github.com/mrjackwills/oxker/commit/832e9782d7765872cbb84df6b3703fc08cb353c9),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.1.3'>v0.1.3</a>
### 2022-08-04

### Chores
+ dependencies updated, [d9801cdf](https://github.com/mrjackwills/oxker/commit/d9801cdf372521fe5624a8d68fac83ed39ef81f4),
+ linting: nursery, pedantic, unused_unwraps, [1bd61d4c](https://github.com/mrjackwills/oxker/commit/1bd61d4ce8b369d6d078201add3eea0f59fe0dea), [1263662b](https://github.com/mrjackwills/oxker/commit/1263662bd9412afacddbc10721bf216ae3a843f1), [ca3315a6](https://github.com/mrjackwills/oxker/commit/ca3315a69f593ad705eb637f227f195edd7781b2),

### Features
+ build all production targets on release, [44f8140e](https://github.com/mrjackwills/oxker/commit/44f8140eaec330abe5a94f3ddae9e8b223688aa8),

### Fixes
+ toml keywords, [dd2d82d1](https://github.com/mrjackwills/oxker/commit/dd2d82d114537e09dbeb12f360157f0e68e7846e),

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
+ allow toggling of mouse capture, to select & copy text with mouse, closes [#2](https://github.com/mrjackwills/oxker/issues/2), [aec184ea](https://github.com/mrjackwills/oxker/commit/aec184ea22b289e91942a4c3e6a415685884bc47),
+ show id column, [b10f9274](https://github.com/mrjackwills/oxker/commit/b10f927481c9e38a48c1d4b94e744ec48e8b6ba6),
+ draw_popup, using enum to draw in one of 9 areas, closes [#6](https://github.com/mrjackwills/oxker/issues/6), [1017850a](https://github.com/mrjackwills/oxker/commit/1017850a6cc91328abc1127bdb117495f5e909d8),
+ use a message rx/sx for all docker commands, remove update loop, wait for update message from gui instead, [9b70fdfa](https://github.com/mrjackwills/oxker/commit/9b70fdfad7b38361ebee301bdc2545d3f0dfcf9e),

### Fixes
+ readme.md typo, [589501f9](https://github.com/mrjackwills/oxker/commit/589501f9a4a0bfabdb0654e68cc0c752c529d97a),
+ column heading mem > memory, [5e8e6b59](https://github.com/mrjackwills/oxker/commit/5e8e6b590b06f01a542fdd10bae8f14d303ab08a),
+ cargo fmt added to create_release.sh, [bb29c0eb](https://github.com/mrjackwills/oxker/commit/bb29c0ebfafd6a9a036eb317a240954d1405966e),

# <a href='https://github.com/mrjackwills/oxker/releases/tag/v0.0.1'>v0.0.1</a>
### 2022-04-25

+ init commit
