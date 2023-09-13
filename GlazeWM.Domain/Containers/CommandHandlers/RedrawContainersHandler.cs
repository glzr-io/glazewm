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
      // Get windows that should be redrawn. When redrawing after a keybinding that
      // changes a window's type (eg. tiling -> floating), the original detached window
      // might still be queued for a redraw and should be ignored.
      var windowsToRedraw = _containerService.ContainersToRedraw
        .SelectMany(container => container.Flatten())
        .OfType<Window>()
        .Distinct()
        .Where(window => !window.IsDetached())
        .ToList();

      // Get windows that are minimized/maximized and shouldn't be.
      var windowsToRestore = windowsToRedraw
        .Where(
          (window) =>
            window is not MinimizedWindow &&
            window.HasWindowStyle(WindowStyles.Maximize | WindowStyles.Minimize)
        )
        .ToList();

      // Restore minimized/maximized windows. Needed to be able to move and resize them.
      foreach (var window in windowsToRestore)
        ShowWindow(window.Handle, ShowWindowFlags.Restore);

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
        SetWindowPosFlags.FrameChanged |
        SetWindowPosFlags.NoActivate |
        SetWindowPosFlags.NoCopyBits |
        SetWindowPosFlags.NoSendChanging;

      // Show or hide the window depending on whether the workspace is displayed.
      if (window.IsDisplayed)
        defaultFlags |= SetWindowPosFlags.ShowWindow;
      else
        defaultFlags |= SetWindowPosFlags.HideWindow;

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
