### 2023-01-29

### Chores
+ dependencies updated, [c129f474fe2976454b1868d00e8d7d99b87ec23b], [9788b8afd98e59b1d4412a8adc54b34d2c5671fd], [2ab88eb26e9bbbc4dad4651256d8d9b044ea3272]

### Docs
+ comment typo, [1025579138f11e4987263c7bfe936c4c8542f8b3]

### Fixes
+ deadlock on draw logs when no containers found, [68e444bfc393eb46bac2b99eb57697bb9b0451af]
+ github workflow release on main only (with semver tag), [e4ca41dfd8ec3acae202a2d2464b8e18f5c5bdd5], [749ec712f07cff2c941aed6726c56bdbd5cb8d2c]

### Refactors
+ major refactor of internal data handling, [b4488e4bdb0252f5c5680cee6a46427f22a282ab]
+ needless (double) referencing removed, [a174dafe1b05908735680a874dc551a86da24777]
+ app_data methods re-ordered & renamed, [c0bb5355d6a5d352260655110ce3d5ab695acda9]

### Reverts
+ is_running AtomicBool back to SeqCst, [c4d80061dab94afd08d4d793dc147f878c965ad6]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
