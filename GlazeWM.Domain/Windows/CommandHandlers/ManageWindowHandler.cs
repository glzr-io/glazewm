using System;
using GlazeWM.Domain.Common.Utils;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.Logging;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class ManageWindowHandler : ICommandHandler<ManageWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly ILogger<ManageWindowHandler> _logger;
    private readonly MonitorService _monitorService;
    private readonly UserConfigService _userConfigService;

    public ManageWindowHandler(
      Bus bus,
      ContainerService containerService,
      ILogger<ManageWindowHandler> logger,
      MonitorService monitorService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _containerService = containerService;
      _logger = logger;
      _monitorService = monitorService;
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(ManageWindowCommand command)
    {
      var windowHandle = command.WindowHandle;
      var shouldRedraw = command.ShouldRedraw;

      // Attach the new window as first child of the target parent (if provided), otherwise, add as
      // a sibling of the focused container.
      var (targetParent, targetIndex) = command.TargetParent != null
        ? (command.TargetParent, 0)
        : GetInsertionTarget();

      // Create the window instance.
      var window = CreateWindow(windowHandle, targetParent);

      _logger.LogWindowEvent("New window managed", window);

      if (window is IResizable)
        _bus.Invoke(new AttachAndResizeContainerCommand(window, targetParent, targetIndex));
      else
        _bus.Invoke(new AttachContainerCommand(window, targetParent, targetIndex));

      // The OS might spawn the window on a different monitor to the target parent, so adjustments
      // might need to be made because of DPI.
      var monitor = _monitorService.GetMonitorFromHandleLocation(windowHandle);
      if (MonitorService.HasDpiDifference(monitor, window.Parent))
        window.HasPendingDpiAdjustment = true;

      // Set the newly added window as focus descendant. This means the window rules will be run as
      // if the window is focused.
      _bus.Invoke(new SetFocusedDescendantCommand(window));

      // Run matching window rules.
      var windowRules = _userConfigService.GetMatchingWindowRules(window);
      _bus.Invoke(new RunWindowRulesCommand(window, windowRules));

      // Window might be detached if 'ignore' command has been invoked.
      if (window.IsDetached())
        return CommandResponse.Ok;

      if (shouldRedraw)
        _bus.Invoke(new RedrawContainersCommand());

      // Set OS focus to the newly added window in case it's not already focused.
      _bus.Invoke(new SetNativeFocusCommand(window));

      return CommandResponse.Ok;
    }

    private static Window CreateWindow(IntPtr windowHandle, Container targetParent)
    {
      var originalPlacement = WindowService.GetPlacementOfHandle(windowHandle).NormalPosition;

      // Calculate where window should be placed when floating is enabled. Use the original
      // width/height of the window, but position it in the center of the workspace.
      var targetWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(targetParent);
      var floatingPlacement = originalPlacement.TranslateToCenter(targetWorkspace.ToRect());

      var defaultBorderDelta = new RectDelta(7, 0, 7, 7);

      var windowType = GetWindowTypeToCreate(windowHandle);
      var isResizable = WindowService.HandleHasWindowStyle(windowHandle, WS.WS_THICKFRAME);

      // TODO: Handle initialization of maximized and fullscreen windows.
      return windowType switch
      {
        WindowType.MINIMIZED => new MinimizedWindow(
          windowHandle,
          floatingPlacement,
          defaultBorderDelta,
          isResizable ? WindowType.TILING : WindowType.FLOATING
        ),
        WindowType.FLOATING => new FloatingWindow(
          windowHandle,
          floatingPlacement,
          defaultBorderDelta
        ),
        WindowType.TILING => new TilingWindow(
          windowHandle,
          floatingPlacement,
          defaultBorderDelta
        ),
        WindowType.MAXIMIZED => throw new ArgumentException(null, nameof(windowHandle)),
        WindowType.FULLSCREEN => throw new ArgumentException(null, nameof(windowHandle)),
        _ => throw new ArgumentException(null, nameof(windowHandle)),
      };
    }

    private static WindowType GetWindowTypeToCreate(IntPtr windowHandle)
    {
      if (WindowService.HandleHasWindowStyle(windowHandle, WS.WS_MINIMIZE))
        return WindowType.MINIMIZED;

      // Initialize windows that can't be resized as floating.
      if (!WindowService.HandleHasWindowStyle(windowHandle, WS.WS_THICKFRAME))
        return WindowType.FLOATING;

      return WindowType.TILING;
    }

    private (SplitContainer targetParent, int targetIndex) GetInsertionTarget()
    {
      var focusedContainer = _containerService.FocusedContainer;

      if (focusedContainer is Workspace)
        return (focusedContainer as Workspace, 0);

      return (focusedContainer.Parent as SplitContainer, focusedContainer.Index + 1);
    }
  }
}
