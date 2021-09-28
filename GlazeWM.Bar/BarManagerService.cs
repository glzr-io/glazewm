using GlazeWM.Domain.Monitors.Events;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using System.Reactive.Linq;
using System;
using System.Threading;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar
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
              var barViewModel = new BarViewModel()
              {
                Monitor = (@event as MonitorAddedEvent).AddedMonitor,
                Dispatcher = application.Dispatcher,
              };
              barViewModel.InitializeState();

              var barWindow = new MainWindow(barViewModel);
              barWindow.Show();
            });
          });

        application.Run();
      });

      thread.Name = "GlazeWMBar";
      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();
    }
  }
}
