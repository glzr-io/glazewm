using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class RedrawContainersHandler : ICommandHandler<RedrawContainersCommand>
  {
    private ContainerService _containerService;

    public RedrawContainersHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public CommandResponse Handle(RedrawContainersCommand command)
    {
      // Get windows that should be redrawn.
      var windowsToRedraw = _containerService.ContainersToRedraw
        .SelectMany(container => container.Flatten())
        .OfType<TilingWindow>()
        .Distinct()
        .ToList();

      // Get windows that are minimized or maximized.
      var windowsToRestore = windowsToRedraw
        .Where(window => window.HasWindowStyle(WS.WS_MAXIMIZE | WS.WS_MINIMIZE))
        .ToList();

      // Restore maximized/minimized windows. Needed to be able to move and resize them.
      foreach (var window in windowsToRestore)
        ShowWindow(window.Hwnd, ShowWindowCommands.RESTORE);

      foreach (var window in windowsToRedraw)
      {
        var flags = SWP.SWP_FRAMECHANGED | SWP.SWP_NOACTIVATE | SWP.SWP_NOCOPYBITS |
          SWP.SWP_NOZORDER | SWP.SWP_NOOWNERZORDER | SWP.SWP_NOSENDCHANGING;

        if (window.IsHidden)
          flags |= SWP.SWP_HIDEWINDOW;
        else
          flags |= SWP.SWP_SHOWWINDOW;

        SetWindowPos(window.Hwnd, IntPtr.Zero, window.X, window.Y, window.Width, window.Height, flags);

        // When there's a mismatch between the DPI of the monitor and the window, `SetWindowPos` might
        // size the window incorrectly. By calling `SetWindowPos` twice, inconsistencies after the first
        // move are resolved.
        if (window.HasPendingDpiAdjustment)
        {
          SetWindowPos(window.Hwnd, IntPtr.Zero, window.X, window.Y, window.Width, window.Height, flags);
          window.HasPendingDpiAdjustment = false;
        }
      }

      _containerService.ContainersToRedraw.Clear();

      return CommandResponse.Ok;
    }
  }
}
