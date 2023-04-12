using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Containers.Commands
{
  public class FocusContainerUnderCursorCommand : Command
  {
    public Point TargetPoint { get; }
    public FocusContainerUnderCursorCommand(Point targetPoint)
    {
      TargetPoint = targetPoint;
    }
  }
}
