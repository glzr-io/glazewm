using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class RedrawContainersHandler : ICommandHandler<RedrawContainersCommand>
  {
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public RedrawContainersHandler(
      ContainerService containerService,
      UserConfigService userConfigService)
    {
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(RedrawContainersCommand command)
    {
      // Get windows that should be redrawn.
      var windowsToRedraw = _containerService.ContainersToRedraw
        .SelectMany(container => container.Flatten())
        .OfType<Window>()
        .Distinct()
        .ToList();

      // Get windows that are minimized/maximized and shouldn't be.
      var windowsToRestore = windowsToRedraw
        .Where(
          (window) =>
            window is not MinimizedWindow &&
            window.HasWindowStyle(WS.WS_MAXIMIZE | WS.WS_MINIMIZE)
        )
        .ToList();

      // Restore minimized/maximized windows. Needed to be able to move and resize them.
      foreach (var window in windowsToRestore)
        ShowWindow(window.Handle, ShowWindowCommands.RESTORE);

      foreach (var window in windowsToRedraw)
      {
        SetWindowPosition(window);

        // When there's a mismatch between the DPI of the monitor and the window,
        // `SetWindowPos` might size the window incorrectly. By calling `SetWindowPos`
        // twice, inconsistencies after the first move are resolved.
        if (window.HasPendingDpiAdjustment)
        {
          SetWindowPosition(window);
          window.HasPendingDpiAdjustment = false;
        }
      }

      _containerService.ContainersToRedraw.Clear();

      return CommandResponse.Ok;
    }

    private void SetWindowPosition(Window window)
    {
      var defaultFlags =
        SWP.SWP_FRAMECHANGED |
        SWP.SWP_NOACTIVATE |
        SWP.SWP_NOCOPYBITS |
        SWP.SWP_NOSENDCHANGING;

      // Show or hide the window depending on whether the workspace is displayed.
      if (window.IsDisplayed)
        defaultFlags |= SWP.SWP_SHOWWINDOW;
      else
        defaultFlags |= SWP.SWP_HIDEWINDOW;

      if (window is TilingWindow)
      {
        SetWindowPos(
          window.Handle,
          new IntPtr((int)ZOrderFlags.NoTopMost),
          window.X - window.BorderDelta.Left,
          window.Y - window.BorderDelta.Top,
          window.Width + window.BorderDelta.Left + window.BorderDelta.Right,
          window.Height + window.BorderDelta.Top + window.BorderDelta.Right,
          defaultFlags
        );
        return;
      }

      // Get z-order to set for floating windows.
      var shouldShowOnTop = _userConfigService.GeneralConfig.ShowFloatingOnTop;
      var floatingZOrder = shouldShowOnTop
        ? ZOrderFlags.TopMost
        : ZOrderFlags.NoTopMost;

      // Avoid adjusting the borders of floating windows. Otherwise the window will
      // increase in size from its original placement.
      SetWindowPos(
        window.Handle,
        new IntPtr((int)floatingZOrder),
        window.X,
        window.Y,
        window.Width,
        window.Height,
        defaultFlags
      );
    }
  }
}
