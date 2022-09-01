using System;
using System.Linq;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Exceptions;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  internal class PopulateInitialStateHandler : ICommandHandler<PopulateInitialStateCommand>
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private readonly RecoveryCacheService _recoveryCacheService;
    private readonly UserConfigService _userConfigService;
    private readonly WindowService _windowService;
    private readonly WorkspaceService _workspaceService;

    public PopulateInitialStateHandler(Bus bus,
      MonitorService monitorService,
      RecoveryCacheService recoveryCacheService,
      UserConfigService userConfigService,
      WindowService windowService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _recoveryCacheService = recoveryCacheService;
      _userConfigService = userConfigService;
      _windowService = windowService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(PopulateInitialStateCommand command)
    {
      var recoveryCache = _recoveryCacheService.GetRecoveryCache();

      if (recoveryCache?.IsValid() == true && command.AcceptCacheRestore)
        PopulateWithCache(recoveryCache);

      PopulateWithoutCache();

      return CommandResponse.Ok;
    }

    private Container GetAdjustedContainerTree(RecoveryCache recoveryCache)
    {
      var cachedTree = recoveryCache.ContainerTree;
      var cachedMonitors = recoveryCache.ContainerTree.Descendants.Cast<Monitor>();

      return null;
    }

    private void PopulateWithCache(RecoveryCache recoveryCache)
    {
      _bus.Invoke(new EvaluateUserConfigCommand());

      var cachedTree = recoveryCache.ContainerTree;
      var cachedWorkspaces = cachedTree.Descendants.OfType<Workspace>();

      foreach (var screen in System.Windows.Forms.Screen.AllScreens)
      {
        _bus.Invoke(new AddMonitorCommand(screen));
        var addedMonitor = _monitorService.GetMonitors().First(
          monitor => monitor.DeviceName == screen.DeviceName
        );

        // Get cached workspaces belonging to that monitor.
        var workspaceConfigs = _userConfigService.UserConfig.Workspaces;
        var workspacesToActivate = cachedWorkspaces.Where(
          (workspace) =>
            (workspace.Parent as Monitor).DeviceName == screen.DeviceName &&
            workspaceConfigs.Exists(workspaceConfig => workspaceConfig.Name == workspace.Name)
        );

        // Get first inactive workspace and activate it on the monitor.
        if (!workspacesToActivate.Any())
        {
          var inactiveWorkspaceName =
            _workspaceService.GetInactiveWorkspaceNames().ElementAtOrDefault(0);

          if (inactiveWorkspaceName == null)
            throw new FatalUserException("At least 1 workspace is required per monitor.");

          _bus.Invoke(new ActivateWorkspaceCommand(inactiveWorkspaceName, addedMonitor));
          continue;
        }

        foreach (var workspace in workspacesToActivate)
          _bus.Invoke(new ActivateWorkspaceCommand(workspace.Name, addedMonitor));
      }

      // Attach split containers and windows from cached state.
      // (get descendants of workspaces and just attach everything)
      var remainingContainersToAttach = cachedWorkspaces
        .SelectMany(workspace => workspace.Descendants);

      foreach (var container in remainingContainersToAttach)
      {
        if (container is SplitContainer)
          _bus.Invoke(new AttachAndResizeContainerCommand(container, container.));

        if (container is Window)
          _bus.Invoke(new AddWindowCommand(target));
      }

      var uncachedWindowHandles = WindowService.GetAllWindowHandles()
        .Where(handle => handle);

      foreach (var windowHandle in uncachedWindowHandles)
      {
        // Attach window.
        // if (!allWindows.Includes(windowHandle))
      }
    }

    private void PopulateWithoutCache()
    {
      // Read user config file and set its values in state.
      _bus.Invoke(new EvaluateUserConfigCommand());

      // Create a Monitor and consequently a Workspace for each detected Screen. `AllScreens` is an
      // abstraction over `EnumDisplayMonitors` native method.
      foreach (var screen in System.Windows.Forms.Screen.AllScreens)
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
