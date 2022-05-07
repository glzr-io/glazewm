using GlazeWM.Bar;
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
using GlazeWM.Infrastructure.WindowsApi;
using GlazeWM.Infrastructure.WindowsApi.Events;
using System;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Forms;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bootstrapper
{
  class Startup
  {
    private readonly Bus _bus;
    private readonly MonitorService _monitorService;
    private readonly KeybindingService _keybindingService;
    private readonly WindowEventService _windowEventService;
    private readonly WindowService _windowService;
    private readonly BarService _barService;
    private readonly WorkspaceService _workspaceService;
    private readonly SystemTrayService _systemTrayService;
    private readonly SystemEventService _systemEventService;

    public Startup(
      Bus bus,
      MonitorService monitorService,
      KeybindingService keybindingService,
      WindowEventService windowEventService,
      WindowService windowService,
      BarService barService,
      WorkspaceService workspaceService,
      SystemTrayService systemTrayService,
      SystemEventService systemEventService
    )
    {
      _bus = bus;
      _monitorService = monitorService;
      _keybindingService = keybindingService;
      _windowEventService = windowEventService;
      _windowService = windowService;
      _barService = barService;
      _workspaceService = workspaceService;
      _systemTrayService = systemTrayService;
      _systemEventService = systemEventService;
    }

    public void Run()
    {
      // Set the process-default DPI awareness.
      _ = SetProcessDpiAwarenessContext(DpiAwarenessContext.Context_PerMonitorAwareV2);

      // Launch bar WPF application. Spawns bar window when monitors are added, so the service needs
      // to be initialized before populating initial state.
      _barService.StartApp();

      // Populate initial monitors, windows, workspaces and user config.
      PopulateInitialState();

      // Listen on registered keybindings.
      _keybindingService.Start();

      // Listen for window events (eg. close, focus).
      _windowEventService.Start();

      // Listen for system-related events (eg. changes to display settings).
      _systemEventService.Start();

      // Add application to system tray.
      _systemTrayService.AddToSystemTray();

      _bus.Events.Where(@event => @event is ApplicationExitingEvent)
        .Subscribe(_ => OnApplicationExit());
    }

    /// <summary>
    /// Populate initial monitors, windows, workspaces and user config.
    /// </summary>
    private void PopulateInitialState()
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
      Container containerToFocus =
        _windowService.GetWindows().FirstOrDefault() as Container
        ?? _workspaceService.GetActiveWorkspaces().FirstOrDefault() as Container;

      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));

      else if (containerToFocus is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((containerToFocus as Workspace).Name));
    }

    private void OnApplicationExit()
    {
      _bus.Invoke(new ShowAllWindowsCommand());
      _barService.ExitApp();
      Application.Exit();
    }
  }
}
