namespace GlazeWM.Infrastructure.Bussing
{
  public class Event
  {
    public string Name => GetType().Name;
  }
}
