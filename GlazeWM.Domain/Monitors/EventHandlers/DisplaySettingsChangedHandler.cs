using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;

namespace GlazeWM.Domain.Monitors.EventHandlers
{
  internal sealed class DisplaySettingsChangedHandler : IEventHandler<DisplaySettingsChangedEvent>
  {
    private readonly Bus _bus;

    public DisplaySettingsChangedHandler(Bus bus)
    {
      _bus = bus;
    }

    public void Handle(DisplaySettingsChangedEvent @event)
    {
      _bus.Invoke(new RefreshMonitorStateCommand());
    }
  }
}
