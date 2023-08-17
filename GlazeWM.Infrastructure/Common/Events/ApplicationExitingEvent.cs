using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Events
{
  public record ApplicationExitingEvent : Event(InfraEvent.ApplicationExiting);
}
