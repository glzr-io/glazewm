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
    private readonly UserConfigService _userConfigService;

    public RunWindowRulesHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(RunWindowRulesCommand command)
    {
      var window = command.Window;

      var matchingWindowRules = _userConfigService.GetMatchingWindowRules(window);

      var commandStrings = matchingWindowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => CommandParsingService.FormatCommand(commandString));

      var parsedCommands = commandStrings
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
