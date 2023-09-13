namespace GlazeWM.Infrastructure.Utils
{
  public class Countdown
  {
    public TimeSpan _timeSpan { get; init; }
    private DateTime _startTime { get; set; }

    public Countdown(TimeSpan timeSpan)
    {
      _timeSpan = timeSpan;
    }

    public void Start()
    {
      _startTime = DateTime.Now;
    }

    public bool HasElapsed()
    {
      return (DateTime.Now - _startTime).TotalMilliseconds > _timeSpan.Milliseconds;
    }
  }
}
