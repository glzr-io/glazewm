using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
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

    public AddWindowHandler(
      Bus bus,
      ContainerService containerService,
      UserConfigService userConfigService,
      CommandParsingService commandParsingService,
      WindowService windowService,
      MonitorService monitorService
    )
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _commandParsingService = commandParsingService;
      _windowService = windowService;
      _monitorService = monitorService;
    }

    public CommandResponse Handle(AddWindowCommand command)
    {
      var windowHandle = command.WindowHandle;
      var targetParent = command.TargetParent;
      var shouldRedraw = command.ShouldRedraw;

      if (!_windowService.IsHandleManageable(windowHandle))
        return CommandResponse.Ok;

      // Get the original width and height of the window.
      var originalPlacement = _windowService.GetPlacementOfHandle(windowHandle).NormalPosition;
      var originalWidth = originalPlacement.Right - originalPlacement.Left;
      var originalHeight = originalPlacement.Bottom - originalPlacement.Top;

      // Create the window instance.
      var window = new TilingWindow(command.WindowHandle, originalWidth, originalHeight);

      var matchingWindowRules = GetMatchingWindowRules(window);

      var commandStrings = matchingWindowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => _commandParsingService.FormatCommand(commandString));

      // Avoid managing a window if a window rule uses 'ignore' command.
      if (commandStrings.Contains("ignore"))
        return CommandResponse.Ok;

      // Attach the new window as a child to the target parent (if provided), otherwise, add as a
      // sibling of the focused container.
      AttachChildWindow(window, targetParent);

      // The OS might spawn the window on a different monitor to the target parent, so adjustments
      // might need to be made because of DPI.
      var monitor = _monitorService.GetMonitorFromHandleLocation(windowHandle);
      if (_monitorService.HasDpiDifference(monitor, window.Parent))
        window.HasPendingDpiAdjustment = true;

      // Set the newly added window as focus descendant. This is necessary because
      // `EVENT_SYSTEM_FOREGROUND` is emitted before `EVENT_OBJECT_SHOW` and thus, focus state
      // isn't updated automatically.
      _bus.Invoke(new SetFocusedDescendantCommand(window));

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

    private void AttachChildWindow(Window window, SplitContainer targetParent)
    {
      if (targetParent != null)
      {
        _bus.Invoke(new AttachAndResizeContainerCommand(window, targetParent));
        return;
      }

      var focusedContainer = _containerService.FocusedContainer;

      // If the focused container is a workspace, attach the window as a child of the workspace.
      if (focusedContainer is Workspace)
      {
        _bus.Invoke(new AttachAndResizeContainerCommand(window, focusedContainer));
        return;
      }

      // Attach the window as a sibling next to the focused window.
      _bus.Invoke(new AttachAndResizeContainerCommand(
        window,
        focusedContainer.Parent,
        focusedContainer.Index + 1
      ));
    }
  }
}
