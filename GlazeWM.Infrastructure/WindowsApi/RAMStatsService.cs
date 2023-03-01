using System.Diagnostics;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class RAMStatsService : System.IDisposable
  {
    private readonly PerformanceCounter _ramCounter = new("Memory", "Available MBytes");
    public float GetCurrentUtilization()
    {
      float total = new Microsoft.VisualBasic.Devices.ComputerInfo().TotalPhysicalMemory;
      var used = 1024.0f * 1024.0f * _ramCounter.NextValue();
      return 100.0f * (total - used) / total;
    }

    public void Dispose()
    {
      _ramCounter.Dispose();
    }
  }
}
