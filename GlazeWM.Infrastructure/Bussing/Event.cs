using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Infrastructure.Bussing
{
  public class Event
  {
    public string Name => GetType().Name;

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
