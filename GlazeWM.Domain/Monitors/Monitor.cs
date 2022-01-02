using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain.Monitors
{
  public class Monitor : Container
  {
    public string DeviceName { get; set; }
    public override int Width { get; set; }
    public override int Height { get; set; }
    public override int X { get; set; }
    public override int Y { get; set; }
    public bool IsPrimary { get; set; }
    public Workspace DisplayedWorkspace { get; set; }

    private MonitorService _monitorService =
      ServiceLocator.Provider.GetRequiredService<MonitorService>();
    public uint Dpi => _monitorService.GetMonitorDpi(this);
    public decimal ScaleFactor => decimal.Divide(Dpi, 96);

    public Monitor(
      string deviceName,
      int width,
      int height,
      int x,
      int y,
      bool isPrimary
    )
    {
      DeviceName = deviceName;
      Width = width;
      Height = height;
      X = x;
      Y = y;
      IsPrimary = isPrimary;
    }
  }
}
