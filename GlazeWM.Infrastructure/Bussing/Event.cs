using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Infrastructure.Bussing
{
  /// <summary>
  /// An event emitted on the bus.
  /// </summary>
  /// <param name="Type">
  /// Identifier for the type of event. For CLI and IPC usage when subscribing to a
  /// given event type.
  /// </param>
  public abstract record Event(string Type);
}
