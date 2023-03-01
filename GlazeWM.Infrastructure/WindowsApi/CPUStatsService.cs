using System.Diagnostics;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class CPUStatsService : System.IDisposable
  {
    private readonly PerformanceCounter _cpuCounter = new("Processor Information", "% Processor Utility", "_Total");
    public float GetCurrentUtilization()
    {
      return _cpuCounter.NextValue();
    }

    public void Dispose()
    {
      _cpuCounter.Dispose();
    }
  }
}
