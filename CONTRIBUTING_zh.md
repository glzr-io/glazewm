# 为 GlazeWM 做贡献

感谢你关注 GlazeWM，并愿意帮助改进它 💛

主要有三种贡献方式：

1. **提交 Issue**。如果发现了缺陷或有功能建议，可以提交 Issue 讨论。
2. **协助整理 Issue**。为已有的 Issue 补充相关细节和建议。
3. **提交 PR**。通过 PR 修复缺陷或实现功能。

如果本指南没有涵盖你的问题，欢迎加入 [Discord 的 #glazewm-dev 频道 ⚡](https://discord.com/invite/ud6z3qjRvM)。

## PR 与开发流程

可以从带有 [`good first issue`](https://github.com/glzr-io/glazewm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) 或 [`help wanted`](https://github.com/glzr-io/glazewm/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) 标签的 Issue 入手。PR 可以独立提交。如果已有对应的 Issue，请留言，维护者可以将它分配给你。

### 搭建开发环境

先复刻仓库，再克隆到本地：

```shell
git clone git@github.com:your-username/glazewm.git
```

如果尚未安装 Rust，请先[安装 Rust](https://rustup.rs/)，再运行：

```shell
# cargo build 会构建所有可执行文件和库。
# cargo run 会运行默认的可执行文件，也就是窗口管理器。
cargo build && cargo run
```

完成修改后，将提交推送到你的复刻仓库，并向主仓库的 `main` 分支[提交 PR](https://github.com/glzr-io/glazewm/pulls)。每个 PR 尽量只包含一项功能或一处修复，便于审查。

### 开发提示

如果使用 VS Code，推荐安装 [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 扩展。在 VS Code 的 `settings.json` 中添加以下设置，即可自动运行 Clippy 检查：

```json
{
  "rust-analyzer.check.command": "clippy"
}
```

## 代码库概览

修改代码时，只需熟悉相关部分。可以参考下文定位所需的模块。

### Rust crate

- `wm`（可执行程序）是主程序，负责核心窗口管理功能。安装路径为 `C:\Program Files\glzr.io\glazewm.exe`。
- `wm-cli`（可执行程序和库）提供与主程序交互的命令行界面。安装路径为 `C:\Program Files\glzr.io\cli\glazewm.exe`，其目录默认会添加到 `PATH`。
- `wm-common`（库）提供各 crate 共用的类型、工具和常量。
- `wm-ipc-client`（库）提供 WebSocket 客户端，用于与主程序进行进程间通信。
- `wm-platform`（库）封装平台专用 API。其他 crate 通过该库使用 Windows 和 macOS API。
- `wm-watcher`（可执行程序）是监视进程，负责在主程序退出后执行清理。安装路径为 `C:\Program Files\glzr.io\glazewm-watcher.exe`。

### 命令与事件

GlazeWM 采用命令与事件驱动的架构。[命令](packages/wm/src/commands)和[事件](packages/wm/src/events)会修改保存在 [`WmState`](packages/wm/src/wm_state.rs) 中的窗口管理器状态。

- 快捷键、进程间通信、命令行界面或其他命令都可以触发命令。命令行界面内部通过进程间通信发送命令。多数命令仅供内部使用，可能没有公开 API。
- Windows 会在窗口创建、销毁或获得焦点等情况下产生事件。每种事件都有相应的处理程序来更新窗口管理器状态。

命令与事件由 [`start_wm`](packages/wm/src/main.rs) 中的循环处理。

## 容器树

GlazeWM 使用树状层级组织窗口，其中包括根容器、显示器（物理屏幕）、工作区（窗口的虚拟分组）、拆分容器（用于平铺布局）和应用程序窗口。

```text
根容器
├── 显示器 1
│   ├── 工作区 1（水平）
│   │   ├── 平铺窗口（终端）
│   │   └── 拆分容器（垂直）
│   │       ├── 平铺窗口（Chrome）
│   │       └── 平铺窗口（VS Code）
│   └── 工作区 2（垂直）
│       └── 平铺窗口（Spotify）
└── 显示器 2
    └── 工作区 3（水平）
        ├── 平铺窗口（Discord）
        └── 浮动窗口（Slack）
```

平铺窗口可以嵌套在拆分容器中。浮动、最小化、最大化或全屏窗口始终是工作区的直接子节点。拆分容器的子节点只能是窗口，而且至少要有一个子窗口。
