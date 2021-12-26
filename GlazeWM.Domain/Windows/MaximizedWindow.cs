using System;
using System.Drawing;

namespace GlazeWM.Domain.Windows
{
  public sealed class MaximizedWindow : Window
  {
    public MaximizedWindow(
      IntPtr hwnd,
      Rectangle floatingPlacement
    ) : base(hwnd, floatingPlacement)
    {
    }
  }
}
