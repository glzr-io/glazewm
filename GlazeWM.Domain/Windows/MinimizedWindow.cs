using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class MinimizedWindow : Window
  {
    public WindowType PreviousState;

    public MinimizedWindow(
      IntPtr hwnd,
      WindowRect floatingPlacement,
      RectDelta borderDelta,
      WindowType previousState
    ) : base(hwnd, floatingPlacement, borderDelta)
    {
      PreviousState = previousState;
    }
  }
}
