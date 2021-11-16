using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class FloatingWindow : Window
  {
    public override int Width => FloatingPlacement.Right - FloatingPlacement.Left;

    public override int Height => FloatingPlacement.Bottom - FloatingPlacement.Top;

    public override int X => FloatingPlacement.Left;

    public override int Y => FloatingPlacement.Top;

    public FloatingWindow(IntPtr hwnd, WindowRect floatingPlacement) : base(hwnd, floatingPlacement)
    {
    }
  }
}
