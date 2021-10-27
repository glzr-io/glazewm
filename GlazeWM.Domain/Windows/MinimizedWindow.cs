using System;

namespace GlazeWM.Domain.Windows
{
  public sealed class MinimizedWindow : Window
  {
    public MinimizedWindow(IntPtr hwnd, int originalWidth, int originalHeight) : base(hwnd, originalWidth, originalHeight)
    {
    }
  }
}
