using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;
using System.Reactive.Linq;
using System;
using System.Threading;

namespace LarsWM.Bar
{
  public class BarManagerService
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;

    public BarManagerService(Bus bus, WorkspaceService workspaceService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
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
              var bar = new MainWindow((@event as MonitorAddedEvent).AddedMonitor, _workspaceService, _bus);
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
