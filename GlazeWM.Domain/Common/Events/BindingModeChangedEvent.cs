using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Monitors.Events
{
  public class BindingModeChangedEvent : Event
  {
    public string BindingMode { get; }

    public BindingModeChangedEvent(string bindingMode)
    {
      BindingMode = bindingMode;
    }
  }
}
