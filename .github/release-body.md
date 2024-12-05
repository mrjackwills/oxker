### 2024-12-05

### Chores
+ dependencies updated, [b78713579c4706d605e5b35fcd832610a0152294], [c6200e8f77f8bb1f0152cb9374029d15cc45df9d]
+ Rust 1.83 linting, [751d997a3dac823e144ae62e6c1455676e50ddb8]

### Features
+ `--no-stderr` cli arg, removes Standard error output from logs, closes #52, [c739637b91c8fa742a69f4d888678d7b3964678c]
+ ContainerPorts use ipaddr, [1b26997d25f748e0d452f41fe41791533046ecdf]

### Fixes
+ update containerised Dockerfile, [0c6f53228f01196e352c2069383ba1e7a10950a8]
+ calculate_usage overflow, [5106a01f3dcb87ce5a8f1fb7bf49dc6b3c25d03e]
+ DockerData spawns insertion error, [d4906d33c26b75d92e7d80040c488faa90a257c6]

### Refactors
+ speed up docker logs init process, [8b9fe4246865441704ae12dff0938868a4fe6f81]
+ remove docker sleep, [f1562d1084336fe5be39894c93cb49107f0a4a6d]
+ dead code removed, [5ee48d5708fa6de0206c021db0bb611196e66fba], [ba6a95241389f99d504ee4bf3e87e19006f12e49], [f0b1145651625ad4e577d79baaf902d4d3bc0579]
+ input_handler, [7f4238349525c01ae9fb8b1f6c0946e5364dd55e]
+ statefulList get_state_title, [2d540b0e2210cc04d73035ec59211ffc739174f6]
+ statefulList next/previous, [7bb2bef28d90ebc58da86a0365a1904a0c32dffe]
+ help_box closure fn, [2860426d57a4458fcee49a2fd20e8e7bb9e71fb5]
+ use check_sub for sleep calculations, [fe3696e5576739d8b033d9e748b5ea696c4b4e4f]
+ rename scheduler to heartbeat, [68a6551ed038a36330b2f098112829465a1c3c7a]
+ remove unnecessary is_running load, [76ccf7c00691f815c3ab0bede838c99252ba84f0]
+ execute_command(), [2a834d6c2fa4a15124d24ddbd12f667829e148ad]
+ Remove numerous clones(), [e5927f781a7e9517b9fa00a2d1a835d2774a9d26]
+ remove app_data param from generate_lock(), [1a8dab654a1fdbf351a72dc54fe3d1943355bba6]
+ combine get_filter methods, [356ea5549bb4877e9893fe0e1053e73c5a62e806]
+ FrameData refactors, [57781701ff14c553dfbafb965ee8a33ab44dd36f], [6e2f82db81caaa98ce4781fa15928eb9e246ace6]
+ update_container_stat combine is_alive(), [55cc746736f6863aedc5ad838744a983796244d8]
+ remove `input_poll_rate` from `Ui`, instead use const `POLL_RATE`, [69f6c96b700b9fde5578ae204992a67986d456ab]
+ pass `&FrameDate` into `draw_frame()`, [35aec5060fdbe606267be26656b4aeee43d50c02]
+ dead code removed, [caf23be4a7faff99aaca80b081a02e4e0a372009]
+ input_handler, [9c4f8910381b90b563da12eaba4b79cb60c40129]
+ draw_block, [de76bc22936b124dcb9646f302f6cc14691dbb63]

### Tests
+ fix logs tests, [9b22f5da18e4bf92766a68a7f4cd61ad72724cfd]

see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
