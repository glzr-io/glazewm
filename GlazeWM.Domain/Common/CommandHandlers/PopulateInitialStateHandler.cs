using System.Linq;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  internal class PopulateInitialStateHandler : ICommandHandler<PopulateInitialStateCommand>
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private readonly WindowService _windowService;
    private readonly WorkspaceService _workspaceService;

    public PopulateInitialStateHandler(Bus bus,
      MonitorService monitorService,
      WindowService windowService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _windowService = windowService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(PopulateInitialStateCommand command)
    {
      // Read user config file and set its values in state.
      _bus.Invoke(new EvaluateUserConfigCommand());

      var focusedHandle = GetForegroundWindow();
      PopulateContainerTree();

      // Register appbar windows.
      foreach (var windowHandle in WindowService.GetAllWindowHandles())
        if (_windowService.IsHandleAppBar(windowHandle))
          _windowService.AppBarHandles.Add(windowHandle);

      _bus.Invoke(new RedrawContainersCommand());

      // Get the originally focused window when the WM is started.
      var focusedWindow =
        _windowService.GetWindows().FirstOrDefault(window => window.Handle == focusedHandle);

      // `GetForegroundWindow` might return a handle that is not in the tree. In that case, set
      // focus to an arbitrary window. If there are no manageable windows in the tree, set focus to
      // an arbitrary workspace.
      var containerToFocus =
        focusedWindow ??
        _windowService.GetWindows().FirstOrDefault() ??
        _workspaceService.GetActiveWorkspaces().FirstOrDefault() as Container;

      _bus.Invoke(new SetFocusedDescendantCommand(containerToFocus));
      _bus.Emit(new FocusChangedEvent(containerToFocus));

      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));
      else if (containerToFocus is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((containerToFocus as Workspace).Name));

      return CommandResponse.Ok;
    }

    private void PopulateContainerTree()
    {
      // Create a Monitor and consequently a Workspace for each detected Screen. `AllScreens` is an
      // abstraction over `EnumDisplayMonitors` native method.
      foreach (var screen in System.Windows.Forms.Screen.AllScreens)
        _bus.Invoke(new AddMonitorCommand(screen));

      // Add initial windows to the tree.
      // TODO: Copy all the below over to populate with cache method, but filter out window handles
      // that have already been added to state.
      foreach (var windowHandle in WindowService.GetAllWindowHandles())
      {
        if (!WindowService.IsHandleManageable(windowHandle))
          continue;

        // Get workspace that encompasses most of the window.
        var targetMonitor = _monitorService.GetMonitorFromHandleLocation(windowHandle);
        var targetWorkspace = targetMonitor.DisplayedWorkspace;

        _bus.Invoke(new ManageWindowCommand(windowHandle, targetWorkspace, false));
      }
    }
  }
}
