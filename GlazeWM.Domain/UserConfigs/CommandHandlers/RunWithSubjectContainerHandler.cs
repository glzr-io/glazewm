using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal sealed class RunWithSubjectContainerHandler
    : ICommandHandler<RunWithSubjectContainerCommand>
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;
    private readonly ContainerService _containerService;

    public RunWithSubjectContainerHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      ContainerService containerService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _containerService = containerService;
    }

    public CommandResponse Handle(RunWithSubjectContainerCommand command)
    {
      var commandStrings = command.CommandStrings;
      var subjectContainer =
        command.SubjectContainer ?? _containerService.FocusedContainer;

      var subjectContainerId = subjectContainer.Id;

      // Return early if any of the commands is an ignore command.
      if (commandStrings.Any(command => command == "ignore"))
        return CommandResponse.Ok;

      // Invoke commands in sequence.
      foreach (var commandString in commandStrings)
      {
        // Avoid calling command if container gets detached. This is to prevent crashes
        // for edge cases like ["close", "move to workspace X"].
        if (subjectContainer?.IsDetached() != false)
          return CommandResponse.Ok;

        var parsedCommand = _commandParsingService.ParseCommand(
          commandString,
          subjectContainer
        );

        // Use `dynamic` to resolve the command type at runtime and allow multiple
        // dispatch.
        _bus.Invoke((dynamic)parsedCommand);

        // Update subject container in case the reference changes (eg. when going from a
        // tiling to a floating window).
        subjectContainer = _containerService.GetContainerById(subjectContainerId);
      }

      return CommandResponse.Ok;
    }
  }
}
