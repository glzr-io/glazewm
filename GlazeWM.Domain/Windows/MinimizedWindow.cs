using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class MinimizedWindow : Window
  {
    public MinimizedWindow(
      IntPtr hwnd,
      WindowRect floatingPlacement
    ) : base(hwnd, floatingPlacement)
    {
    }
  }
}
