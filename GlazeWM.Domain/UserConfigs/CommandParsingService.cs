using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Utils;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs
{
  public class CommandParsingService
  {
    private readonly UserConfigService _userConfigService;

    public CommandParsingService(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public static string FormatCommand(string commandString)
    {
      var trimmedCommandString = commandString.Trim().ToLowerInvariant();

      var multipleSpacesRegex = new Regex(@"\s+");
      var formattedCommandString = multipleSpacesRegex.Replace(trimmedCommandString, " ");

      var caseSensitiveCommandRegex = new List<Regex>
      {
        new Regex("^(exec).*", RegexOptions.IgnoreCase),
      };

      // Some commands are partially case-sensitive (eg. `exec ...`). To handle such cases, only
      // format part of the command string to be lowercase.
      foreach (var regex in caseSensitiveCommandRegex)
      {
        if (regex.IsMatch(formattedCommandString))
        {
          return regex.Replace(formattedCommandString, (Match match) =>
            match.Value.ToLowerInvariant()
          );
        }
      }

      return formattedCommandString.ToLowerInvariant();
    }

    public void ValidateCommand(string commandString)
    {
      try
      {
        ParseCommand(commandString, null);
      }
      catch
      {
        throw new FatalUserException($"Invalid command '{commandString}'.");
      }
    }

    public Command ParseCommand(string commandString, Container subjectContainer)
    {
      var commandParts = commandString.Split(" ");

      return commandParts[0] switch
      {
        "tiling" => ParseTilingCommand(commandParts, subjectContainer),
        // TODO: "layout <LAYOUT>" commands are deprecated. Remove in next major release.
        "layout" => ParseLayoutCommand(commandParts, subjectContainer),
        "focus" => ParseFocusCommand(commandParts),
        "move" => ParseMoveCommand(commandParts, subjectContainer),
        "resize" => ParseResizeCommand(commandParts, subjectContainer),
        "set" => ParseSetCommand(commandParts, subjectContainer),
        "toggle" => ParseToggleCommand(commandParts, subjectContainer),
        "exit" => ParseExitCommand(commandParts),
        "close" => subjectContainer is Window
          ? new CloseWindowCommand(subjectContainer as Window)
          : new NoopCommand(),
        "reload" => ParseReloadCommand(commandParts),
        "exec" => new ExecProcessCommand(
          ExtractProcessName(string.Join(" ", commandParts[1..])),
          ExtractProcessArgs(string.Join(" ", commandParts[1..]))
        ),
        "exec_a" => new ExecProcessCommand(
          ExtractProcessAName(string.Join(" ", commandParts[1..])),
          ExtractProcessAArgs(string.Join(" ", commandParts[1..])),
          ExtractProcessAUserName(string.Join(" ", commandParts[1..])),
          ExtractProcessAPassword(string.Join(" ", commandParts[1..]))
        ),
        "ignore" => subjectContainer is Window
          ? new IgnoreWindowCommand(subjectContainer as Window)
          : new NoopCommand(),
        "binding" => ParseBindingCommand(commandParts),
        _ => throw new ArgumentException(null, nameof(commandString)),
      };
    }

    private static Command ParseTilingCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "direction" => commandParts[2] switch
        {
          "vertical" => new ChangeContainerLayoutCommand(subjectContainer, Layout.Vertical),
          "horizontal" => new ChangeContainerLayoutCommand(subjectContainer, Layout.Horizontal),
          "toggle" => new ToggleContainerLayoutCommand(subjectContainer),
          _ => throw new ArgumentException(null, nameof(commandParts)),
        },
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseLayoutCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "vertical" => new ChangeContainerLayoutCommand(subjectContainer, Layout.Vertical),
        "horizontal" => new ChangeContainerLayoutCommand(subjectContainer, Layout.Horizontal),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseFocusCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "left" => new FocusInDirectionCommand(Direction.Left),
        "right" => new FocusInDirectionCommand(Direction.Right),
        "up" => new FocusInDirectionCommand(Direction.Up),
        "down" => new FocusInDirectionCommand(Direction.Down),
        "workspace" => ParseFocusWorkspaceCommand(commandParts),
        "mode" => commandParts[2] switch
        {
          "toggle" => new ToggleFocusModeCommand(),
          _ => throw new ArgumentException(null, nameof(commandParts)),
        },
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseFocusWorkspaceCommand(string[] commandParts)
    {
      return commandParts[2] switch
      {
        "recent" => new FocusWorkspaceRecentCommand(),
        "prev" => new FocusWorkspaceSequenceCommand(Sequence.Previous),
        "next" => new FocusWorkspaceSequenceCommand(Sequence.Next),
        _ when IsValidWorkspace(commandParts[2]) => new FocusWorkspaceCommand(commandParts[2]),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseMoveCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "left" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.Left)
          : new NoopCommand(),
        "right" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.Right)
          : new NoopCommand(),
        "up" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.Up)
          : new NoopCommand(),
        "down" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.Down)
          : new NoopCommand(),
        "to" when IsValidWorkspace(commandParts[3]) => subjectContainer is Window
          ? new MoveWindowToWorkspaceCommand(subjectContainer as Window, commandParts[3])
          : new NoopCommand(),
        "workspace" => ParseMoveWorkspaceCommand(commandParts),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseMoveWorkspaceCommand(string[] commandParts)
    {
      return commandParts[2] switch
      {
        "left" => new MoveWorkspaceInDirectionCommand(Direction.Left),
        "right" => new MoveWorkspaceInDirectionCommand(Direction.Right),
        "up" => new MoveWorkspaceInDirectionCommand(Direction.Up),
        "down" => new MoveWorkspaceInDirectionCommand(Direction.Down),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseResizeCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "height" => subjectContainer is Window
          ? new ResizeWindowCommand(subjectContainer as Window, ResizeDimension.Height, commandParts[2])
          : new NoopCommand(),
        "width" => subjectContainer is Window
          ? new ResizeWindowCommand(subjectContainer as Window, ResizeDimension.Width, commandParts[2])
          : new NoopCommand(),
        "borders" => subjectContainer is Window
          ? new ResizeWindowBordersCommand(
            subjectContainer as Window,
            ShorthandToRectDelta(string.Join(" ", commandParts[2..]))
          )
          : new NoopCommand(),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseSetCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "floating" => subjectContainer is Window
          ? new SetFloatingCommand(subjectContainer as Window)
          : new NoopCommand(),
        "minimized" => subjectContainer is Window
          ? new SetMinimizedCommand(subjectContainer as Window)
          : new NoopCommand(),
        "maximized" => subjectContainer is Window
          ? new SetMaximizedCommand(subjectContainer as Window)
          : new NoopCommand(),
        "tiling" => subjectContainer is Window
          ? new SetTilingCommand(subjectContainer as Window)
          : new NoopCommand(),
        "width" => subjectContainer is Window
          ? new SetWindowSizeCommand(subjectContainer as Window, ResizeDimension.Width, commandParts[2])
          : new NoopCommand(),
        "height" => subjectContainer is Window
          ? new SetWindowSizeCommand(subjectContainer as Window, ResizeDimension.Height, commandParts[2])
          : new NoopCommand(),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseToggleCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "floating" => subjectContainer is Window
          ? new ToggleFloatingCommand(subjectContainer as Window)
          : new NoopCommand(),
        "maximized" => subjectContainer is Window
          ? new ToggleMaximizedCommand(subjectContainer as Window)
          : new NoopCommand(),
        // TODO: "toggle focus mode" command is deprecated. Remove in next major release.
        "focus" => commandParts[2] switch
        {
          "mode" => new ToggleFocusModeCommand(),
          _ => throw new ArgumentException(null, nameof(commandParts)),
        },
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseExitCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "wm" => new ExitApplicationCommand(false),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseBindingCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "mode" when IsValidBindingMode(commandParts[2]) =>
          new SetBindingModeCommand(commandParts[2]),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseReloadCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "config" => new ReloadUserConfigCommand(),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    /// <summary>
    /// Whether a workspace exists with the given name.
    /// </summary>
    private bool IsValidWorkspace(string workspaceName)
    {
      var workspaceConfig = _userConfigService.GetWorkspaceConfigByName(workspaceName);

      return workspaceConfig is not null;
    }

    /// <summary>
    /// Whether a binding mode exists with the given name.
    /// </summary>
    private bool IsValidBindingMode(string bindingModeName)
    {
      var bindingMode = _userConfigService.GetBindingModeByName(bindingModeName);

      return bindingMode is not null || bindingModeName == "none";
    }

    public static string ExtractProcessName(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith(
        "'",
        StringComparison.InvariantCulture
      );

      return hasSingleQuotes
        ? processNameAndArgs.Split("'")[1]
        : processNameAndArgs.Split(" ")[0];
    }

    public static List<string> ExtractProcessArgs(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith(
        "'",
        StringComparison.InvariantCulture
      );

      var args = hasSingleQuotes
        ? processNameAndArgs.Split("'")[2..]
        : processNameAndArgs.Split(" ")[1..];

      return args.Where(arg => !string.IsNullOrWhiteSpace(arg)).ToList();
    }

    public static string ExtractProcessAUserName(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith(
        "'",
        StringComparison.InvariantCulture
      );

      return hasSingleQuotes
        ? processNameAndArgs.Split("'")[1]
        : processNameAndArgs.Split(" ")[0];
    }

    public static string ExtractProcessAPassword(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith(
        "'",
        StringComparison.InvariantCulture
      );

      return hasSingleQuotes
        ? processNameAndArgs.Split("'")[2]
        : processNameAndArgs.Split(" ")[1];
    }

    public static string ExtractProcessAName(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith(
        "'",
        StringComparison.InvariantCulture
      );

      return hasSingleQuotes
        ? processNameAndArgs.Split("'")[3]
        : processNameAndArgs.Split(" ")[2];
    }

    public static List<string> ExtractProcessAArgs(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith(
        "'",
        StringComparison.InvariantCulture
      );

      var args = hasSingleQuotes
        ? processNameAndArgs.Split("'")[4..]
        : processNameAndArgs.Split(" ")[3..];

      return args.Where(arg => !string.IsNullOrWhiteSpace(arg)).ToList();
    }

    private static RectDelta ShorthandToRectDelta(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ")
        .Select(shorthandPart => UnitsHelper.TrimUnits(shorthandPart))
        .ToList();

      return shorthandParts.Count switch
      {
        1 => new RectDelta(shorthandParts[0], shorthandParts[0], shorthandParts[0], shorthandParts[0]),
        2 => new RectDelta(shorthandParts[1], shorthandParts[0], shorthandParts[1], shorthandParts[0]),
        3 => new RectDelta(shorthandParts[1], shorthandParts[0], shorthandParts[1], shorthandParts[2]),
        4 => new RectDelta(shorthandParts[3], shorthandParts[0], shorthandParts[1], shorthandParts[2]),
        _ => throw new ArgumentException(null, nameof(shorthand)),
      };
    }
  }
}
