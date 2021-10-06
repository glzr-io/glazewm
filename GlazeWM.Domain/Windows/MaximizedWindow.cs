using System;

namespace GlazeWM.Domain.Windows
{
  public sealed class MaximizedWindow : Window
  {
    public MaximizedWindow(IntPtr hwnd) : base(hwnd)
    {
    }
  }
}
