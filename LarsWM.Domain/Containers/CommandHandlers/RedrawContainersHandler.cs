using System;
using System.Linq;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class RedrawContainersHandler : ICommandHandler<RedrawContainersCommand>
  {
    private ContainerService _containerService;
    private UserConfigService _userConfigService;
    private MonitorService _monitorService;

    public RedrawContainersHandler(ContainerService containerService, UserConfigService userConfigService, MonitorService monitorService)
    {
      _containerService = containerService;
      _userConfigService = userConfigService;
      _monitorService = monitorService;
    }

    public dynamic Handle(RedrawContainersCommand command)
    {
      var containersToRedraw = _containerService.SplitContainersToRedraw;

      // Get windows that should be redrawn.
      var windowsToRedraw = containersToRedraw
        .SelectMany(container => container.Flatten())
        .OfType<Window>()
        .Distinct()
        .ToList();

      // Get windows that are minimized or maximized.
      var windowsToRestore = windowsToRedraw
        .Where(window => window.HasWindowStyle(WS.WS_MAXIMIZE | WS.WS_MINIMIZE))
        .ToList();

      // Restore maximized/minimized windows. Needed in order to move and resize them.
      foreach (var window in windowsToRestore)
        ShowWindow(window.Hwnd, ShowWindowFlags.RESTORE);

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

      containersToRedraw.Clear();

      return CommandResponse.Ok;
    }
  }
}
