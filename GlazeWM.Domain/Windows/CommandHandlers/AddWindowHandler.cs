using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class AddWindowHandler : ICommandHandler<AddWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private UserConfigService _userConfigService;
    private CommandParsingService _commandParsingService;
    private WindowService _windowService;

    public AddWindowHandler(Bus bus, ContainerService containerService, UserConfigService userConfigService, CommandParsingService commandParsingService, WindowService windowService)
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
      _commandParsingService = commandParsingService;
      _windowService = windowService;
    }

    public CommandResponse Handle(AddWindowCommand command)
    {
      var windowHandle = command.WindowHandle;
      var targetParent = command.TargetParent;
      var shouldRedraw = command.ShouldRedraw;

      if (!_windowService.IsHandleManageable(windowHandle))
        return CommandResponse.Ok;

      var window = new TilingWindow(command.WindowHandle);

      var matchingWindowRules = GetMatchingWindowRules(window);

      var commandStrings = matchingWindowRules
        .SelectMany(rule => rule.CommandList)
        .Select(commandString => _commandParsingService.FormatCommand(commandString));

      // Avoid managing a window if a window rule uses 'ignore' command.
      if (commandStrings.Contains("ignore"))
        return CommandResponse.Ok;

      // Store the original width and height of the window.
      var originalPlacement = _windowService.GetPlacementOfHandle(window.Hwnd).NormalPosition;
      window.OriginalWidth = originalPlacement.Right - originalPlacement.Left;
      window.OriginalHeight = originalPlacement.Bottom - originalPlacement.Top;

      // Attach the new window as a child to the target parent (if provided), otherwise, add as a
      // sibling of the focused container.
      AttachChildWindow(window, targetParent);

      // Set focus to newly added window in case it has not been focused automatically.
      _bus.Invoke(new FocusWindowCommand(window));

      var parsedCommands = commandStrings
        .Select(commandString => _commandParsingService.ParseCommand(commandString));

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
          if (rule.ProcessNameRegex != null && !rule.ProcessNameRegex.IsMatch(window.Process.ProcessName))
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
        _bus.Invoke(new AttachContainerCommand(targetParent, window));
        return;
      }

      var focusedContainer = _containerService.FocusedContainer;

      // If the focused container is a workspace, attach the window as a child of the workspace.
      if (focusedContainer is Workspace)
      {
        _bus.Invoke(new AttachContainerCommand(focusedContainer as Workspace, window));
        return;
      }

      // Attach the window as a sibling next to the focused window.
      _bus.Invoke(new AttachContainerCommand(
        focusedContainer.Parent as SplitContainer, window, focusedContainer.Index + 1
      ));
    }
  }
}
