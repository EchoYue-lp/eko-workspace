# abdd 手验包的重建配方（发现缺陷后使用）

**用途**：用户在 79 项手验中发现缺陷 → Agent 定位并修 → 需要**再出一个可手验的包**时，按本文件复现同一构建条件，不重新摸索。

**状态**：本文件只是从构建记录里提取的事实。**Agent 未执行、也不会自行执行**——重建一个交付包需要单独授权。

事实来源：`.echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/entry.json`、`.echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/evidence/actual-build-process-start.json`、`.echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/evidence/actual-tauri-build-run.json`。

---

## 一、实际发生过的那次构建

| 项 | 值 |
|---|---|
| 自然退出 | 0 |
| 耗时 | 1992.799 秒（约 33 分钟） |
| 并行 | `CARGO_BUILD_JOBS=1` |
| 编译器警告 | 0 |
| 前端产物 | 149 个 asset |
| 产物主二进制 | `abdd3fac…`（176,890,208 字节） |
| 当时的 target | `latest-formal-arm64-App-20261103-r01/cargo-target`（warm；**该目录已于 2026-11-04 11:33 本地时间被官方 clean 删除**） |

## 二、冻结输入（缺一不可，不用系统默认）

| 输入 | 绝对路径 |
|---|---|
| Rust 工具链 | `.echo-semantic/local/root-rust-toolchain-20261029/rust-1.99.1-frozen/bin/` |
| Tauri CLI | `.echo-semantic/local/frozen-tauri-cli-2.12.4-20261102-r01/unpacked/node_modules/@tauri-apps/cli` |
| target 目录 | `.echo-semantic/local/latest-formal-arm64-App-20261103-r01/cargo-target` —— **已不存在**（2026-11-04 03:33Z 按官方 `cargo clean --target-dir <exact>` 清空，`Removed 6333 files, 3.7GiB`，见 `validation-stage-maintenance-20261104-r01/three-target-cleanup-actual-results.json`）。重建时为**冷构建** |
| npm 缓存 | `.echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/npm-cache`（构建时使用，离线） |

## 三、真实 cargo argv（原样照抄，不要"顺手修正"）

```
<cargo> build --bin echo-agent-tauri --locked --no-default-features \
  -p echo-agent-cli --verbose --bins --features gui,gui,tauri/custom-protocol \
  --release --target aarch64-apple-darwin
```

注意：`--features` 里 `gui` 出现两次，这是**当时实际执行的 argv**，不是誊写错误。复现时应保持原样；若认为该去重，属另一项决定，需先说明会不会改变 feature 闭包。

## 四、构建环境（关键项）

```
CARGO_BUILD_JOBS=1
CARGO_INCREMENTAL=0
CARGO_NET_OFFLINE=true
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_RELEASE_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_TERM_COLOR=never
CARGO_TARGET_DIR=/…/latest-formal-arm64-App-20261103-r01/cargo-target
CARGO_HOME=<按当时记录>
RUSTC / RUSTDOC / RUSTUP_HOME=<冻结工具链绝对路径>
RUSTUP_TOOLCHAIN=eko-native-1.99.1
PATH 前置冻结工具链 bin
```

`RUSTC`／`RUSTDOC` 必须是**绝对路径**。历史教训：一旦让 cargo 走 PATH 上的 rustup 代理，依赖自带的 `rust-toolchain.toml` 会触发下载别的版本并被沙箱拒绝，构建会在与源码无关的地方失败。

## 五、前端不是可选项

Tauri 的 `beforeBuildCommand` 会执行：

```
cd ../web-frontend && npm run build:tauri      # = tsc -b && vite build --mode tauri
```

**不要跳过、不要复用旧 `dist`**。工作树里的 `web-frontend/dist`（mtime 2026-11-02 12:31）是更早的构建，**不含** C02/C04 的修复文案。上次构建是在 `.echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/snapshot/echo-agent-cli/` 里跑的这条命令，产物为 `dist/assets/index-DskFRksZ.js`。

## 六、产出之后还有四步

构建出 `.app` 只是第一步，后面按顺序：

1. **保全到 target 外**（`retain-repair.py`）——Rust 源、程序、FP、`.d`、raw 日志必须先落盘，再谈清理；
2. **本机 ad-hoc 签名**（`--no-sign` 构建后补签）；
3. **`codesign --deep --strict` 严格校验**（历史坑：`codesign … | head` 后取 `$?` 拿到的是 `head` 的退出码，必须单独捕获）；
4. **重新生成双击入口**（`finalize-repair.py` / 启动脚本）。

**注意**：现有 `启动修复版.command` 钉死了 `ready.json` 的 sha256 `17f949cb…`。重建后 `ready.json` 会变，**旧入口会拒绝启动新包**——必须同步重新生成入口，或明确接受"入口与包不匹配"。

## 七、与用户数据的关系

- 构建**不读取、不散列、不恢复**用户配置（记录里 `userConfigurationReadHashRestore=false`）；
- 启动脚本把 `EKO_DATA_DIR`／配置／MCP／项目 argv 当**路径字符串**传递，不解析内容；
- 重装包不会改动 `isolated-runtime/data` 里的会话、工作空间、渠道配置。

## 八、成本与前置

- **1992.8 秒（约 33 分钟）是在 warm target 上测得的**；该 target 已于 2026-11-04 清空，故重建是从零开始的**冷构建**，耗时会明显长于 33 分钟，不要用旧数字做时间预期；
- 需要**独占 Cargo 窗口**（AGENTS：同机最多五条 focused 批次，jobs ≤5；本项 jobs=1，但仍属重型批次）；
- 需要先确认没有其他 cargo/rustc 在跑、磁盘可用空间足够（当前 26 GiB，够）；
- 旧的 warm target 已被官方 clean（非本次主张），重建属于冷构建；完成后按 AGENTS 的批次规则处理新 target，不要顺手清别的 target。

## 九、这份配方不能替代什么

- 不能替代 canonical／Source-Final 资格（那需要联网，当前 GitHub 443 超时）；
- 不能替代 79 项 GUI 验收；
- 不能因为"能重建"就认为可以删除现有包——现有包是**唯一**已知含三项修复且已核身份的手验制品。
