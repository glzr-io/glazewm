using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class GPUStatsService
  {
    public float GetCurrentUtilization()
    {
      var gpuCounters = GetGPUCounters();
      gpuCounters.ForEach(x => x.NextValue());
      return (float)gpuCounters.Sum(x => x?.NextValue());
    }

    public static List<PerformanceCounter> GetGPUCounters()
    {
      var category = new PerformanceCounterCategory("GPU Engine");
      var counterNames = category.GetInstanceNames();

      return counterNames
        .Where(counterName => counterName.EndsWith("engtype_3D"))
        .SelectMany(counterName => category.GetCounters(counterName))
        .Where(counter => counter.CounterName.Equals("Utilization Percentage"))
        .ToList();
    }
  }
}
