<div align="center">

> V3 终于发布了 - 查看更新日志 [这里](https://github.com/glzr-io/GlazeWM/releases) 🔥

  <br>
  <img src="./resources/assets/logo.svg" width="230" alt="GlazeWM logo" />
  <br>

# GlazeWM

**一个受 i3wm 启发的 Windows 平铺窗口管理器。**

[![Discord invite][discord-badge]][discord-link]
[![Downloads][downloads-badge]][downloads-link]
[![Good first issues][issues-badge]][issues-link]

GlazeWM 让您可以通过键盘驱动的命令轻松组织窗口并即时调整其布局。

[安装](#安装) •
[默认快捷键](#默认快捷键) •
[配置文档](#配置文档) •
[常见问题](#常见问题) •
[贡献 ↗](https://github.com/glzr-io/glazewm/blob/main/CONTRIBUTING.md)

![Demo video][demo-video]

</div>

### 🌟 主要特性

- 简单的 YAML 配置
- 多显示器支持
- 针对特定窗口的可自定义规则
- 简单的一键安装
- 与 [Zebar](https://github.com/glzr-io/zebar) 状态栏集成

## 安装

**GlazeWM 的最新版本可通过 [releases](https://github.com/glzr-io/GlazeWM/releases) 下载。** 在安装过程中可以通过复选框选择性安装 Zebar。

GlazeWM 也可以通过多个包管理器获得：

**Winget**

```sh
winget install GlazeWM
```

**Chocolatey**

```sh
choco install glazewm
```

**Scoop**

```sh
scoop bucket add extras
scoop install extras/glazewm
```

## 贡献

帮助修复困扰您的问题，或添加您一直想要的功能！我们非常欢迎贡献。

本地开发和指南可在 [贡献指南](https://github.com/glzr-io/glazewm/blob/main/CONTRIBUTING.md) 中找到。

## 默认快捷键

在 GlazeWM 首次启动时，可以选择性地生成默认配置。

以下是所有可用命令及其默认快捷键的速查表。

![Infographic](/resources/assets/cheatsheet_cn_ZH.png)

## 配置文档

[默认配置](https://github.com/glzr-io/glazewm/blob/main/resources/assets/sample-config.yaml) 文件生成在 `%userprofile%\.glzr\glazewm\config.yaml`。

要使用不同的配置文件位置，您可以使用 CLI 参数 `--config="..."` 启动 GlazeWM 可执行文件，如下所示：

```sh
./glazewm.exe start --config="C:\<配置文件路径>\config.yaml"
```

或者为 `GLAZEWM_CONFIG_PATH` 环境变量传递一个值：

```sh
setx GLAZEWM_CONFIG_PATH "C:\<配置文件路径>\config.yaml"
```

使用自定义路径的好处是您可以为配置文件选择不同的名称，例如 `glazewm.yaml`。

### 配置：常规

```yaml
general:
  # WM 启动时运行的命令（例如运行脚本或启动另一个应用程序）。
  startup_commands: []

  # WM 关闭前运行的命令。
  shutdown_commands: []

  # WM 配置重新加载后运行的命令。
  config_reload_commands: []

  # 是否自动聚焦光标下方的窗口。
  focus_follows_cursor: false

  # 当聚焦当前工作区时，是否在先前聚焦的工作区之间来回切换。
  toggle_workspace_on_refocus: false

  cursor_jump:
    # 是否在指定触发器上自动移动光标。
    enabled: true

    # 光标跳转的触发器：
    # - 'monitor_focus': 当焦点在显示器之间切换时跳转。
    # - 'window_focus': 当焦点在窗口之间切换时跳转。
    trigger: "monitor_focus"
```

### 配置：快捷键

可用的键盘快捷键可以通过 `keybindings` 选项自定义。快捷键由一个或多个按键组合和按下时运行的一个或多个命令组成。

建议使用 alt 键作为快捷键。不幸的是，Windows 键很难重新映射，因为操作系统保留了某些快捷键（例如 `lwin+l`）。

```yaml
keybindings:
  # 要运行的命令。
  - commands: ["focus --workspace 1"]

    # 触发快捷键的按键组合。
    bindings: ["alt+1"]

  # 可以按顺序运行多个命令（例如将窗口移动到工作区 + 聚焦工作区）。
  - commands: ["move --workspace 1", "focus --workspace 1"]
    bindings: ["alt+shift+1"]
```

**可用于快捷键的完整按键列表：**

<details>
<summary>按键列表</summary>

| 按键                  | 描述                                                           |
| --------------------- | -------------------------------------------------------------- |
| `a` - `z`             | 字母键                                                         |
| `0` - `9`             | 数字键                                                         |
| `numpad0` - `numpad9` | 数字小键盘键                                                   |
| `f1` - `f24`          | 功能键                                                         |
| `shift`               | 左或右 SHIFT 键                                                |
| `lshift`              | 左 SHIFT 键                                                    |
| `rshift`              | 右 SHIFT 键                                                    |
| `control`             | 左或右 CTRL 键                                                 |
| `lctrl`               | 左 CTRL 键                                                     |
| `rctrl`               | 右 CTRL 键                                                     |
| `alt`                 | 左或右 ALT 键                                                  |
| `lalt`                | 左 ALT 键                                                      |
| `ralt`                | 右 ALT 键                                                      |
| `lwin`                | 左 ⊞ Windows 徽标键                                            |
| `rwin`                | 右 ⊞ Windows 徽标键                                            |
| `space`               | 空格键                                                         |
| `escape`              | ESCAPE 键                                                      |
| `back`                | BACKSPACE 键                                                   |
| `tab`                 | TAB 键                                                         |
| `enter`               | ENTER 键                                                       |
| `left`                | ← 方向键                                                       |
| `right`               | → 方向键                                                       |
| `up`                  | ↑ 方向键                                                       |
| `down`                | ↓ 方向键                                                       |
| `num_lock`            | NUM LOCK 键                                                    |
| `scroll_lock`         | SCROLL LOCK 键                                                 |
| `caps_lock`           | CAPS LOCK 键                                                   |
| `page_up`             | PAGE UP 键                                                     |
| `page_down`           | PAGE DOWN 键                                                   |
| `insert`              | INSERT 键                                                      |
| `delete`              | DELETE 键                                                      |
| `end`                 | END 键                                                         |
| `home`                | HOME 键                                                        |
| `print_screen`        | PRINT SCREEN 键                                                |
| `multiply`            | `*` 键（仅限数字小键盘）                                       |
| `add`                 | `+` 键（仅限数字小键盘）                                       |
| `subtract`            | `-` 键（仅限数字小键盘）                                       |
| `decimal`             | DEL 键（仅限数字小键盘）                                       |
| `divide`              | `/` 键（仅限数字小键盘）                                       |
| `volume_up`           | 音量增加键                                                     |
| `volume_down`         | 音量减少键                                                     |
| `volume_mute`         | 静音键                                                         |
| `media_next_track`    | 媒体下一曲键                                                   |
| `media_prev_track`    | 媒体上一曲键                                                   |
| `media_stop`          | 媒体停止键                                                     |
| `media_play_pause`    | 媒体播放/暂停键                                                |
| `oem_semicolon`       | 美式标准键盘上的 `;`/`:` 键（因键盘而异）                      |
| `oem_question`        | 美式标准键盘上的 `/`/`?` 键（因键盘而异）                      |
| `oem_tilde`           | 美式标准键盘上的 `` ` ``/`~` 键（因键盘而异）                  |
| `oem_open_brackets`   | 美式标准键盘上的 `[`/`{` 键（因键盘而异）                      |
| `oem_pipe`            | 美式标准键盘上的 `\`/`\|` 键（因键盘而异）                     |
| `oem_close_brackets`  | 美式标准键盘上的 `]`/`}` 键（因键盘而异）                      |
| `oem_quotes`          | 美式标准键盘上的 `'`/`"` 键（因键盘而异）                      |
| `oem_plus`            | 美式标准键盘上的 `=`/`+` 键（因键盘而异）                      |
| `oem_comma`           | 美式标准键盘上的 `,`/`<` 键（因键盘而异）                      |
| `oem_minus`           | 美式标准键盘上的 `-`/`_` 键（因键盘而异）                      |
| `oem_period`          | 美式标准键盘上的 `.`/`>` 键（因键盘而异）                      |

</details>

如果某个按键不在上述列表中，如果您在快捷键中使用其字符，它很可能仍然受支持（例如挪威语 Å 字符的 `alt+å`）。

> 德语和美式国际键盘对右侧 alt 键的处理不同。对于这些键盘布局，请使用 `ralt+ctrl` 而不是 `ralt` 来绑定右侧 alt 键。

### 配置：间隙

窗口之间的间隙可以通过配置文件中的 `gaps` 属性更改。内部和外部间隙分别设置。

```yaml
gaps:
  # 相邻窗口之间的间隙。
  inner_gap: "20px"

  # 窗口与屏幕边缘之间的间隙。
  outer_gap:
    top: "20px"
    right: "20px"
    bottom: "20px"
    left: "20px"
```

### 配置：工作区

工作区需要通过配置文件中的 `workspaces` 属性预定义。启动时，每个显示器会自动分配一个工作区。

```yaml
workspaces:
  # 这是工作区的唯一 ID。它用于快捷键命令，如果未提供 `display_name`，
  # 它也是第三方应用程序（例如 Zebar）中显示的标签。
  - name: "1"

    # 第三方应用程序中使用的工作区标签的可选覆盖。
    # 不需要是唯一的。
    display_name: "工作"

    # 如果存在，可选择强制工作区在特定显示器上。
    # 0 是您最左边的屏幕，1 是右边的下一个，依此类推。
    bind_to_monitor: 0

    # 可选择防止工作区在空时被停用。
    keep_alive: false
```

### 配置：窗口规则

可以在窗口首次启动时运行命令。这对于添加特定于窗口的行为很有用，比如始终以全屏模式启动窗口或分配到特定工作区。

窗口可以通过其进程、类和标题进行定位。可以一起使用多个匹配条件来更精确地定位窗口。

```yaml
window_rules:
  - commands: ["move --workspace 1"]
    match:
      # 将浏览器移动到工作区 1。
      - window_process: { regex: "msedge|brave|chrome" }

  - commands: ["ignore"]
    match:
      # 忽略任何 Zebar 窗口。
      - window_process: { equals: "zebar" }

      # 忽略浏览器的画中画窗口。
      # 注意标题和类都必须匹配才能运行规则。
      - window_title: { regex: "[Pp]icture.in.[Pp]icture" }
        window_class: { regex: "Chrome_WidgetWin_1|MozillaDialogClass" }
```

### 配置：窗口效果

可以通过 `window_effects` 选项对窗口应用视觉效果。目前，彩色边框是唯一可用的效果，未来会有更多效果。

> 注意：窗口效果仅适用于 Windows 11。

```yaml
window_effects:
  # 应用于聚焦窗口的视觉效果。
  focused_window:
    # 用彩色边框突出显示窗口。
    border:
      enabled: true
      color: "#0000ff"

  # 应用于非聚焦窗口的视觉效果。
  other_windows:
    border:
      enabled: false
      color: "#d3d3d3"
```

### 配置：窗口行为

`window_behavior` 配置选项用于自定义窗口可以处于的状态（`tiling`、`floating`、`minimized` 和 `fullscreen`）。

```yaml
window_behavior:
  # 新窗口在可能的情况下以此状态创建。
  # 允许的值：'tiling'、'floating'。
  initial_state: "tiling"

  # 设置创建新窗口时的默认选项。这也会更改状态更改命令
  # （如 `set-floating`）在不使用任何标志时的默认值。
  state_defaults:
    floating:
      # 是否默认居中浮动窗口。
      centered: true

      # 是否将浮动窗口显示为始终在顶部。
      shown_on_top: false

    fullscreen:
      # 如果可能，最大化窗口。如果窗口没有最大化按钮，
      # 则会正常全屏显示。
      maximized: false
```

### 配置：绑定模式

绑定模式用于在 GlazeWM 运行时修改快捷键。

可以使用 `wm-enable-binding-mode --name <名称>` 启用绑定模式，使用 `wm-disable-binding-mode --name <名称>` 禁用。

```yaml
binding_modes:
  # 启用时，可以通过方向键或 HJKL 调整聚焦窗口的大小。
  - name: "resize"
    keybindings:
      - commands: ["resize --width -2%"]
        bindings: ["h", "left"]
      - commands: ["resize --width +2%"]
        bindings: ["l", "right"]
      - commands: ["resize --height +2%"]
        bindings: ["k", "up"]
      - commands: ["resize --height -2%"]
        bindings: ["j", "down"]
      # 按 enter/escape 返回默认快捷键。
      - commands: ["wm-disable-binding-mode --name resize"]
        bindings: ["escape", "enter"]
```

## 常见问题

**问：如何在启动时运行 GlazeWM？**

通过右键单击 GlazeWM 可执行文件 -> `创建快捷方式` 为可执行文件创建快捷方式。将快捷方式放在启动文件夹中，您可以通过在文件资源管理器的顶部栏中输入 `shell:startup` 来访问该文件夹。

**问：如何创建 `<插入布局>`？**

您可以通过使用 `alt+v` 更改平铺方向来创建自定义布局。这会改变下一个窗口相对于当前窗口的放置位置。如果当前窗口的方向是水平的，新窗口将放置在其右侧。如果是垂直的，将放置在其下方。这也适用于移动窗口；固定窗口的平铺方向将影响移动窗口的放置位置。

社区制作的脚本如 [Dutch-Raptor/GAT-GWM](https://github.com/Dutch-Raptor/GAT-GWM) 和 [burgr033/GlazeWM-autotiling-python](https://github.com/burgr033/GlazeWM-autotiling-python) 可用于自动更改平铺方向。目前不支持自动布局的原生支持。

**问：如何为 `<插入应用程序>` 创建规则？**

要匹配特定应用程序，您需要一个要执行的命令以及窗口的进程名称、标题或类名称。例如，如果您使用 Flow-Launcher 并希望设置窗口浮动，您可以执行以下操作：

```yaml
window_rules:
  - commands: ["set-floating"]
    match:
      - window_process: { equals: "Flow.Launcher" }
        window_title: { equals: "Settings" }
```

像 Winlister 或 AutoHotkey 的 Window Spy 这样的程序对于获取窗口信息很有用。

**问：当 `<插入应用程序>` 聚焦时，如何忽略 GlazeWM 的快捷键？**

目前不支持此功能，但是，默认配置中的快捷键 `alt+shift+p` 用于禁用所有其他快捷键，直到再次按下 `alt+shift+p`。

[discord-badge]: https://img.shields.io/discord/1041662798196908052.svg?logo=discord&colorB=7289DA
[discord-link]: https://discord.gg/ud6z3qjRvM
[downloads-badge]: https://img.shields.io/github/downloads/glzr-io/glazewm/total?logo=github&logoColor=white
[downloads-link]: https://github.com/glzr-io/glazewm/releases
[issues-badge]: https://img.shields.io/badge/good_first_issues-7057ff
[issues-link]: https://github.com/orgs/glzr-io/projects/4/views/1?sliceBy%5Bvalue%5D=good+first+issue
[demo-video]: resources/assets/demo.webp
