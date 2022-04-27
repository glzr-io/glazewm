using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class MaximizedWindow : Window
  {
    public MaximizedWindow(
      IntPtr hwnd,
      WindowRect floatingPlacement,
      RectDelta borderDelta
    ) : base(hwnd, floatingPlacement, borderDelta)
    {
    }
  }
}
