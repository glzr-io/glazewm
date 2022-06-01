using System;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
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
      var formattedCommandString = commandString.Trim().ToLowerInvariant();
      var multipleSpacesRegex = new Regex(@"\s+");
      return multipleSpacesRegex.Replace(formattedCommandString, " ");
    }

    public void ValidateCommand(string commandString)
    {
      try
      {
        ParseCommand(commandString);
      }
      catch
      {
        throw new FatalUserException($"Invalid command '{commandString}'.");
      }
    }

    public Command ParseCommand(string commandString)
    {
      var commandParts = commandString.Split(" ");

      return commandParts[0] switch
      {
        "layout" => ParseLayoutCommand(commandParts),
        "focus" => ParseFocusCommand(commandParts),
        "move" => ParseMoveCommand(commandParts),
        "resize" => ParseResizeCommand(commandParts),
        "set" => ParseSetCommand(commandParts),
        "toggle" => ParseToggleCommand(commandParts),
        "exit" => ParseExitCommand(commandParts),
        "close" => new CloseFocusedWindowCommand(),
        _ => throw new ArgumentException(null, nameof(commandString)),
      };
    }

    private static Command ParseLayoutCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "vertical" => new ChangeFocusedContainerLayoutCommand(Layout.VERTICAL),
        "horizontal" => new ChangeFocusedContainerLayoutCommand(Layout.HORIZONTAL),
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
          new FocusWorkspaceCommand(commandParts[2]),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private Command ParseMoveCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "left" => new MoveFocusedWindowCommand(Direction.LEFT),
        "right" => new MoveFocusedWindowCommand(Direction.RIGHT),
        "up" => new MoveFocusedWindowCommand(Direction.UP),
        "down" => new MoveFocusedWindowCommand(Direction.DOWN),
        "to" when IsValidWorkspace(commandParts[3]) =>
          new MoveFocusedWindowToWorkspaceCommand(commandParts[3]),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseResizeCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "height" => new ResizeFocusedWindowCommand(ResizeDimension.HEIGHT, commandParts[2]),
        "width" => new ResizeFocusedWindowCommand(ResizeDimension.WIDTH, commandParts[2]),
        "borders" => new ResizeFocusedWindowBordersCommand(
          ShorthandToRectDelta(string.Join(" ", commandParts[2..]))
        ),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseSetCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "floating" => new SetFocusedWindowFloatingCommand(),
        _ => throw new ArgumentException(null, nameof(commandParts)),
      };
    }

    private static Command ParseToggleCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "floating" => new ToggleFocusedWindowFloatingCommand(),
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
        "wm" => new ExitApplicationCommand(),
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
