using System;

namespace GlazeWM.Domain.Windows
{
  public sealed class FloatingWindow : Window
  {
    public FloatingWindow(IntPtr hwnd) : base(hwnd)
    {
    }
  }
}
