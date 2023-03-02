### 2023-03-02

### Chores
+ dependencies updated, [aac3ef2b1def3345d749d813d9b76020d6b5e5ca], [4723be7fb2eb101024bb9d5a514e2c6cc51eb6f6], [c69ab4f7c3b873f25ea46958add37be78d23e9cf], [ba6437862dae0f422660a602aeabd6217d023fac], [2bb4c338903e09856053894d9646307e31d32f1c]
+ dev container install x86 musl toolchain, [e650034d50f01a7598876d4f2887df691700e06a]

### Docs
+ typos removed, [23ad9a5fb3cacf3fb8cb70c65ca9133ed9949e45], [cebb975cb82f653407ec801fd8c726ca6ed68289], [fdc67c9249a239bac97a78b20c9378472865209c]
+ comments improved, [ec962295a8789ff8010604e974969bf618ea7108]

### Features
+ mouse capture is now more specific, should have substantial performance impact, 10x reduction in cpu usage when mouse is moved observed, as well as fixing intermittent mouse events output bug, [0a1b53111627206cc7436589e5b7212e1b72edb8], [93f7c07f708885f8870da5dfb6d57c62f93c9c78], [c74f6c1179b5f62989eb74f395a56b43a8781b03]
+ improve the styling of the help information popup, [28de74b866f07c8543e46be3cab929eff28953fd]
+ use checked_sub & checked_div for bounds checks, [72279e26ae996353c95a75527f704bac1e4bcf4d]

### Fixes
+ correctly set gui error, [340893a860e99ec4029d12613f2a6de3cb7b47e2]

### Refactors
+ dead code removed, [b8f5792d1865d3a398cd7f23aa9473a55dc6ea44]
+ improve the get_width function, [04c26fe8fc7c79506921b9cff42825b1ee132737]
+ place ui methods into a Ui struct, [3437df59884f084624031fceb34ea3012a8e2251]
+ get_horizotal/vertical constraints into single method, [e8f5cf9c6f8cd5f807a05fb61e31d7cd1426486f]
+ docker update_everything variables, [074cb957f274675a468f08fecb1c43ff7453217d]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
