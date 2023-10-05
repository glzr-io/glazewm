using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.EventHandlers
{
  internal sealed class WindowLocationChangedHandler : IEventHandler<WindowLocationChangedEvent>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly WindowService _windowService;

    public WindowLocationChangedHandler(Bus bus,
      WindowService windowService,
      ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
      _windowService = windowService;
    }

    public void Handle(WindowLocationChangedEvent @event)
    {
      var windowHandle = @event.WindowHandle;

      HandleMaximizedWindow(windowHandle);

      if (!_windowService.AppBarHandles.Contains(windowHandle))
        return;

      _bus.Invoke(new RefreshMonitorStateCommand());
      _bus.Invoke(new RedrawContainersCommand());
    }

    private void HandleMaximizedWindow(IntPtr windowHandle)
    {
      var window = _windowService.GetWindowByHandle(windowHandle);
      if (window is null)
        return;

      var windowPlacement = WindowService.GetPlacementOfHandle(windowHandle);
      var isMaximized = windowPlacement.ShowCommand == ShowWindowFlags.Maximize;

      if (isMaximized && window is not MaximizedWindow)
      {
        var previousState = WindowService.GetWindowType(window);
        var maximizedWindow = new MaximizedWindow(
          window.Handle,
          window.FloatingPlacement,
          window.BorderDelta,
          previousState
        )
        {
          Id = window.Id
        };

        _bus.Invoke(new ReplaceContainerCommand(maximizedWindow, window.Parent, window.Index));
      }

      if (!isMaximized && window is MaximizedWindow)
      {
        var restoredWindow = CreateWindowFromPreviousState(window as MaximizedWindow);

        _bus.Invoke(new ReplaceContainerCommand(restoredWindow, window.Parent, window.Index));

        if (restoredWindow is not TilingWindow)
        {
          // Needed to reposition the floating window
          _bus.Invoke(new RedrawContainersCommand());
          return;
        }

        var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);
        var insertionTarget = workspace.LastFocusedDescendantOfType<IResizable>();

        // Insert the created tiling window after the last focused descendant of the workspace.
        if (insertionTarget == null)
          _bus.Invoke(new MoveContainerWithinTreeCommand(restoredWindow, workspace, 0, true));
        else
          _bus.Invoke(
            new MoveContainerWithinTreeCommand(
              restoredWindow,
              insertionTarget.Parent,
              insertionTarget.Index + 1,
              true
            )
          );

        _containerService.ContainersToRedraw.Add(workspace);
        _bus.Invoke(new RedrawContainersCommand());
      }
    }

    private static Window CreateWindowFromPreviousState(MaximizedWindow window)
    {
      Window restoredWindow = window.PreviousState switch
      {
        WindowType.Floating => new FloatingWindow(
          window.Handle,
          window.FloatingPlacement,
          window.BorderDelta
        ),
        WindowType.Fullscreen => new FullscreenWindow(
          window.Handle,
          window.FloatingPlacement,
          window.BorderDelta
        ),
        // Set `SizePercentage` to 0 to correctly resize the container when moved within tree.
        WindowType.Tiling => new TilingWindow(
          window.Handle,
          window.FloatingPlacement,
          window.BorderDelta,
          0
        ),
        WindowType.Maximized => throw new ArgumentException(null, nameof(window)),
        _ => throw new ArgumentException(null, nameof(window)),
      };

      restoredWindow.Id = window.Id;
      return restoredWindow;
    }
  }
}
