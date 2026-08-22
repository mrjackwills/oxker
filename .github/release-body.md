### 2026-08-22

### Chores
+ Rust version bump to 1.95.0, [0a6e30f154970c8d2af19e1f73c72705995146aa]
* dependencies updated, [dcd2d699ddcae9d905379b808e61888352570d1d], [907774f3ddbd51604875fc5c1873468547dd9f94]

### Docs
+ workflow links added, [baa575fac288176cc693376b1330979bbe26c873]

### Features
+ use future::streams instead of tokio::spawns, [7b55f9b98a45f20cc74bea3478fc746395084495]

### Fixes
+ container sort, closes #89, [a386e24633cb47eb9621cba42481c02ff25c0390]
+ colour parser typo, [fe38ad170ec7f54ac8e4ccc4a1c4adec5b152b1a]
+ format_log_line account for multiple spans, [2320cb527cc4992496c582eeab20b4f192624a97]
+ input_handle quit, [31fd3a18e0e0247ce9af6963bbed017470996d7c]
+ update_summaries remove fix, [9d1cbed5e51720c489ecf578760375504f9b05d0], [f99220924e9267b020831623ba3ada122a14ca97]

### Refactors
+ appdata fn params, [dcb036580138ac2a0c0bff682e3cc015897dda60]
+ eth0 const, [ce1c568f77b53b62c5bedb25428dda48220e2b51]
+ health removed from container summary, [6b79ce6303476f9ea16ba0907f6a179224bb192b]
+ input_handler quit(), [dc5c14135312f5a4c8d41f79b47009797ca78011]
+ simplify show_self config, [755b9fbcecc3d7190b8f73290865fe6b727cebf7]
+ use a STATS_MAX const, [c260ddd8d9563c9377271bb62ef01f590f06eaf5]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
