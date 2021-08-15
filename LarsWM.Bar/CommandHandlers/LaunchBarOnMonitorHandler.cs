using LarsWM.Bar.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;
using System.Threading;
using System.Diagnostics;
using System.Windows.Forms;
using System;

namespace LarsWM.Bar.CommandHandlers
{
  class LaunchBarOnMonitorHandler : ICommandHandler<LaunchBarOnMonitorCommand>
  {
    private MonitorService _monitorService { get; }
    private WorkspaceService _workspaceService { get; }
    public Bus _bus { get; }

    public LaunchBarOnMonitorHandler(MonitorService monitorService, WorkspaceService workspaceService, Bus bus)
    {
      _monitorService = monitorService;
      _workspaceService = workspaceService;
      _bus = bus;
    }

    // TODO: Set bar width to width of monitor and launch bar on given monitor.
    public dynamic Handle(LaunchBarOnMonitorCommand command)
    {
      var thread = new Thread(() =>
      {
        var bar = new MainWindow(command.Monitor, _workspaceService, _bus);
        bar.Show();
        System.Windows.Threading.Dispatcher.Run();
      });

      thread.Name = "LarsWMBar";
      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();

      // Application app = new Application();

      // app.SetUiContext(System.Threading.SynchronizationContext.Current);
      // app.Show();
      // Application.Run();

      // Application app = new Application();
      // app.Run(new MainWindow(command.Monitor, _workspaceService, _bus));

      return CommandResponse.Ok;
    }
  }
}
