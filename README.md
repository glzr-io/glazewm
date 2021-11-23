GlazeWM is a tiling window manager for Windows inspired by i3 and Polybar.

# Download

The runnable binary can be downloaded via [releases](https://todo-add-url-here) or built from source using `dotnet publish --configuration=Release` and running `GlazeWM.Bootstrapper.exe` from the compiled output.

# Roadmap

- Improve handling of fullscreen and maximized windows.
- More bar components.
- Be able to move floating windows via `move to workspace <NAME>` command.
- Avoid managing windows that can't be moved/resized (eg. windows running with elevated priveleges).
- Change the size of sibling windows when a tiling window is resized with the sizing border.

# Configuration

A configuration file is created with some sensible defaults on the first run of GlazeWM. It can be found at `C:\Users\<YOUR_USER>\.glaze-wm\config.yaml`.

## Keybindings

The available keybindings can be customized via the `keybindings` property in the config file. A keybinding consists of one or more key combinations and one or more commands to run when pressed.

A full list of keys that can be used for keybindings can be found [here](https://docs.microsoft.com/en-us/dotnet/api/system.windows.forms.keys?view=windowsdesktop-5.0#fields). Numbers can be used in keybindings with and without a `D` prefix (eg. either `D1` or `1` works).

It's recommended to use the alt key for keybindings. The windows key is unfortunately a pain to remap, since certain keybindings (eg. `LWin+L`) are reserved by the OS.

```
keybindings:
		# Command to run. Use "commands" to specify an array of commands to run in sequence.
  - command: "focus workspace 1"

		# Key combination to trigger the keybinding. Use "bindings" to provide an array of key combinations that can trigger the keybinding.
    binding: "Alt+1"
```

### Default keybindings

Keybindings with Alt pressed:

[Graphic of default keybindings and the command invoked].

Keybindings with Alt+Shift pressed:

[Graphic of default keybindings and the command invoked when shift is held].

Apart from the `Alt+Shift+E` binding for exiting GlazeWM, it's also possibly to safely exit via the system tray icon.

## Gap configuration

The gaps between windows can be changed via the `gaps` property in the config file. Inner and outer gaps are set separately.

```
gaps:
  # Gap between adjacent windows.
  inner_gap: 20

  # Gap between windows and the screen edge.
  outer_gap: 20
```

## Workspaces configuration

Workspaces need to be predefined via the `workspaces` property in the config file. A workspace is automatically assigned to each monitor on startup.

```
workspaces:
  # Uniquely identifies the workspace and is used as the label for the workspace in the bar.
  - name: 1
```

## Bar configuration

The appearance of the bar can be changed via the `bar` property in the config file.

```
bar:
  # Height of the bar in pixels.
  height: 30

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

  # Horizontal and vertical borders in pixels. Borders are inside the dimensions of the bar and do not affect bar height. See "Shorthand properties" for more info.
  border_width: "0"

  # Color of the border.
  border_color: "blue"

  # Horizontal and vertical spacing between components within the bar and the edges of the bar. See "Shorthand properties" for more info.
  padding: "1 6 1 6"

  # Components to display on the left side of the bar.
  components_left:
    - type: "workspaces"

  # Components to display on the right side of the bar.
  components_right:
    - type: "clock"
```

### Bar component configuration

Bar components have some properties that can be changed regardless of the component type.

```
  # Type of component to display. Currently only 2 component types exist:  "workspaces" and "clock".
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

  # Font family used within the component.
  font_size: "13"

  # Horizontal and vertical borders in pixels. Borders are inside the dimensions of the component. See "Shorthand properties" for more info.
  border_width: "0"

  # Color of the border.
  border_color: "blue"
```

### Shorthand properties

Properties related to the edges of the bar or a component, like `padding`, `margin`, and `border_width`, use a 1-to-4 value syntax. This is the same convention that's common in CSS, for those familiar with that.

Using the example of padding:

- When one value is specified, it applies the same padding to all four sides.
- When two values are specified, the first padding applies to the top and bottom, the second to the left and right.
- When three values are specified, the first padding applies to the top, the second to the right and left, the third to the bottom.
- When four values are specified, the paddings apply to the top, right, bottom, and left in that order (clockwise).

# Known issues

## Blurry buttons in bar window

An app called "Sonic Studio", which is installed by default on ASUS ROG machines can cause rendering issues with WPF apps. This can be resolved by disabling `NahimicService` in Windows Services Manager.
