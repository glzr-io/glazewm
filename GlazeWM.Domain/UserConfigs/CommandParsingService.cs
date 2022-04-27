using System;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs
{
  public class CommandParsingService
  {
    private UserConfigService _userConfigService;

    public CommandParsingService(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public string FormatCommand(string commandString)
    {
      var formattedCommandString = commandString.Trim().ToLowerInvariant();
      return Regex.Replace(formattedCommandString, @"\s+", " ");
    }

    public Command ParseCommand(string commandString)
    {
      try
      {
        var commandParts = commandString.Split(" ");

        return commandParts[0] switch
        {
          "layout" => ParseLayoutCommand(commandParts),
          "focus" => ParseFocusCommand(commandParts),
          "move" => ParseMoveCommand(commandParts),
          "resize" => ParseResizeCommand(commandParts),
          "toggle" => ParseToggleCommand(commandParts),
          "exit" => ParseExitCommand(commandParts),
          "close" => new CloseFocusedWindowCommand(),
          _ => throw new ArgumentException(),
        };
      }
      catch
      {
        throw new FatalUserException($"Invalid command '{commandString}'.");
      }
    }

    private Command ParseLayoutCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "vertical" => new ChangeFocusedContainerLayoutCommand(Layout.VERTICAL),
        "horizontal" => new ChangeFocusedContainerLayoutCommand(Layout.HORIZONTAL),
        _ => throw new ArgumentException(),
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
        "workspace" => new FocusWorkspaceCommand(ValidateWorkspaceName(commandParts[2])),
        _ => throw new ArgumentException(),
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
        "to" => new MoveFocusedWindowToWorkspaceCommand(ValidateWorkspaceName(commandParts[3])),
        _ => throw new ArgumentException(),
      };
    }

    private Command ParseResizeCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "height" => new ResizeFocusedWindowCommand(ResizeDimension.HEIGHT, commandParts[2]),
        "width" => new ResizeFocusedWindowCommand(ResizeDimension.WIDTH, commandParts[2]),
        "borders" => new ResizeFocusedWindowBordersCommand(
          ShorthandToRectDelta(string.Join(" ", commandParts[2..]))
        ),
        _ => throw new ArgumentException(),
      };
    }

    private Command ParseToggleCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "floating" => new ToggleFocusedWindowFloatingCommand(),
        "focus" => commandParts[2] switch
        {
          "mode" => new ToggleFocusModeCommand(),
          _ => throw new ArgumentException(),
        },
        _ => throw new ArgumentException(),
      };
    }

    private Command ParseExitCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "wm" => new ExitApplicationCommand(),
        _ => throw new ArgumentException(),
      };
    }

    /// <summary>
    /// Checks whether a workspace exists with the given name.
    /// </summary>
    /// <returns>The workspace name if valid.</returns>
    private string ValidateWorkspaceName(string workspaceName)
    {
      var workspaceConfig = _userConfigService.GetWorkspaceConfigByName(workspaceName);

      if (workspaceConfig == null)
        throw new ArgumentException();

      return workspaceName;
    }

    private RectDelta ShorthandToRectDelta(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ")
        .Select(shorthandPart => UnitsHelper.TrimUnits(shorthandPart))
        .ToList();

      return shorthandParts.Count() switch
      {
        1 => new RectDelta(shorthandParts[0], shorthandParts[0], shorthandParts[0], shorthandParts[0]),
        2 => new RectDelta(shorthandParts[1], shorthandParts[0], shorthandParts[1], shorthandParts[0]),
        3 => new RectDelta(shorthandParts[1], shorthandParts[0], shorthandParts[1], shorthandParts[2]),
        4 => new RectDelta(shorthandParts[3], shorthandParts[0], shorthandParts[1], shorthandParts[2]),
        _ => throw new ArgumentException(),
      };
    }
  }
}
