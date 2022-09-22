using System;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Common.Events;

namespace GlazeWM.Infrastructure.Common.CommandHandlers
{
  internal class ExitApplicationHandler : ICommandHandler<ExitApplicationCommand>
  {
    private readonly Bus _bus;

    public ExitApplicationHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(ExitApplicationCommand command)
    {
      var withErrorCode = command.WithErrorCode;

      // Signal that application is about to exit (to perform cleanup).
      _bus.Emit(new ApplicationExitingEvent());

      // Use exit code 1 if exiting due to an exception.
      Environment.Exit(withErrorCode ? 1 : 0);

      return CommandResponse.Ok;
    }
  }
}
