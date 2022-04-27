using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowBordersCommand : Command
  {
    public RectDelta BorderDelta { get; }

    public ResizeFocusedWindowBordersCommand(RectDelta borderDelta)
    {
      BorderDelta = borderDelta;
    }
  }
}
