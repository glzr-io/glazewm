using System;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ShowAllWindowsHandler : ICommandHandler<ShowAllWindowsCommand>
  {
    private readonly WindowService _windowService;

    public ShowAllWindowsHandler(WindowService windowService)
    {
      _windowService = windowService;
    }

    public CommandResponse Handle(ShowAllWindowsCommand command)
    {
      // Show all windows regardless of whether their workspace is displayed.
      foreach (var window in _windowService.GetWindows())
      {
        const SWP flags = SWP.SWP_FRAMECHANGED | SWP.SWP_NOACTIVATE | SWP.SWP_NOCOPYBITS |
          SWP.SWP_NOZORDER | SWP.SWP_NOOWNERZORDER | SWP.SWP_NOSENDCHANGING | SWP.SWP_SHOWWINDOW;

        SetWindowPos(
          window.Hwnd,
          IntPtr.Zero,
          window.X,
          window.Y,
          window.Width,
          window.Height,
          flags
        );
      }

      return CommandResponse.Ok;
    }
  }
}
