using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Infrastructure.Bussing
{
  public abstract class Event
  {
    /// <summary>
    /// Identifier for the type of event. For CLI and IPC usage when subscribing to a
    /// given event type.
    /// </summary>
    public abstract string Type { get; };
  }
}
