using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Infrastructure.Bussing
{
  public class Event
  {
    public string Name => GetType().Name;

    /// <summary>
    /// Name used to subscribe to for IPC and CLI usage.
    /// </summary>
    public string FriendlyName
    {
      get
      {
        var shortName = Name.Replace("Event", "");
        return CasingUtil.PascalToSnake(shortName);
      }
    }
  }
}
