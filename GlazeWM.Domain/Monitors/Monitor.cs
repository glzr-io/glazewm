using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Workspaces;

namespace GlazeWM.Domain.Monitors
{
  public sealed class Monitor : Container
  {
    public override string Id { get; init; }
    public string DeviceName { get; set; }
    public override int Width { get; set; }
    public override int Height { get; set; }
    public override int X { get; set; }
    public override int Y { get; set; }

    public Workspace DisplayedWorkspace => LastFocusedChild as Workspace;
    public uint Dpi => MonitorService.GetMonitorDpi(this);
    public double ScaleFactor => Dpi / 96.0;

    public Monitor(
      string deviceName,
      int width,
      int height,
      int x,
      int y)
    {
      Id = $"MONITOR/{deviceName}";
      DeviceName = deviceName;
      Width = width;
      Height = height;
      X = x;
      Y = y;
    }
  }
}
