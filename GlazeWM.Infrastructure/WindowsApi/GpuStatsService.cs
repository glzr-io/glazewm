using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.CompilerServices;
using Vostok.Sys.Metrics.PerfCounters;

namespace GlazeWM.Infrastructure.WindowsApi
{
  /// <summary>
  /// Provides access to current GPU statistics.
  /// </summary>
  public class GpuStatsService : IDisposable
  {
    private readonly PerformanceCounterCategory GpuEngineCategory = new("GPU Engine");

    /// <summary>
    /// Stats for individual GPU performance counters indexed by GpuPerformanceCategory.
    /// </summary>
    private readonly GpuStatsForType[] StatsForTypes =
    {
      new("engtype_3D"),
      new("engtype_LegacyOverlay"),
      new("engtype_VideoDecode"),
      new("engtype_Security"),
      new("engtype_Copy"),
      new("engtype_VideoEncode"),
      new("engtype_VR")
  };

    public void Dispose()
    {
      GC.SuppressFinalize(this);
      foreach (var stat in StatsForTypes)
        stat.Dispose();
    }

    ~GpuStatsService()
    {
      Dispose();
    }

    /// <summary>
    /// Returns the current GPU utilization (average of all GPUs) for a given set of categories.
    /// </summary>
    /// <param name="categories">Categories to fetch info for.</param>
    /// <returns>Average load.</returns>
    public float GetAverageLoadPercent(GpuPerformanceCategoryFlags categories)
    {
      var instanceNames = GpuEngineCategory.GetInstanceNames();
      var numCategories = 0;
      var totalValue = 0f;

      QueryCategory(categories, GpuPerformanceCategoryFlags.Copy, GpuPerformanceCategory.Copy, instanceNames, ref numCategories, ref totalValue);
      QueryCategory(categories, GpuPerformanceCategoryFlags.VideoDecode, GpuPerformanceCategory.VideoDecode, instanceNames, ref numCategories, ref totalValue);
      QueryCategory(categories, GpuPerformanceCategoryFlags.VideoEncode, GpuPerformanceCategory.VideoEncode, instanceNames, ref numCategories, ref totalValue);
      QueryCategory(categories, GpuPerformanceCategoryFlags.LegacyOverlay, GpuPerformanceCategory.LegacyOverlay, instanceNames, ref numCategories, ref totalValue);
      QueryCategory(categories, GpuPerformanceCategoryFlags.Graphics, GpuPerformanceCategory.Graphics, instanceNames, ref numCategories, ref totalValue);
      QueryCategory(categories, GpuPerformanceCategoryFlags.Security, GpuPerformanceCategory.Security, instanceNames, ref numCategories, ref totalValue);
      QueryCategory(categories, GpuPerformanceCategoryFlags.Vr, GpuPerformanceCategory.Vr, instanceNames, ref numCategories, ref totalValue);

      if (numCategories == 0)
        numCategories = 1;

      return totalValue / numCategories;
    }

