using System;
using System.Reactive.Linq;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using Microsoft.Win32;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemEventService
  {
    private readonly Bus _bus;

    public SystemEventService(Bus bus)
    {
      _bus = bus;
    }

    public void Start()
    {
      // TODO: Unsubscribe on application exit.
      // TODO: Add as static field instead and subcribe directly in `Startup`. Consider renaming
      // class to `MonitorEvents, `SystemEvents`, `Win32MonitorApi`.
      var displaySettingChanges = Observable.FromEventPattern<EventHandler, EventArgs>(
        handler => SystemEvents.DisplaySettingsChanged += handler,
        handler => SystemEvents.DisplaySettingsChanged -= handler
      );

      displaySettingChanges.Subscribe((_) => _bus.EmitAsync(new DisplaySettingsChangedEvent()));
    }
  }
}
