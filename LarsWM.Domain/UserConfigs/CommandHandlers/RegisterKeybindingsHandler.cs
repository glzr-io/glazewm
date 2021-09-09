using System;
using System.Text.RegularExpressions;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;

namespace LarsWM.Domain.UserConfigs.CommandHandlers
{
  class RegisterKeybindingsHandler : ICommandHandler<RegisterKeybindingsCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private KeybindingService _keybindingService;
    private string _directionRegex = "up|down|left|right";
    private string _moveRegex => $"move {_directionRegex}";

    public RegisterKeybindingsHandler(Bus bus, ContainerService containerService, KeybindingService keybindingService)
    {
      _bus = bus;
      _containerService = containerService;
      _keybindingService = keybindingService;
    }

    public dynamic Handle(RegisterKeybindingsCommand command)
    {
      foreach (var keybinding in command.Keybindings)
      {
        var commandName = FormatCommandName(keybinding.Command);
        var parsedCommand = ParseKeybindingCommand(commandName);

        foreach (var binding in keybinding.Bindings)
          // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
          _keybindingService.AddGlobalKeybinding(binding, () => _bus.Invoke((dynamic)parsedCommand));
      }

      return CommandResponse.Ok;
    }

    private string FormatCommandName(string commandName)
    {
      var formattedCommandString = commandName.Trim().ToLowerInvariant();
      return Regex.Replace(formattedCommandString, @"\s+", " ");
    }

    private Command ParseKeybindingCommand(string commandName)
    {
      var commandParts = commandName.Split(" ");

      return commandParts[0] switch
      {
        "layout" => ParseLayoutKeybindingCommand(commandParts),
        // TODO: Change this to close command once implemented.
        "close" => new FocusWorkspaceCommand("1"),
        "focus" => ParseFocusKeybindingCommand(commandParts),
        "move" => ParseMoveKeybindingCommand(commandParts),
        "resize" => ParseResizeKeybindingCommand(commandParts),
        _ => throw new ArgumentException($"Invalid command {commandName}"),
      };
    }

    private Command ParseLayoutKeybindingCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "vertical" => new ChangeContainerLayoutCommand(Layout.VERTICAL),
        "horizontal" => new ChangeContainerLayoutCommand(Layout.HORIZONTAL),
        _ => throw new ArgumentException(),
      };
    }

    // TODO: Change this to focus command once implemented.
    private Command ParseFocusKeybindingCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "left" => new FocusWorkspaceCommand("1"),
        "right" => new FocusWorkspaceCommand("1"),
        "up" => new FocusWorkspaceCommand("1"),
        "down" => new FocusWorkspaceCommand("1"),
        _ => throw new ArgumentException(),
      };
    }

    private Command ParseMoveKeybindingCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "left" => new MoveFocusedWindowCommand(Direction.LEFT),
        "right" => new MoveFocusedWindowCommand(Direction.RIGHT),
        "up" => new MoveFocusedWindowCommand(Direction.UP),
        "down" => new MoveFocusedWindowCommand(Direction.DOWN),
        _ => throw new ArgumentException(),
      };
    }

    private Command ParseResizeKeybindingCommand(string[] commandParts)
    {
      return commandParts[1] switch
      {
        "left" => new ResizeFocusedWindowCommand(ResizeDirection.SHRINK_WIDTH),
        "right" => new ResizeFocusedWindowCommand(ResizeDirection.GROW_WIDTH),
        "up" => new ResizeFocusedWindowCommand(ResizeDirection.GROW_HEIGHT),
        "down" => new ResizeFocusedWindowCommand(ResizeDirection.SHRINK_HEIGHT),
        _ => throw new ArgumentException(),
      };
    }

    private string ExtractWorkspaceName(string commandName)
    {
      var match = Regex.Match(commandName, @"focus workspace (?<workspaceName>.*?)$");
      return match.Groups["workspaceName"].Value;
    }

    private Direction ExtractDirection(string commandName)
    {
      throw new NotImplementedException();
    }
  }
}
