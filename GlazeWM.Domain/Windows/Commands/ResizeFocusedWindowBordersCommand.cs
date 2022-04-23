using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ResizeFocusedWindowBordersCommand : Command
  {
    // TODO: Alternate names: ResizeDelta.
    public RectDelta ResizeDimensions { get; }

    public ResizeFocusedWindowBordersCommand(RectDelta resizeRect)
    {
      ResizeDimensions = resizeRect;
    }
  }
}
