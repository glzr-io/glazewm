using System.Text.RegularExpressions;
using LarsWM.Domain.Containers;
using LarsWM.Domain.UserConfigs.Commands;
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
        var parsedCommand = ParseKeybindingCommand(keybinding.Command);

        foreach (var binding in keybinding.Bindings)
          // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
          _keybindingService.AddGlobalKeybinding(binding, () => _bus.Invoke((dynamic)parsedCommand));
      }

      return CommandResponse.Ok;
    }

    private Command ParseKeybindingCommand(string commandName)
    {
      var commandString = commandName.Trim().ToLowerInvariant();
      commandString = Regex.Replace(commandString, @"\s+", " ");

      var commandParts = commandString.Split(" ");
      switch (commandString)
      {
        case var _ when Regex.IsMatch(commandString, @"focus workspace"):
          var match = Regex.Match(commandString, @"focus workspace (?<workspaceName>.*?)$");
          // TODO: Check whether a workspace with the name exists (either here or in `FocusWorkspaceHandler`)
          return new FocusWorkspaceCommand(match.Groups["workspaceName"].Value);

        // Throw error with message box to user if no command matches.
        // default: throw new ArgumentException();
        default: return new FocusWorkspaceCommand("1");
      }
    }
  }
}
