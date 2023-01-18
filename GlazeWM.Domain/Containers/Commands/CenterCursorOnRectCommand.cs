using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Containers.Commands
{
  public class CenterCursorOnRectCommand : Command
  {
    public Rect TargetRect { get; }

    /// <summary>
    ///  Center cursor in the middle of target container
    /// </summary>
    public CenterCursorOnRectCommand(Rect target)
    {
      TargetRect = target;
    }
  }
}
