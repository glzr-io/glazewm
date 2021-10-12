using System;

namespace GlazeWM.Domain.Windows
{
  public sealed class FloatingWindow : Window
  {
    public FloatingWindow(IntPtr hwnd, int originalWidth, int originalHeight, int x, int y) : base(hwnd, originalWidth, originalHeight)
    {
      Width = originalWidth;
      Height = originalHeight;
      X = x;
      Y = y;
    }
  }
}
