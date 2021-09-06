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

        switch (parsedCommand)
        {
          case FocusWorkspaceKeybindingCommand focusWorkspaceCommand:
            _keybindingService.AddGlobalKeybinding(
              keybinding.Bindings[0],
              () => _bus.Invoke(new FocusWorkspaceCommand(focusWorkspaceCommand.WorkspaceName))
            );
            break;
        }
      }

      return CommandResponse.Ok;
    }

    // TODO: Consider changing to return a `Command` and simply invoking directly via bus above.
    private object ParseKeybindingCommand(string commandName)
    {
      var commandString = commandName.Trim().ToLowerInvariant();
      commandString = Regex.Replace(commandString, @"\s+", " ");

      var commandParts = commandString.Split(" ");
      switch (commandString)
      {
        case var a when Regex.IsMatch(commandString, @"focus workspace"):
          var match = Regex.Match(a, @"focus workspace (?<workspaceName>.*?)$");
          // TODO: Check whether a workspace with the name exists (either here or in `FocusWorkspaceHandler`)
          return new FocusWorkspaceKeybindingCommand(match.Groups["workspaceName"].Value);

        // Throw error with message box to user if no command matches.
        // default: throw new ArgumentException();
        default: return 1;
      }
    }
  }
}
