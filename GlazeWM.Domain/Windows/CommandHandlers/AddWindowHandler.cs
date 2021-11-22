using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class AddWindowHandler : ICommandHandler<AddWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private UserConfigService _userConfigService;
    private CommandParsingService _commandParsingService;
    private WindowService _windowService;
    private MonitorService _monitorService;
    private WorkspaceService _workspaceService;

    public AddWindowHandler(
      Bus bus,
      ContainerService containerService,
      UserConfigService userConfigService,
      CommandParsingService commandParsingService,
      WindowService windowService,
      MonitorService monitorService,
      WorkspaceService workspaceService
    )
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _commandParsingService = commandParsingService;
      _windowService = windowService;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(AddWindowCommand command)
    {
      var windowHandle = command.WindowHandle;
      var shouldRedraw = command.ShouldRedraw;

      if (!_windowService.IsHandleManageable(windowHandle))
        return CommandResponse.Ok;

      // Attach the new window as first child of the target parent (if provided), otherwise, add as
      // a sibling of the focused container.
      var (targetParent, targetIndex) = command.TargetParent != null
        ? (command.TargetParent, 0)
        : GetInsertionTarget();

      var originalPlacement = _windowService.GetPlacementOfHandle(windowHandle).NormalPosition;
      var floatingWidth = originalPlacement.Right - originalPlacement.Left;
      var floatingHeight = originalPlacement.Bottom - originalPlacement.Top;

      var targetWorkspace = _workspaceService.GetWorkspaceFromChildContainer(targetParent);

      // Calculate where window should be placed when floating is enabled. Use the original
      // width/height of the window, but position it in the center of the workspace.
      // TODO: This can be simplified. Add utility methods to `WindowRect` struct?
      var floatingPlacement = new WindowRect
      {
        Left = targetWorkspace.X + (targetWorkspace.Width / 2) - (floatingWidth / 2),
        Right = targetWorkspace.X + (targetWorkspace.Width / 2) - (floatingWidth / 2) + floatingWidth,
        Top = targetWorkspace.Y + (targetWorkspace.Height / 2) - (floatingHeight / 2),
        Bottom = targetWorkspace.Y + (targetWorkspace.Height / 2) - (floatingHeight / 2) + floatingHeight,
      };

      // Create the window instance.
      var window = new TilingWindow(command.WindowHandle, floatingPlacement);

      _bus.Invoke(new AttachAndResizeContainerCommand(window, targetParent, targetIndex));

      var matchingWindowRules = GetMatchingWindowRules(window);

      var commandStrings = matchingWindowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => _commandParsingService.FormatCommand(commandString));

      // Avoid managing a window if a window rule uses 'ignore' command.
      if (commandStrings.Contains("ignore"))
        return CommandResponse.Ok;

      // The OS might spawn the window on a different monitor to the target parent, so adjustments
      // might need to be made because of DPI.
      var monitor = _monitorService.GetMonitorFromHandleLocation(windowHandle);
      if (_monitorService.HasDpiDifference(monitor, window.Parent))
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

      // Initialize windows that can't be resized as floating.
      if (!window.HasWindowStyle(WS.WS_THICKFRAME))
        parsedCommands.Insert(0, new ToggleFloatingCommand(window));

      // Invoke commands in the matching window rules.  Use `dynamic` to resolve the command type
      // at runtime and allow multiple dispatch.
      foreach (var parsedCommand in parsedCommands)
        _bus.Invoke((dynamic)parsedCommand);

      if (shouldRedraw)
        _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }

    private (SplitContainer targetParent, int targetIndex) GetInsertionTarget()
    {
      var focusedContainer = _containerService.FocusedContainer;

      if (focusedContainer is Workspace)
        return (focusedContainer as Workspace, 0);

      return (focusedContainer.Parent as SplitContainer, focusedContainer.Index + 1);
    }

    private List<WindowRuleConfig> GetMatchingWindowRules(Window window)
    {
      return _userConfigService.UserConfig.WindowRules
        .Where(rule =>
        {
          if (rule.ProcessNameRegex != null && !rule.ProcessNameRegex.IsMatch(window.ProcessName))
            return false;

          if (rule.ClassNameRegex != null && !rule.ClassNameRegex.IsMatch(window.ClassName))
            return false;

          if (rule.TitleRegex != null && !rule.TitleRegex.IsMatch(window.Title))
            return false;

          return true;
        })
        .ToList();
    }
  }
}
