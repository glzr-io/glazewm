using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ResizeWindowBordersCommand : Command
  {
    public Window WindowToResize { get; }
    public RectDelta BorderDelta { get; }

    public ResizeWindowBordersCommand(Window windowToResize, RectDelta borderDelta)
    {
      WindowToResize = windowToResize;
      BorderDelta = borderDelta;
    }
  }
}