    /// <summary>
    /// Returns the current GPU utilization (average of all GPUs) for a given category.
    /// </summary>
    /// <param name="category">Category to fetch info for.</param>
    /// <param name="instanceNames">Names of all available counter instances.</param>
    /// <returns>Average load.</returns>
    public float GetAverageLoadPercent(GpuPerformanceCategory category, string[] instanceNames = null)
    {
      try
      {
        instanceNames ??= GpuEngineCategory.GetInstanceNames();
        var totalUtilization = 0f;

        var statsForType = StatsForTypes[(int)category];
        foreach (var counterForProcess in statsForType.Update(instanceNames))
        {
          var processUtilization = 0f;

          // Try/catch here isn't ideal, but this appears to throw on
          // own PID after a reload; I don't exactly know why.
          foreach (var counter in counterForProcess.Value)
          {
            try { processUtilization += (float)counter.Observe(); }
            catch { /* ignored */ }
          }

          totalUtilization += processUtilization / counterForProcess.Value.Count;
        }

        return totalUtilization;
      }
      catch (Exception)
      {
        // TODO: Some decent way of logging error here; this has no instance state.
        return -1;
      }
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private void QueryCategory(GpuPerformanceCategoryFlags requestedCategories, GpuPerformanceCategoryFlags flags, GpuPerformanceCategory category, string[] instanceNames, ref int numCategories, ref float totalValue)
    {
      if ((requestedCategories & flags) != flags)
        return;

      totalValue += GetAverageLoadPercent(category, instanceNames);
      numCategories++;
    }

    private class GpuStatsForType : IDisposable
    {
      /// <summary>
      /// Name of the counter, e.g. engtype_VR.
      /// </summary>
      private readonly string _counter;

      /// <summary>
      /// Counters for each given instance of this counter.
      /// </summary>
      private readonly Dictionary<string, IPerformanceCounter<double>> _counters = new();

      public GpuStatsForType(string counter)
      {
        _counter = counter;
      }

      /// <summary>
      /// Updates the state of this counter.
      /// </summary>
      /// <param name="instanceNames">Name of all counter instances.</param>
      public Dictionary<int, List<IPerformanceCounter<double>>> Update(string[] instanceNames)
      {
        // Problem, process can have multiple eng of 1 type; in that case, value should be averaged.
        var filteredInstances = new HashSet<string>(instanceNames.Length);
        foreach (var name in instanceNames)
        {
          if (name.Contains(_counter, StringComparison.Ordinal))
            filteredInstances.Add(name);
        }

        // Remove dead items from counters.
        foreach (var key in _counters.Keys.ToArray())
        {
          if (filteredInstances.Contains(key))
            continue;

          if (_counters.Remove(key, out var value))
            value.Dispose();
        }

        // Add items not present in dictionary.
        foreach (var name in filteredInstances)
        {
          if (!_counters.ContainsKey(name))
            _counters[name] = PerformanceCounterFactory.Default.CreateCounter(
              "GPU Engine",
              "Utilization Percentage",
              name
            );
        }

        // Copy remaining results.
        var byProcessId = new Dictionary<int, List<IPerformanceCounter<double>>>();
        const string searchString = "pid_";
        var stringLength = searchString.Length;
        foreach (var counter in _counters)
        {
          var pidIndex = counter.Key.IndexOf(searchString, StringComparison.Ordinal);
          if (pidIndex == -1)
            continue;

          var processId = ExtractNumber(counter.Key.AsSpan(pidIndex + stringLength));
          if (!byProcessId.TryGetValue(processId, out var value))
          {
            value = new List<IPerformanceCounter<double>>();
            byProcessId[processId] = value;
          }

          value.Add(counter.Value);
        }

        return byProcessId;
      }

      /// <summary>
      /// Extracts a number from a string which begins with a number.
      /// </summary>
      /// <param name="input">Input string.</param>
      /// <returns>Number starting at input.</returns>
      private static int ExtractNumber(ReadOnlySpan<char> input)
      {
        var result = 0;
        foreach (var c in input)
        {
          if (char.IsDigit(c))
            result = (result * 10) + (c - '0');
          else
            break;
        }

        return result;
      }

      public void Dispose()
      {
        foreach (var counter in _counters)
          counter.Value.Dispose();
      }
    }
  }

  // DO NOT REARRANGE ITEMS BELOW, ENUM USED AS INDEXER.

  /// <summary>
  /// Category used for fetching GPU performance data.
  /// </summary>
  [Flags]
  public enum GpuPerformanceCategoryFlags
  {
    /// <summary>
    /// 3D GPU Engine usage. i.e. engtype_3D
    /// </summary>
    Graphics = 1 << 0,

    /// <summary>
    /// Legacy API for displaying items over other items. i.e. engtype_LegacyOverlay
    /// </summary>
    LegacyOverlay = 1 << 1,

    /// <summary>
    /// Dedicated video decoding silicon. i.e. engtype_VideoDecode
    /// </summary>
    VideoDecode = 1 << 2,

    /// <summary>
    /// Workloads related to cryptography, such as encryption, decryption, and secure video processing. i.e. engtype_Security
    /// </summary>
    Security = 1 << 3,

    /// <summary>
    /// Copying data without intervention of CPU e.g. copying framebuffer across screens in multi GPU setup or uploading textures. engtype_Copy
    /// </summary>
    Copy = 1 << 4,

    /// <summary>
    /// Dedicated video encoding silicon load, i.e. NVENC/AMD AMF/QuickSync. engtype_VideoEncode
    /// </summary>
    VideoEncode = 1 << 5,

    /// <summary>
    /// Virtual Reality Related Stuff (don't know exact work, probably space reproduction stuff). engtype_VR
    /// </summary>
    Vr = 1 << 6,

    /// <summary>
    /// All supported categories.
    /// </summary>
    All = Graphics | LegacyOverlay | VideoDecode | Security | Copy | VideoEncode | Vr
  }

  /// <inheritdoc cref="GpuPerformanceCategoryFlags"/>
  [Flags]
  public enum GpuPerformanceCategory
  {
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.Graphics"/>
    Graphics,
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.LegacyOverlay"/>
    LegacyOverlay,
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.VideoDecode"/>
    VideoDecode,
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.Security"/>
    Security,
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.Copy"/>
    Copy,
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.VideoEncode"/>
    VideoEncode,
    /// <inheritdoc cref="GpuPerformanceCategoryFlags.Vr"/>
    Vr
  }
}
