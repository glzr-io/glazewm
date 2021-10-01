using GlazeWM.Bar;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Monitors.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using System.Linq;
using System.Windows.Forms;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bootstrapper
{
  class Startup
  {
    private Bus _bus;
    private MonitorService _monitorService;
    private KeybindingService _keybindingService;
    private WindowEventService _windowEventService;
    private WindowService _windowService;
    private BarService _barService;
    private WorkspaceService _workspaceService;

    public Startup(
      Bus bus,
      MonitorService monitorService,
      KeybindingService keybindingService,
      WindowEventService windowEventService,
      WindowService windowService,
      BarService barService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _keybindingService = keybindingService;
      _windowEventService = windowEventService;
      _windowService = windowService;
      _barService = barService;
      _workspaceService = workspaceService;
    }

    public void Init()
    {
      // Launch bar WPF application. Spawns bar window when monitors are added, so the service needs
      // to be initialized before populating initial state.
      _barService.StartApp();
      _keybindingService.Start();

      // Populate initial monitors, windows, workspaces and user config.
      PopulateInitialState();

      // Listen for window events (eg. close, focus).
      _windowEventService.Start();
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
      foreach (var windowHandle in _windowService.GetAllWindowHandles())
      {
        // Get workspace that encompasses most of the window.
        var targetMonitor = _monitorService.GetMonitorFromUnmanagedHandle(windowHandle);
        var targetWorkspace = targetMonitor.DisplayedWorkspace;

        _bus.Invoke(new AddWindowCommand(windowHandle, targetWorkspace, false));
      }

      _bus.Invoke(new RedrawContainersCommand());

      // `GetForegroundWindow` might return a handle that is not in tree. In that case, set
      // focus to an arbitrary window. If there are no manageable windows in the tree, set focus
      // to an arbitrary workspace.
      Container focusedContainer =
        _windowService.GetWindows().FirstOrDefault(window => window.Hwnd == GetForegroundWindow())
        ?? _windowService.GetWindows().FirstOrDefault() as Container
        ?? _workspaceService.GetActiveWorkspaces().FirstOrDefault() as Container;

      if (focusedContainer is Window)
        _bus.Invoke(new FocusWindowCommand(focusedContainer as Window));

      else if (focusedContainer is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((focusedContainer as Workspace).Name));
    }
  }
}
