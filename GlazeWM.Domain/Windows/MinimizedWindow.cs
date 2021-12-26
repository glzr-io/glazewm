using System;
using System.Drawing;

namespace GlazeWM.Domain.Windows
{
  public sealed class MinimizedWindow : Window
  {
    public WindowType PreviousState;

    public MinimizedWindow(
      IntPtr hwnd,
      Rectangle floatingPlacement,
      WindowType previousState
    ) : base(hwnd, floatingPlacement)
    {
      PreviousState = previousState;
    }
  }
}
