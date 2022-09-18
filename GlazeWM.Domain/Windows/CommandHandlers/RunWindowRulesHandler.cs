using System.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class RunWindowRulesHandler : ICommandHandler<RunWindowRulesCommand>
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;

    public RunWindowRulesHandler(
      Bus bus,
      CommandParsingService commandParsingService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
    }

    public CommandResponse Handle(RunWindowRulesCommand command)
    {
      var window = command.Window;
      var windowRules = command.WindowRules;

      var parsedCommands = windowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => CommandParsingService.FormatCommand(commandString))
        .Select(commandString => _commandParsingService.ParseCommand(commandString, window))
        .ToList();

      // Invoke commands in the matching window rules. Use `dynamic` to resolve the command type
      // at runtime and allow multiple dispatch.
      foreach (var parsedCommand in parsedCommands)
        _bus.Invoke((dynamic)parsedCommand);

      return CommandResponse.Ok;
    }
  }
}
