# GlazeWM &middot; [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/lars-berger/GlazeWM/pulls) [![License](https://img.shields.io/github/license/lars-berger/GlazeWM)](https://github.com/lars-berger/GlazeWM/blob/master/LICENSE.md) [![Discord invite](https://img.shields.io/discord/1041662798196908052)](https://discord.gg/ud6z3qjRvM)

GlazeWM is a tiling window manager for Windows inspired by i3 and Polybar.

Why use a tiling window manager? A tiling WM lets you easily organize windows and adjust their layout on the fly by using keyboard-driven commands.

- Simple YAML configuration
- Multi-monitor support
- Customizable bar window
- Customizable rules for specific windows
- Easy one-click installation

![demo](https://user-images.githubusercontent.com/34844898/142960922-fb3abd0d-082c-4f92-8613-865c68006bd8.gif)

Under the hood, GlazeWM adds functionality to the built-in window manager and uses the Windows API via P/Invoke to position windows.

# Download

The latest runnable binary can be downloaded via [releases](https://github.com/lars-berger/GlazeWM/releases). No installation necessary, simply run the executable.

Alternatively, to build from source, use the following .NET CLI command:

```
dotnet publish ./GlazeWM.Bootstrapper/GlazeWM.Bootstrapper.csproj --configuration=Release --runtime=win-x64 --output=. --self-contained -p:PublishSingleFile=true -p:IncludeAllContentForSelfExtract=true
```

To build for other runtimes than Windows x64, see [here](https://docs.microsoft.com/en-us/dotnet/core/rid-catalog#windows-rids).

# Roadmap

- Improve handling of fullscreen and maximized windows.
- More bar components.

[📋 Full roadmap](https://github.com/users/lars-berger/projects/2/views/1)

# Configuration

The configuration file for GlazeWM can be found at `C:\Users\<YOUR_USER>\.glaze-wm\config.yaml`. If this file doesn't exist, it can optionally be generated with some sensible defaults on application launch.

To use a different config file location, you can launch the GlazeWM executable with the CLI argument `--config="..."`, like so:

```
./GlazeWM.exe --config="C:\<PATH_TO_CONFIG>\config.yaml"
```

## Keybindings

The available keybindings can be customized via the `keybindings` property in the config file. A keybinding consists of one or more key combinations and one or more commands to run when pressed.

A full list of keys that can be used for keybindings can be found [here](https://docs.microsoft.com/en-us/dotnet/api/system.windows.forms.keys?view=windowsdesktop-5.0#fields). Numbers can be used in keybindings with and without a `D` prefix (eg. either `D1` or `1` works).

It's recommended to use the alt key for keybindings. The windows key is unfortunately a pain to remap, since certain keybindings (eg. `LWin+L`) are reserved by the OS.

```yaml
keybindings:
  # Command to run.
  - command: "focus workspace 1"

    # Key combination to trigger the keybinding.
    binding: "Alt+1"

  # To run multiple commands in a sequence, use the `commands` property (eg. to move a window to a
  # workspace + focus workspace).
  - commands: ["move to workspace 1", "focus workspace 1"]
    binding: "Alt+Shift+1"

  - command: "focus left"
    # To have multiple key combinations that can trigger a command, use the `bindings` property.
    bindings: ["Alt+H", "Alt+Left"]
```

### Default keybindings

Keybindings with Alt pressed:

![Alt key pressed - with keybindings](https://user-images.githubusercontent.com/34844898/194635035-152ed4a6-e5a1-4878-8863-f62391e7d703.png)

Keybindings with Alt+Shift pressed:

![Alt+shift key pressed - with keybindings](https://user-images.githubusercontent.com/34844898/194635089-d5ed152b-1527-43e8-a69c-4e154b97a207.png)

Apart from the `Alt+Shift+E` binding for exiting GlazeWM, it's also possibly to safely exit via the system tray icon.

## Gap configuration

The gaps between windows can be changed via the `gaps` property in the config file. Inner and outer gaps are set separately.

```yaml
gaps:
  # Gap between adjacent windows.
  inner_gap: 20

  # Gap between windows and the screen edge.
  outer_gap: 20
```

## Workspaces configuration

Workspaces need to be predefined via the `workspaces` property in the config file. A workspace is automatically assigned to each monitor on startup.

```yaml
workspaces:
  # Uniquely identifies the workspace and is used as the label for the workspace in the bar if
  # `display_name` is not provided.
  - name: 1

    # Optional override for the workspace label in the bar. Does not need to be unique.
    display_name: "Work"

    # Optionally force the workspace on a specific monitor if it exists. Use the monitor's number
    # as shown in the Windows display settings (eg. 1, 2, 3...).
    bind_to_monitor: 1
```

## Bar configuration

The appearance of the bar can be changed via the `bar` property in the config file.

```yaml
bar:
  # Height of the bar in pixels.
  height: 30

  # The position of the bar on the screen. Can be either "top" or "bottom".
  position: "top"

  # Opacity value between 0.0 and 1.0.
  opacity: 1.0

  # Background color of the bar.
  background: "#101010"

  # Default font color. Can be overriden by setting `foreground` in a component's config.
  foreground: "white"

  # Default font family. Can be overriden by setting `font_family` in a component's config.
  font_family: "Segoe UI"

  # Default font size. Can be overriden by setting `font_size` in a component's config.
  font_size: "13"

  # Default font weight. Typically ranges from 100 to 950, where a higher value is thicker. Can
  # be overriden by setting `font_weight` in a component's config.
  font_weight: "400"

  # Horizontal and vertical borders in pixels. Borders are inside the dimensions of the bar and do
  # not affect bar height. See "Shorthand properties" for more info.
  border_width: "0"

  # Color of the border.
  border_color: "blue"

  # Horizontal and vertical spacing between components within the bar and the edges of the bar. See
  # "Shorthand properties" for more info.
  padding: "1 6 1 6"

  # Components to display on the left side of the bar.
  components_left:
    - type: "workspaces"

  # Components to display on the right side of the bar.
  components_right:
    - type: "clock"
```

### Bar component configuration

The appearance of bar components can also be customized. The following properties can change the styling of a component, regardless of the component type.

```yaml
# Type of component to display. Currently only 3 component types exist: "workspaces", "clock" and "text".
type: <COMPONENT_TYPE>

# Horizontal and vertical margins. See "Shorthand properties" for more info.
margin: "0 10 0 0"

# Horizontal and vertical padding. See "Shorthand properties" for more info.
padding: "0"

# Opacity value between 0.0 and 1.0.
opacity: 1.0

# Background color of the component.
background: "#101010"

# Font color used within the component.
foreground: "white"

# Font family used within the component.
font_family: "Segoe UI"

# Font size used within the component.
font_size: "13"

# Font weight used within the component. Typically ranges from 100 to 950, where a higher value is
# thicker.
font_weight: "400"

# Horizontal and vertical borders in pixels. Borders are inside the dimensions of the component.
# See "Shorthand properties" for more info.
border_width: "0"

# Color of the border.
border_color: "blue"
```

### Shorthand properties

Properties related to the edges of the bar or a component, like `padding`, `margin`, and `border_width`, use a 1-to-4 value syntax. This is the same convention that's common in CSS.

Using the example of padding:

- When one value is specified, it applies the same padding to all four sides.
- When two values are specified, the first padding applies to the top and bottom, the second to the left and right.
- When three values are specified, the first padding applies to the top, the second to the right and left, the third to the bottom.
- When four values are specified, the paddings apply to the top, right, bottom, and left in that order (clockwise).

## Window rules

Commands can be run when a window is initially launched. This can be used to assign an app to a specific workspace or to always start an app in floating mode.

Multiple matching criteria can be used together to target a window more precisely. Regex syntax can also be used by wrapping the pattern with `/` (eg. `/notepad|chrome/`)

```yaml
window_rules:
  # Command to run. Use `commands` to specify an array of commands to run in sequence.
  - command: "move to workspace 2"

    # Process name to match exactly.
    match_process_name: "chrome"

    # Window title to match exactly.
    match_title: "/.*/"

    # Class name to match exactly.
    match_class_name: "Chrome_WidgetWin_1"

  # To prevent the WM from managing an app, use the "ignore" command.
  - command: "ignore"
    match_process_name: "notepad"
```

# Available commands

- layout \<vertical | horizontal>
- focus \<left | right | up | down>
- focus workspace \<prev | next | recent>
- focus workspace \<workspace name>
- move \<left | right | up | down>
- move to workspace \<workspace name>
- resize \<height | width> \<amount in px | amount in %> (eg. `resize height 3%` or `resize width 20px`)
- resize borders [\<shorthand property>](#shorthand-properties) (eg. `resize borders 0px -7px -7px -7px`)
- set \<floating | minimized | maximized>
- toggle \<floating | maximized>
- toggle focus mode
- exit wm
- reload config
- close
- exec \<process name | path to executable> (eg. `exec chrome` or `exec 'C:/Program Files/Google/Chrome/Application/chrome'`)
- ignore

# Known issues

## Blurry buttons in bar window

An app called "Sonic Studio", which is installed by default on ASUS ROG machines can cause rendering issues with WPF apps. This can be resolved by disabling `NahimicService` in Windows Services Manager.

## Binding the right-side Alt key `RMenu` on certain keyboard layouts

Most keyboard layouts treat the right-side Alt key as a regular Alt key, while others (eg. US International and German) treat it as AltGr and generate both Ctrl and Alt when it is pressed. For these keyboard layouts, keybindings with the AltGr key need to specify both `RMenu` and `Control` (eg. `RMenu+Control+A`).
