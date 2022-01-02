using GlazeWM.Infrastructure.Bussing;
using System;
using Microsoft.Win32;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemEventService
  {
    private Bus _bus;

    public SystemEventService(Bus bus)
    {
      _bus = bus;
    }

    public void Start()
    {
      // TODO: Unsubscribe on application exit.
      SystemEvents.DisplaySettingsChanged += new EventHandler(OnDisplaySettingsChanged);
    }

    private void OnDisplaySettingsChanged(object sender, EventArgs e)
    {
      _bus.RaiseEvent(new DisplaySettingsChangedEvent());
    }
  }
}
