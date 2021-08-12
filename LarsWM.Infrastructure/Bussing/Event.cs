namespace LarsWM.Infrastructure.Bussing
{
  public class Event
  {
    public string Name { get; set; }

    public Event()
    {
      Name = GetType().Name;
    }
  }
}
