using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class FullscreenWindow : Window
  {
    public FullscreenWindow(
      IntPtr hwnd,
      WindowRect floatingPlacement
    ) : base(hwnd, floatingPlacement)
    {
    }
  }
}
