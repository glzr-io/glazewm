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

## Direct download

The latest runnable executable can be downloaded via [releases](https://github.com/lars-berger/GlazeWM/releases). No installation necessary, simply run the `.exe` file.

## Winget

GlazeWM can be downloaded via Winget package manager:

```
winget install lars-berger.GlazeWM
```

Winget installs portable packages in `%LOCALAPPDATA%\Microsoft\Winget\Packages\` by default. This can be overrided with the flag `--location \path\to\folder`.

## Scoop

GlazeWM can be download via Scoop in the [Extras](https://github.com/ScoopInstaller/Extras) bucket:

```powershell
scoop bucket add extras # Ensure bucket is added first
scoop install glazewm
```

## Build from source

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

```console
./GlazeWM.exe --config="C:\<PATH_TO_CONFIG>\config.yaml"
```

## General

```yaml
general:
  show_floating_on_top: false
  floating_window_move_amount: "5%"
  # When enabled, switching to the current workspace activates the previously focused workspace
  toggle_workspace_on_refocus: false
  focus_border_color: "#42c0fb"
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

Keybindings with <kbd>Alt</kbd> pressed:

![Alt key pressed - with keybindings](https://user-images.githubusercontent.com/34844898/194635035-152ed4a6-e5a1-4878-8863-f62391e7d703.png)

Keybindings with <kbd>Alt</kbd>+<kbd>Shift</kbd> pressed:

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
  - name: "1"

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
  # The option to enable/disable the bar.
  enabled: true

  # Height of the bar in pixels.
  height: "30px"

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
  font_size: "13px"

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
  padding: "4px 6px 4px 6px"

  # Separator between components within the bar. `label` is used for each section
  # of the bar unless `label_{left,center,right}` is explictly set, in which case
  # they are preferred over default.
  component_separator:
    label: " | "

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
# Type of component to display. Currently 7 component types exist: "workspaces", "clock", "text", "battery", "window title", "binding mode" and "tiling direction".
type: <COMPONENT_TYPE>

# Horizontal and vertical margins. See "Shorthand properties" for more info.
margin: "0 10px 0 0"

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
font_size: "13px"

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

### Bar component: Clock

The text shown in the clock component is specified via `time_formatting`. The supported time format specifiers are defined by [.NET's time/date string formatting](https://learn.microsoft.com/en-us/dotnet/standard/base-types/custom-date-and-time-format-strings).

Additionally supported format specifiers:

| Specifier | Description         | Example                                          |
| --------- | ------------------- | ------------------------------------------------ |
| "w"       | Week of year: 1..53 | 'HH:mm dd.MM.yyyy (ww)' => 13:05 21.12.2022 (51) |
| "ww"      | Week of year 01..53 | 'HH:mm dd.MM.yyyy (ww)' => 13:05 02.01.2022 (02) |

**Example usage:**

```yaml
- type: "clock"
  time_formatting: "hh:mm tt  ddd MMM d"
```

### Bar Component: Battery

The battery component displays the system's battery level in percent.
There are three labels available that can be customized:

- `label_draining`: used when the system is draining battery power(i.e. not charging).
- `label_power_saver`: used when the system is on power saving mode.
- `label_charging`: used when the system is connected to power.

`{battery_level}` is a variable which is replaced by the actual battery level when the label is displayed.

**Example usage:**

```yaml
- type: "battery"
  label_draining: "{battery_level}% remaining"
  label_power_saver: "{battery_level}% (power saver)"
  label_charging: "{battery_level}% (charging)"
```

### Bar Component: CPU Usage

Displays the current CPU usage.

```yaml
- type: "cpu"

  # {0} is substituted by percentage, {1} by current value, {2} by max value
  # Example: '{1}/{2} MHz ({0}%)'
  #          'CPU {0}%'
  string_format: "CPU {0}%"

  # For formats, see: https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-numeric-format-strings#standard-format-specifiers.
  percent_format: "00" # Format for {0}.
  current_value_format: "0.00" # Format for {1}.
  max_value_format: "0.00" # Format for {2}.
  refresh_interval_ms: 500 # How often this counter is refreshed
  divide_by: 1000 # Convert MHz to GHz (where appropriate).
  counter: CpuUsage

  # Supported Counters Include:
  # CpuUsage: Overall CPU Usage across All Cores.
  # CpuFrequency: Overall CPU Frequency across All Cores.
  # PackagePower: [Requires Admin] Overall Power used by CPU Package [not guaranteed to work]
  # CoreTemp: [Requires Admin] Average Core Temperature of CPU Package [not guaranteed to work]
```

### Bar Component: GPU Usage

This component has high CPU requirement (compared to others); due to no efficient way to pull data from Windows API. Avoid using low refresh intervals.

```yaml
- type: "gpu"
  string_format: "GPU {0}%" # {0} is substituted by number format
  number_format: "00" # See https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-numeric-format-strings#standard-format-specifiers.
  refresh_interval_ms: 1000 # How often this counter is refreshed
  flags: Graphics

  # Supported flags
  # Multiple flags can be specified by e.g. `flags: Graphics, VideoDecode, VideoEncode, Copy`

  # Graphics: 3D GPU Engine usage. [Probably what you want]
  # VideoDecode: Load of dedicated video decoding silicon.
  # VideoEncode: Load of dedicated video encoding silicon, i.e. NVENC/AMD AMF/QuickSync.
  # LegacyOverlay: Legacy API for overlaying items over other items.
  # Copy: Load copying data without intervention of CPU e.g. copying framebuffer across screens in multi GPU setup or uploading textures.
  # Security: Workloads related to cryptography, such as encryption, decryption, and secure video processing.
  # Vr: Virtual Reality related workloads.
```

### Bar Component: Memory Usage

Displays the current Memory usage.

```yaml
- type: "memory"
  # {0} is substituted by percentage, {1} by current value, {2} by max value
  # Example: '{1}/{2} MB ({0}%)'
  #          '{0}%'
  string_format: "RAM {1}/{2}GB ({0}%)"

  # For formats, see: https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-numeric-format-strings#standard-format-specifiers.
  percent_format: "00" # Format for {0}.
  current_value_format: "00.00" # Format for {1}.
  max_value_format: "00.00" # Format for {2}.
  refresh_interval_ms: 1000 # How often this counter is refreshed
  divide_by: 1000000000 # Convert to GigaBytes.

  # Supported Counters Include:
  # PhysicalMemory: Current amount of physical RAM in use; i.e. working set.
  # CacheBytes: Amount of cached file data in physical RAM.
  # CommitSize: Retrieves the amount of committed virtual memory (bytes); i.e. which has space reserved on the disk paging file(s).
  # PagedResidentBytes: Size of the active portion of the paged pool in physical memory, storing objects that can be written to disk when they're not in use.
  counter: PhysicalMemory
```

### Bar Component: Text File

For displaying any content without a native integrated widget; updates in real time.

```yaml
- type: "text file"
  file_path: "PATH_HERE" # path to file
```

### Bar Component: Weather

Uses Open-Meteo API, refreshes every hour.

```yaml
- type: "weather"
  latitude: 40.6892
  longitude: 74.0445
  format: "{0}{1}°C" # {0} icon, {1} temperature.
  temperature_unit: Celsius # or Fahrenheit
  temperature_format: "0" # Format of {1}
  label_sun: "☀️"
  label_moon: "🌙"
  label_cloud_moon: "🌙☁️"
  label_cloud_sun: "⛅"
  label_cloud_moon_rain: "🌙🌧️"
  label_cloud_sun_rain: "🌦️"
  label_cloud_rain: "🌧️"
  label_snow_flake: "❄️"
  label_thunderstorm: "⚡"
  label_cloud: "☁️"
```

### Adding Custom Bar Components

[Guide Available Here](./docs/contributing-new-components.md)

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
- set \<floating | tiling | minimized | maximized>
- set \<width | height> \<amount in px | amount in %> (eg. `set height 30%` or `set width 200px`)
- toggle \<floating | maximized>
- toggle focus mode
- tiling direction toggle
- exit wm
- reload config
- close
- exec \<process name | path to executable> (eg. `exec chrome` or `exec 'C:/Program Files/Google/Chrome/Application/chrome'`)
- ignore

# Known issues

## Blurry buttons in bar window

An app called "Sonic Studio", which is installed by default on ASUS ROG machines can cause rendering issues with WPF apps. This can be resolved by disabling `NahimicService` in Windows Services Manager.

## Binding the right-side Alt key `RMenu` on certain keyboard layouts

Most keyboard layouts treat the right-side <kbd>Alt</kbd> key the same as the left, while others (eg. US International and German) treat it as <kbd>AltGr</kbd> and generate both <kbd>Ctrl</kbd> and <kbd>Alt</kbd> when it is pressed. For these keyboard layouts, keybindings with the <kbd>AltGr</kbd> key need to specify both `RMenu` and `Control` (eg. `RMenu+Control+A`).
