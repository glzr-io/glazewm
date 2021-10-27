using System;

namespace GlazeWM.Domain.Windows
{
  public sealed class FullscreenWindow : Window
  {
    public FullscreenWindow(IntPtr hwnd, int originalWidth, int originalHeight) : base(hwnd, originalWidth, originalHeight)
    {
    }
  }
}
