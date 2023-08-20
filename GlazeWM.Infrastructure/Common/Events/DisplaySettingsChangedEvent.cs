using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record DisplaySettingsChangedEvent() : Event(InfraEvent.DisplaySettingsChanged);
}
