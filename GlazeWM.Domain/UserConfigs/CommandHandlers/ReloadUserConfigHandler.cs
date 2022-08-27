using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Domain.UserConfigs.CommandHandlers
{
  internal class ReloadUserConfigHandler : ICommandHandler<ReloadUserConfigCommand>
  {
    private readonly Bus _bus;

    public ReloadUserConfigHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(ReloadUserConfigCommand command)
    {
      // TODO: Alternate name `CacheContainerStateCommand`.
      _bus.Invoke(new CreateRecoveryCacheCommand());
      _bus.RaiseEvent(new ApplicationRestartingEvent());

      return CommandResponse.Ok;
    }
  }
}
