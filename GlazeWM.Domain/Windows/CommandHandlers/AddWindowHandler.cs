using System;
using System.Linq;
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
  internal class AddWindowHandler : ICommandHandler<AddWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;
    private readonly CommandParsingService _commandParsingService;
    private readonly WindowService _windowService;
    private readonly MonitorService _monitorService;
    private readonly WorkspaceService _workspaceService;
    private readonly ILogger<AddWindowHandler> _logger;

    public AddWindowHandler(
      Bus bus,
      ContainerService containerService,
      UserConfigService userConfigService,
      CommandParsingService commandParsingService,
      WindowService windowService,
      MonitorService monitorService,
      WorkspaceService workspaceService,
      ILogger<AddWindowHandler> logger
    )
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _commandParsingService = commandParsingService;
      _windowService = windowService;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
      _logger = logger;
    }

    public CommandResponse Handle(AddWindowCommand command)
    {
      var windowHandle = command.WindowHandle;
      var shouldRedraw = command.ShouldRedraw;

      // Attach the new window as first child of the target parent (if provided), otherwise, add as
      // a sibling of the focused container.
      var (targetParent, targetIndex) = command.TargetParent != null
        ? (command.TargetParent, 0)
        : GetInsertionTarget();

      // Create the window instance.
      var targetWorkspace = WorkspaceService.GetWorkspaceFromChildContainer(targetParent);
      var window = CreateWindow(windowHandle, targetWorkspace);

      var matchingWindowRules = _userConfigService.GetMatchingWindowRules(window);

      var commandStrings = matchingWindowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => CommandParsingService.FormatCommand(commandString));

      // Avoid managing a window if a window rule uses 'ignore' command.
      if (commandStrings.Contains("ignore"))
        return CommandResponse.Ok;

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

      // Set the newly added window as focus descendant. This is necessary because
      // `EVENT_SYSTEM_FOREGROUND` is emitted before `EVENT_OBJECT_SHOW` and thus, focus state
      // isn't updated automatically.
      _bus.Invoke(new SetFocusedDescendantCommand(window));

      // Set OS focus to the newly added window in case it's not already focused. This is also
      // necessary for window rule commands to run properly on startup with existing windows.
      _bus.Invoke(new FocusWindowCommand(window));

      var parsedCommands = commandStrings
        .Select(commandString => _commandParsingService.ParseCommand(commandString))
        .ToList();

      // Invoke commands in the matching window rules.  Use `dynamic` to resolve the command type
      // at runtime and allow multiple dispatch.
      foreach (var parsedCommand in parsedCommands)
        _bus.Invoke((dynamic)parsedCommand);

      if (shouldRedraw)
        _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private static Window CreateWindow(IntPtr windowHandle, Workspace targetWorkspace)
    {
      // Calculate where window should be placed when floating is enabled. Use the original
      // width/height of the window, but position it in the center of the workspace.
      var originalPlacement = WindowService.GetPlacementOfHandle(windowHandle).NormalPosition;
      var floatingPlacement = originalPlacement.TranslateToCenter(targetWorkspace.ToRectangle());

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
