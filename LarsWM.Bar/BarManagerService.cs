using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;
using System.Reactive.Linq;
using System;
using System.Threading;
using LarsWM.Domain.UserConfigs;

namespace LarsWM.Bar
{
  public class BarManagerService
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;
    private UserConfigService _userConfigService;

    public BarManagerService(Bus bus, WorkspaceService workspaceService, UserConfigService userConfigService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _userConfigService = userConfigService;
    }

    public void Init()
    {
      var thread = new Thread(() =>
      {
        var application = new App();

        // Launch the bar window on the added monitor.
        _bus.Events.Where(@event => @event is MonitorAddedEvent)
          .Subscribe((@event) =>
          {
            application.Dispatcher.Invoke(() =>
            {
              var bar = new MainWindow((@event as MonitorAddedEvent).AddedMonitor, _workspaceService, _bus, _userConfigService);
              bar.Show();
            });
          });

        application.Run();
      });

      thread.Name = "LarsWMBar";
      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();
    }
  }
}
