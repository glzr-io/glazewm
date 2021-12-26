using System;
using System.Drawing;

namespace GlazeWM.Domain.Windows
{
  public sealed class FloatingWindow : Window
  {
    public override int Width => FloatingPlacement.Right - FloatingPlacement.Left;

    public override int Height => FloatingPlacement.Bottom - FloatingPlacement.Top;

    public override int X => FloatingPlacement.Left;

    public override int Y => FloatingPlacement.Top;

    public FloatingWindow(IntPtr hwnd, Rectangle floatingPlacement) : base(hwnd, floatingPlacement)
    {
    }
  }
}
