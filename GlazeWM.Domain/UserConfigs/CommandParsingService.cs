using System;
using System.Linq;
using System.Text.RegularExpressions;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Bussing.Commands;
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
          new FocusWorkspaceCommand(commandParts[2]),
        _ => throw new ArgumentException(null, nameof(commandParts)),
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
