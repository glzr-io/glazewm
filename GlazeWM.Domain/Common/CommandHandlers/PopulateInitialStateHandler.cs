using System.Linq;
using System.Windows.Forms;
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
      PopulateWithoutCache();
      return CommandResponse.Ok;
    }

    private void PopulateWithoutCache()
    {
      // Read user config file and set its values in state.
      _bus.Invoke(new EvaluateUserConfigCommand());

      // Create a Monitor and consequently a Workspace for each detected Screen. `AllScreens` is an
      // abstraction over `EnumDisplayMonitors` native method.
      foreach (var screen in Screen.AllScreens)
        _bus.Invoke(new AddMonitorCommand(screen));

      // Add initial windows to the tree.
      foreach (var windowHandle in WindowService.GetAllWindowHandles())
      {
        // Register appbar windows.
        if (_windowService.IsHandleAppBar(windowHandle))
        {
          _windowService.AppBarHandles.Add(windowHandle);
          continue;
        }

        if (!WindowService.IsHandleManageable(windowHandle))
          continue;

        // Get workspace that encompasses most of the window.
        var targetMonitor = _monitorService.GetMonitorFromHandleLocation(windowHandle);
        var targetWorkspace = targetMonitor.DisplayedWorkspace;

        _bus.Invoke(new AddWindowCommand(windowHandle, targetWorkspace, false));
      }

      _bus.Invoke(new RedrawContainersCommand());

      // Get the originally focused window when the WM is started.
      var focusedWindow =
        _windowService.GetWindows().FirstOrDefault(window => window.Hwnd == GetForegroundWindow());

      if (focusedWindow != null)
      {
        _bus.Invoke(new SetFocusedDescendantCommand(focusedWindow));
        _bus.RaiseEvent(new FocusChangedEvent(focusedWindow));
        return;
      }

      // `GetForegroundWindow` might return a handle that is not in the tree. In that case, set
      // focus to an arbitrary window. If there are no manageable windows in the tree, set focus to
      // an arbitrary workspace.
      var containerToFocus =
        _windowService.GetWindows().FirstOrDefault() as Container
        ?? _workspaceService.GetActiveWorkspaces().FirstOrDefault();

      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));
      else if (containerToFocus is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((containerToFocus as Workspace).Name));
    }
  }
}
