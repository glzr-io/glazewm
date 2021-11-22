using System;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ShowAllWindowsHandler : ICommandHandler<ShowAllWindowsCommand>
  {
    private WindowService _windowService;

    public ShowAllWindowsHandler(WindowService windowService)
    {
      _windowService = windowService;
    }

    public CommandResponse Handle(ShowAllWindowsCommand command)
    {
      // Reset all managed windows to their floating positions.
      foreach (var window in _windowService.GetWindows())
      {
        var flags = SWP.SWP_FRAMECHANGED | SWP.SWP_NOACTIVATE | SWP.SWP_NOCOPYBITS |
          SWP.SWP_NOZORDER | SWP.SWP_NOOWNERZORDER | SWP.SWP_NOSENDCHANGING | SWP.SWP_SHOWWINDOW;

        SetWindowPos(
          window.Hwnd,
          IntPtr.Zero,
          window.FloatingPlacement.Left,
          window.FloatingPlacement.Top,
          window.FloatingPlacement.Right - window.FloatingPlacement.Left,
          window.FloatingPlacement.Bottom - window.FloatingPlacement.Top,
          flags
        );
      }

      return CommandResponse.Ok;
    }
  }
}
