using System;

namespace GlazeWM.Domain.Windows
{
  public sealed class MaximizedWindow : Window
  {
    public MaximizedWindow(IntPtr hwnd, int originalWidth, int originalHeight) : base(hwnd, originalWidth, originalHeight)
    {
    }
  }
}
