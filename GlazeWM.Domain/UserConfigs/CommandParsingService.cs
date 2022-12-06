using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Common.Utils;
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
        "ignore" => subjectContainer is Window
          ? new IgnoreWindowCommand(subjectContainer as Window)
          : new NoopCommand(),
        _ => throw new ArgumentException(null, nameof(commandString)),
      };
    }

    private static Command ParseLayoutCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "vertical" => new ChangeContainerLayoutCommand(subjectContainer, Layout.VERTICAL),
        "horizontal" => new ChangeContainerLayoutCommand(subjectContainer, Layout.HORIZONTAL),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseFocusCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "left" => new FocusInDirectionCommand(Direction.LEFT),
        "right" => new FocusInDirectionCommand(Direction.RIGHT),
        "up" => new FocusInDirectionCommand(Direction.UP),
        "down" => new FocusInDirectionCommand(Direction.DOWN),
        "workspace" when IsValidWorkspace(commandParts[2]) =>
          ParseFocusWorkspaceCommand(commandParts),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseFocusWorkspaceCommand(string[] commandParts)
    {
      return commandParts[2] switch
      {
        "recent" => new FocusWorkspaceRecentCommand(),
        "prev" => new FocusWorkspaceSequenceCommand(Sequence.PREVIOUS),
        "next" => new FocusWorkspaceSequenceCommand(Sequence.NEXT),
        // errors already checked at the previous level parsing
        _  => new FocusWorkspaceCommand(commandParts[2]),
      };
    }

    private Command ParseMoveCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "left" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.LEFT)
          : new NoopCommand(),
        "right" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.RIGHT)
          : new NoopCommand(),
        "up" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.UP)
          : new NoopCommand(),
        "down" => subjectContainer is Window
          ? new MoveWindowCommand(subjectContainer as Window, Direction.DOWN)
          : new NoopCommand(),
        "to" when IsValidWorkspace(commandParts[3]) => subjectContainer is Window
          ? new MoveWindowToWorkspaceCommand(subjectContainer as Window, commandParts[3])
          : new NoopCommand(),
        "workspace" => ParseMoveWorkspaceCommand(commandParts, subjectContainer),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseMoveWorkspaceCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[2] switch
      {
        "left" => new MoveWorkspaceInDirectionCommand(Direction.LEFT),
        "right" => new MoveWorkspaceInDirectionCommand(Direction.RIGHT),
        "up" => new MoveWorkspaceInDirectionCommand(Direction.UP),
        "down" => new MoveWorkspaceInDirectionCommand(Direction.DOWN),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseResizeCommand(string[] commandParts, Container subjectContainer)
    {
      return commandParts[1] switch
      {
        "height" => subjectContainer is Window
          ? new ResizeWindowCommand(subjectContainer as Window, ResizeDimension.HEIGHT, commandParts[2])
          : new NoopCommand(),
        "width" => subjectContainer is Window
          ? new ResizeWindowCommand(subjectContainer as Window, ResizeDimension.WIDTH, commandParts[2])
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

    private static Command ParseReloadCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "config" => new ReloadUserConfigCommand(),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    /// <summary>
    /// Whether a workspace exists with the given name 
    /// or workspace name is part of focus workspace command.
    /// </summary>
    private bool IsValidWorkspace(string workspaceName)
    {
      if (Keywords.WorkspaceKeyswords.Contains(workspaceName))
      {
        return true;
      }

      var workspaceConfig = _userConfigService.GetWorkspaceConfigByName(workspaceName);

      return workspaceConfig is not null;
    }

    public static string ExtractProcessName(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith("'");

      return hasSingleQuotes
        ? processNameAndArgs.Split("'")[1]
        : processNameAndArgs.Split(" ")[0];
    }

    public static List<string> ExtractProcessArgs(string processNameAndArgs)
    {
      var hasSingleQuotes = processNameAndArgs.StartsWith("'");

      var args = hasSingleQuotes
        ? processNameAndArgs.Split("'")[2..]
        : processNameAndArgs.Split(" ")[1..];

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
