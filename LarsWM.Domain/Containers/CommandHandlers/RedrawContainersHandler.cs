using System;
using System.Collections.Generic;
using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers.Commands;
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

    public RedrawContainersHandler(ContainerService containerService, UserConfigService userConfigService)
    {
      _containerService = containerService;
      _userConfigService = userConfigService;
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

      var handle = BeginDeferWindowPos(windowsToRedraw.Count());

      foreach (var window in windowsToRedraw)
      {
        var flags = SWP.SWP_FRAMECHANGED | SWP.SWP_NOACTIVATE | SWP.SWP_NOCOPYBITS |
            SWP.SWP_NOZORDER | SWP.SWP_NOOWNERZORDER;

        if (window.IsHidden)
          flags |= SWP.SWP_HIDEWINDOW;
        else
          flags |= SWP.SWP_SHOWWINDOW;

        DeferWindowPos(handle, window.Hwnd, IntPtr.Zero, window.X, window.Y, window.Width, window.Height, flags);
      }

      EndDeferWindowPos(handle);

      containersToRedraw.Clear();

      return CommandResponse.Ok;
    }
  }
}
