using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.WindowsApi.Events;

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
      _bus.RaiseEvent(new ApplicationExitingEvent());

      return CommandResponse.Ok;
    }
  }
}
