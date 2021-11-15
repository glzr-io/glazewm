using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class FloatingWindow : Window
  {
    public FloatingWindow(IntPtr hwnd, WindowRect floatingPlacement) : base(hwnd, floatingPlacement)
    {
    }
  }
}
