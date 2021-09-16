using LarsWM.Bar;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi;
using System.Linq;
using System.Windows.Forms;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Bootstrapper
{
  class Startup
  {
    private Bus _bus;
    private MonitorService _monitorService;
    private KeybindingService _keybindingService;
    private WindowEventService _windowEventService;
    private WindowService _windowService;
    private BarManagerService _barManagerService;
    private WorkspaceService _workspaceService;

    public Startup(
      Bus bus,
      MonitorService monitorService,
      KeybindingService keybindingService,
      WindowEventService windowEventService,
      WindowService windowService,
      BarManagerService barManagerService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _monitorService = monitorService;
      _keybindingService = keybindingService;
      _windowEventService = windowEventService;
      _windowService = windowService;
      _barManagerService = barManagerService;
      _workspaceService = workspaceService;
    }

    public void Init()
    {
      // Launch bar WPF application. Spawns bar window when monitors are added, so the service needs
      // to be initialized before populating initial state.
      // TODO: Rename `Init` method to `Start`.
      _barManagerService.Init();
      _keybindingService.Start();

      // Populate initial monitors, windows, workspaces and user config.
      PopulateInitialState();

      _windowEventService.Start();
    }

    /// <summary>
    /// Populate initial monitors, windows, workspaces and user config.
    /// </summary>
    private void PopulateInitialState()
    {
      // Read user config file and set its values in state.
      _bus.Invoke(new EvaluateUserConfigCommand());

      // Create a Monitor and consequently a Workspace for each detected Screen.
      foreach (var screen in Screen.AllScreens)
        _bus.Invoke(new AddMonitorCommand(screen));

      // Add initial windows to tree.
      _bus.Invoke(new AddInitialWindowsCommand());

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
