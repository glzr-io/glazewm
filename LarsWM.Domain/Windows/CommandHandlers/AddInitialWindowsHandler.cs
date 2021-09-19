using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using System;
using System.Diagnostics;
using System.Linq;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class AddInitialWindowsHandler : ICommandHandler<AddInitialWindowsCommand>
  {
    private Bus _bus;
    private UserConfigService _userConfigService;
    private MonitorService _monitorService;
    private WindowService _windowService;

    public AddInitialWindowsHandler(
        Bus bus,
        UserConfigService userConfigService,
        MonitorService monitorService,
        WindowService windowService)
    {
      _bus = bus;
      _userConfigService = userConfigService;
      _monitorService = monitorService;
      _windowService = windowService;
    }

    public CommandResponse Handle(AddInitialWindowsCommand command)
    {
      var manageableWindows = _windowService.GetAllWindowHandles()
        .Select(handle => new Window(handle))
        .Where(window => window.IsManageable);

      foreach (var window in manageableWindows)
      {
        // Get workspace that encompasses most of the window.
        var targetMonitor = _monitorService.GetMonitorFromUnmanagedHandle(window.Hwnd);
        var targetWorkspace = targetMonitor.DisplayedWorkspace;

        _bus.Invoke(new AttachContainerCommand(targetMonitor.DisplayedWorkspace, window));
      }

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
