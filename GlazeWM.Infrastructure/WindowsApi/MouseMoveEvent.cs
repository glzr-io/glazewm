using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

public class MouseMoveEvent : Event
{
  /// <summary>
  /// (X,Y) location of mouse with 0,0 being the top-left corner of the main monitor
  /// </summary>
  public Point Point { get; }

  /// <summary>
  /// Whether left-click is currently pressed.
  /// </summary>
  public bool IsMouseDown { get; }

  /// <summary>
  /// The time stamp for this message, equivalent to what `GetMessageTime` would
  /// return.
  /// </summary>
  public int TimeStamp { get; }

  public MouseMoveEvent(Point point, bool isMouseDown, int timeStamp)
  {
    Point = point;
    IsMouseDown = isMouseDown;
    TimeStamp = timeStamp;
  }
}
