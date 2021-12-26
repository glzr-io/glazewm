using System;
using System.Drawing;

namespace GlazeWM.Domain.Windows
{
  public sealed class FullscreenWindow : Window
  {
    public FullscreenWindow(
      IntPtr hwnd,
      Rectangle floatingPlacement
    ) : base(hwnd, floatingPlacement)
    {
    }
  }
}
