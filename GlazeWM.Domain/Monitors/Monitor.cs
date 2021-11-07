using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;
using System.Windows.Forms;

namespace GlazeWM.Domain.Monitors
{
  public class Monitor : Container
  {
    public string Name => Screen.DeviceName;
    public override int Width => Screen.WorkingArea.Width;
    public override int Height => Screen.WorkingArea.Height;
    public override int X => Screen.WorkingArea.X;
    public override int Y => Screen.WorkingArea.Y;
    public bool IsPrimary => Screen.Primary;
    public Workspace DisplayedWorkspace;

    public Screen Screen { get; }

    private MonitorService _monitorService = ServiceLocator.Provider.GetRequiredService<MonitorService>();

    public uint Dpi => _monitorService.GetMonitorDpi(Screen);

    public decimal ScaleFactor => decimal.Divide(Dpi, 96);

    public Monitor(Screen screen)
    {
      Screen = screen;
    }
  }
}
