### 2023-03-13

### Chores
+ Rust 1.68.0 clippy linting, [5582c45403413d3355bbcd629cfad559296f5e5b]
+ devcontainer use sparse protocol index, [20b79e9cd5bf75bb253158c0b590284139e0291d]
+ dependencies updated, [0c07d4b40607a0eba003b6dcd0345ec0543c6264], [601a73d2c830043a25d64922c4d4aa38f8801912], [5aaa3c1ab08b0c85df9bfce18a3e60206556fa58], [7a1563030e48499da7f41033673c70deefe3de8a], [457157755baa1f9e9cfef9315a7940c357b0953d]

### Features
+ increase mpsc channel size from 16 to 32 messages, [924f14e998f79f731447a2eded038eab51f2e932]
+ KeyEvents send modifier, so can quit on `ctrl + c`, [598f67c6f6a8713102bcc415f0409911763bb914]
+ only send relevant mouse events to input handler, [507660d835d0beaa8cd021110401ecc58c0613c6]

### Fixes
+ GitHub workflow on SEMEVR tag only, [140773865165bf006e74f9d436fc744220f5eae7]

### Refactors
+ replace `unwrap_or(())` with `.ok()`, [8ba37a165bb89277ab957194da6464bdb35be2e6]
+ use `unwrap_or_default()`, [79de92c3921702417bb2df1f44939a7b09cb7fa0]
+ Result return, [d9f0bd5566e27218b8c8eaba6ece237907771c1d]

### Reverts
+ temporary devcontainer buildkit fix removed, [d1497a4451f4de54d3cc26c5a3957cd636c29118]


see <a href='https://github.com/mrjackwills/oxker/blob/main/CHANGELOG.md'>CHANGELOG.md</a> for more details
