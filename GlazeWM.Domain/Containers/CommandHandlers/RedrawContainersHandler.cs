using System;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
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
        .OfType<Window>()
        .Distinct()
        .ToList();

      // Get windows that are minimized/maximized and shouldn't be.
      var windowsToRestore = windowsToRedraw
        .Where(
          window => !(window is MinimizedWindow) && window.HasWindowStyle(WS.WS_MAXIMIZE | WS.WS_MINIMIZE)
        )
        .ToList();

      // Restore minimized/maximized windows. Needed to be able to move and resize them.
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

        SetWindowPosition(window, flags);

        // When there's a mismatch between the DPI of the monitor and the window, `SetWindowPos`
        // might size the window incorrectly. By calling `SetWindowPos` twice, inconsistencies after
        // the first move are resolved.
        if (window.HasPendingDpiAdjustment)
        {
          SetWindowPosition(window, flags);
          window.HasPendingDpiAdjustment = false;
        }
      }

      _containerService.ContainersToRedraw.Clear();

      return CommandResponse.Ok;
    }

    private void SetWindowPosition(Window window, SWP flags)
    {
      if (window is TilingWindow)
      {
        SetWindowPos(
          window.Hwnd,
          IntPtr.Zero,
          window.X - window.InvisibleBorders.Left,
          window.Y - window.InvisibleBorders.Top,
          window.Width + window.InvisibleBorders.Left + window.InvisibleBorders.Right,
          window.Height + window.InvisibleBorders.Top + window.InvisibleBorders.Right,
          flags
        );
        return;
      }

      // Avoid adjusting the borders of floating windows. Otherwise the window will increase in size
      // from its original placement.
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
  }
}
