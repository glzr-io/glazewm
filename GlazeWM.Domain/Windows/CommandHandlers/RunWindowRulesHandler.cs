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
    private readonly WindowService _windowService;

    public RunWindowRulesHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      WindowService windowService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _windowService = windowService;
    }

    public CommandResponse Handle(RunWindowRulesCommand command)
    {
      var window = command.Window;
      var windowHandle = window.Handle;
      var windowRules = command.WindowRules;

      var commandStrings = windowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => CommandParsingService.FormatCommand(commandString));

      var subjectWindow = window;
      foreach (var commandString in commandStrings)
      {
        var parsedCommand = _commandParsingService.ParseCommand(commandString, subjectWindow);

        // Invoke commands in the matching window rules. Use `dynamic` to resolve the command type
        // at runtime and allow multiple dispatch.
        _bus.Invoke((dynamic)parsedCommand);

        // Update subject window in case the reference changes (eg. when going from a tiling to a
        // floating window).
        subjectWindow = _windowService.GetWindowByHandle(windowHandle);
      }

      return CommandResponse.Ok;
    }
  }
}
