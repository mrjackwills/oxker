### 2023-01-21

### Chores
+ dependencies updated, [8cd199db49186fad6ce432bb277e3a10f0a08d34], [d880b829c123dbe57deccadef97810e45c083737], [66d57c99558ca14d9593d6dbfd5b0e8e5d59055d], [33f9374908942f4a3b90be227fad94ca353cf351], [007d5d83d7f1b93e1e78777a4417b2740db706bd]
+ create_release.sh typos, [9a27d46a044452080144ee1367dc95886b10abf8]
+ dev container post create install cross, [2d253f034182741d434e4bac12317f24221d0d4a]

### Features
**all potentially considered breaking changes**
+ store Logs in own struct, use a hashset to track timestamps, hopefully closes #11, [657ea2d751a71f05b17547b47c492d5676817336]
+ Spawn docker commands into own thread, can now execute multiple docker commands at the same time, [9ec43e124a62a80f4e78acba85fc3af5980ce260]
+ align memory columns correctly, minimum byte display value now `0.00 kB`, rather than `0 B`, closes #20, [bd7dfcd2c512a527d66a1388f90006988a487186], [51c580010a24de2427373795803936d498dc8cee]

### Refactors
+ main.rs tidy up, [97b89349dc2de275ca514a1e6420255a63d775e8]
+ derive Default for GuiState, [9dcd0509efeb464f58fb53d813bd78de2447949d]
+ param reduction, AtomicBool to Relaxed, [0350293de3c00c6e5e5d787b7596bb3413d1cda1]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
