using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure;
using LarsWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Domain.UserConfigs
{
  public class CommandStringParser
  {
    private List<string> _commandStrings = new List<string>();

    private WorkspaceService _workspaceService = ServiceLocator.Provider.GetRequiredService<WorkspaceService>();

    public bool HasConflict()
    {
      // TODO: Check whether added commands have conflicts.
      return false;
    }

    public void AddCommandStrings(List<string> commandStrings)
    {
      _commandStrings.InsertRange(0, commandStrings);
    }

    private string FormatCommandName(string commandName)
    {
      var formattedCommandString = commandName.Trim().ToLowerInvariant();
      return Regex.Replace(formattedCommandString, @"\s+", " ");
    }

    private Command ParseCommand(string commandName)
    {
      var commandParts = commandName.Split(" ");

      return commandParts[0] switch
      {
        "layout" => ParseLayoutCommand(commandParts),
        "focus" => ParseFocusCommand(commandParts),
        "move" => ParseMoveCommand(commandParts),
        "resize" => ParseResizeCommand(commandParts),
        "close" => new CloseFocusedWindowCommand(),
        _ => throw new ArgumentException(),
      };
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
        "workspace" => new FocusWorkspaceCommand(GetValidWorkspaceName(commandParts[2])),
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
        "to" => new MoveFocusedWindowToWorkspaceCommand(GetValidWorkspaceName(commandParts[3])),
        _ => throw new ArgumentException(),
      };
    }

    private Command ParseResizeCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "grow" => commandParts[2] switch
        {
          "height" => new ResizeFocusedWindowCommand(ResizeDirection.GROW_HEIGHT),
          "width" => new ResizeFocusedWindowCommand(ResizeDirection.GROW_WIDTH),
          _ => throw new ArgumentException(),
        },
        "shrink" => commandParts[2] switch
        {
          "height" => new ResizeFocusedWindowCommand(ResizeDirection.SHRINK_HEIGHT),
          "width" => new ResizeFocusedWindowCommand(ResizeDirection.SHRINK_WIDTH),
          _ => throw new ArgumentException(),
        },
        _ => throw new ArgumentException(),
      };
    }

    private string GetValidWorkspaceName(string workspaceName)
    {
      var workspace = _workspaceService.GetInactiveWorkspaceByName(workspaceName);

      if (workspace == null)
        throw new ArgumentException();

      return workspaceName;
    }
  }
}
