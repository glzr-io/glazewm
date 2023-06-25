using System;
using Microsoft.VisualBasic.Devices;
using Vostok.Sys.Metrics.PerfCounters;

namespace GlazeWM.Infrastructure.WindowsApi
{
  /// <summary>
  /// Provides access to current CPU statistics.
  /// </summary>
  public class MemoryStatsService : IDisposable
  {
    private readonly IPerformanceCounter<double> _availableBytes = MakeCounter("Available Bytes");
    private readonly IPerformanceCounter<double> _cacheBytes = MakeCounter("Cache Bytes");
    private readonly IPerformanceCounter<double> _commitSize = MakeCounter("Committed Bytes");
    private readonly IPerformanceCounter<double> _commitLimit = MakeCounter("Commit Limit");
    private readonly IPerformanceCounter<double> _pagedResidentBytes = MakeCounter("Pool Paged Resident Bytes");
    private readonly long _physicalBytes = (long)new ComputerInfo().TotalPhysicalMemory;

    /// <inheritdoc />
    ~MemoryStatsService() => Dispose();

    /// <inheritdoc />
    public void Dispose()
    {
      GC.SuppressFinalize(this);
      _availableBytes.Dispose();
      _cacheBytes.Dispose();
      _commitSize.Dispose();
      _commitLimit.Dispose();
      _pagedResidentBytes.Dispose();
    }

    /// <summary>
    /// Returns the current CPU utilization as a percentage.
    /// </summary>
    /// <exception cref="ArgumentOutOfRangeException">Invalid measurement.</exception>
    public MemoryMeasurement GetMeasurement(RamMeasurement measurement)
    {
      return measurement switch
      {
        RamMeasurement.PhysicalMemory => new MemoryMeasurement((float)(_physicalBytes - _availableBytes.Observe()), _physicalBytes),
        RamMeasurement.CacheBytes => new MemoryMeasurement((float)_cacheBytes.Observe(), _physicalBytes),
        RamMeasurement.CommitSize => new MemoryMeasurement((float)_commitSize.Observe(), (float)_commitLimit.Observe()),
        RamMeasurement.PagedResidentBytes => new MemoryMeasurement((float)_pagedResidentBytes.Observe(), _physicalBytes),
        _ => throw new ArgumentOutOfRangeException(nameof(measurement), measurement, null)
      };
    }

    private static IPerformanceCounter<double> MakeCounter(string counter)
    {
      return PerformanceCounterFactory.Default.CreateCounter("Memory", counter);
    }
  }

  /// <summary>
  /// Individual memory measurement.
  /// </summary>
  public struct MemoryMeasurement
  {
    public float CurrentValue { get; set; }
    public float MaxValue { get; set; }

    public MemoryMeasurement(float currentValue, float maxValue)
    {
      CurrentValue = currentValue;
      MaxValue = maxValue;
    }

    /// <summary>
    /// Divides the items in the measurement by a specific value.
    /// </summary>
    /// <param name="divideBy">Number to divide by.</param>
    public void DivideBy(float divideBy)
    {
      CurrentValue /= divideBy;
      MaxValue /= divideBy;
    }
  }

  /// <summary>
  /// The value to obtain measurements for.
  /// </summary>
  public enum RamMeasurement
  {
    /// <summary>
    /// Current amount of physical RAM in use; i.e. working set.
    /// </summary>
    PhysicalMemory,

    /// <summary>
    /// Amount of cached file data in physical RAM.
    /// </summary>
    CacheBytes,

    /// <summary>
    /// Retrieves the amount of committed virtual memory (bytes); i.e. which has space reserved on the disk paging file(s).
    /// </summary>
    CommitSize,

    /// <summary>
    /// Size of the active portion of the paged pool in physical memory, storing objects that can be written to disk when they're not in use.
    /// </summary>
    PagedResidentBytes
  }
}

/*
// All Counters with Documentation, in case you ever want to add additional options to RamMeasurement
// Documentation is official sourced from `Performance Monitor -> Add Counter (Ctrl+A) + Check 'Show Description'`.

/// <summary>
/// Available Bytes is the amount of physical memory, immediately available for allocation to a process or for system use.
/// </summary>
AvailableBytes,

/// <summary>
/// Cache Bytes the size, in bytes, of the portion of the system file cache which is currently resident and active in physical memory.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
CacheBytes,

/// <summary>
/// Cache Bytes Peak is the maximum number of bytes used by the system file cache since the system was last restarted.
/// This might be larger than the current size of the cache. This counter displays the last observed value only; it is not an average.
/// </summary>
CacheBytesPeak,

/// <summary>
/// Cache Faults/sec is the rate at which faults occur when a page sought in the file system cache is not found and must be retrieved
/// from elsewhere in memory (a soft fault) or from disk (a hard fault). The file system cache is an area of physical memory that stores
/// recently used pages of data for applications. Cache activity is a reliable indicator of most application I/O operations.
/// This counter shows the number of faults, without regard for the number of pages faulted in each operation.
/// </summary>
CacheFaultsPerSec,

/// <summary>
/// Commit Limit is the amount of virtual memory that can be committed without having to extend the paging file(s).
/// It is measured in bytes. Committed memory is the physical memory which has space reserved on the disk paging files.
/// There can be one paging file on each logical drive). If the paging file(s) are be expanded, this limit increases accordingly.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
CommitLimitBytes,

/// <summary>
/// Committed Bytes is the amount of committed virtual memory, in bytes.
/// Committed memory is the physical memory which has space reserved on the disk paging file(s).
/// There can be one or more paging files on each physical drive.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
CommitBytes,

/// <summary>
/// Free & Zero Page List Bytes is the amount of physical memory, in bytes, that is assigned to the free and zero page lists.
/// This memory does not contain cached data. It is immediately available for allocation to a process or for system use.
/// </summary>
FreeZeroPageListBytes,

/// <summary>
/// Free System Page Table Entries is the number of page table entries not currently in used by the system.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
FreePageTableEntries,

/// <summary>
/// Modified Page List Bytes is the amount of physical memory, in bytes, that is assigned to the modified page list.
/// This memory contains cached data and code that is not actively in use by processes, the system and the system cache.
/// This memory needs to be written out before it will be available for allocation to a process or for system use.
/// </summary>
ModifiedPageListBytes,

/// <summary>
/// Page Faults/sec is the average number of pages faulted per second.
/// It is measured in number of pages faulted per second because only one page is faulted in each fault operation, hence
/// this is also equal to the number of page fault operations. This counter includes both hard faults (those that require disk access)
/// and soft faults (where the faulted page is found elsewhere in physical memory.) Most processors can handle large numbers of soft
/// faults without significant consequence. However, hard faults, which require disk access, can cause significant delays.
/// </summary>
PageFaultsPerSec,

/// <summary>
/// Page Reads/sec is the rate at which the disk was read to resolve hard page faults.
/// It shows the number of reads operations, without regard to the number of pages retrieved in each operation.
/// Hard page faults occur when a process references a page in virtual memory that is not in working set or elsewhere
/// in physical memory, and must be retrieved from disk. This counter is a primary indicator of the kinds of faults that
/// cause system-wide delays. It includes read operations to satisfy faults in the file system cache (usually requested by applications)
/// and in non-cached mapped memory files. Compare the value of Memory\\Pages Reads/sec to the value of Memory\\Pages Input/sec
/// to determine the average number of pages read during each operation.
/// </summary>
PageReadsPerSec,

/// <summary>
/// Page Writes/sec is the rate at which pages are written to disk to free up space in physical memory.
/// Pages are written to disk only if they are changed while in physical memory, so they are likely to hold data, not code.
/// This counter shows write operations, without regard to the number of pages written in each operation.
/// This counter displays the difference between the values observed in the last two samples, divided by the duration
/// of the sample interval.
/// </summary>
PageWritesPerSec,

/// <summary>
/// Pages/sec is the rate at which pages are read from or written to disk to resolve hard page faults.
/// This counter is a primary indicator of the kinds of faults that cause system-wide delays.
/// It is the sum of Memory\\Pages Input/sec and Memory\\Pages Output/sec.
/// It is counted in numbers of pages, so it can be compared to other counts of pages, such as Memory\\Page Faults/sec,
/// without conversion. It includes pages retrieved to satisfy faults in the file system cache (usually requested by
/// applications) non-cached mapped memory files.
/// </summary>
PagesPerSec,

/// <summary>
/// Pool Nonpaged Allocs is the number of calls to allocate space in the nonpaged pool.
/// The nonpaged pool is an area of system memory area for objects that cannot be written to disk,
/// and must remain in physical memory as long as they are allocated.  It is measured in numbers of calls to
/// allocate space, regardless of the amount of space allocated in each call.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
PoolNonpagedAllocs,

/// <summary>
/// Pool Nonpaged Bytes is the size, in bytes, of the nonpaged pool, an area of the system virtual memory that is used
/// for objects that cannot be written to disk, but must remain in physical memory as long as they are allocated.
/// Memory\\Pool Nonpaged Bytes is calculated differently than Process\\Pool Nonpaged Bytes, so it might not equal
/// Process(_Total)\\Pool Nonpaged Bytes.  This counter displays the last observed value only; it is not an average.
/// </summary>
PoolNonpagedBytes,

/// <summary>
/// Pool Paged Allocs is the number of calls to allocate space in the paged pool. The paged pool is an area of the
/// system virtual memory that is used for objects that can be written to disk when they are not being used.
/// It is measured in numbers of calls to allocate space, regardless of the amount of space allocated in each call.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
PoolPagedAllocs,

/// <summary>
/// Pool Paged Bytes is the size, in bytes, of the paged pool, an area of the system virtual memory that is used for
/// objects that can be written to disk when they are not being used.  Memory\\Pool Paged Bytes is calculated differently
/// than Process\\Pool Paged Bytes, so it might not equal Process(_Total)\\Pool Paged Bytes. This counter displays the
/// last observed value only; it is not an average.
/// </summary>
PoolPagedBytes,

/// <summary>
/// Pool Paged Resident Bytes is the size, in bytes, of the portion of the paged pool that is currently resident and active
/// in physical memory. The paged pool is an area of the system virtual memory that is used for objects that can be written
/// to disk when they are not being used. This counter displays the last observed value only; it is not an average.
/// </summary>
PoolPagedResidentBytes,

/// <summary>
/// Standby Cache Core Bytes is the amount of physical memory, in bytes, that is assigned to the core standby cache page lists.
/// This memory contains cached data and code that is not actively in use by processes, the system and the system cache.
/// It is immediately available for allocation to a process or for system use.
/// If the system runs out of available free and zero memory, memory on lower priority standby cache page lists will be
/// repurposed before memory on higher priority standby cache page lists.
/// </summary>
StandbyCacheCoreBytes,

/// <summary>
/// Standby Cache Normal Priority Bytes is the amount of physical memory, in bytes, that is assigned to the normal
/// priority standby cache page lists. This memory contains cached data and code that is not actively in use by processes,
/// the system and the system cache. It is immediately available for allocation to a process or for system use.
/// If the system runs out of available free and zero memory, memory on lower priority standby cache page lists will be
/// repurposed before memory on higher priority standby cache page lists.
/// </summary>
StandbyCacheNormalPriorityBytes,

/// <summary>
/// Standby Cache Reserve Bytes is the amount of physical memory, in bytes, that is assigned to the reserve standby
/// cache page lists. This memory contains cached data and code that is not actively in use by processes, the system
/// and the system cache. It is immediately available for allocation to a process or for system use. If the system
/// runs out of available free and zero memory, memory on lower priority standby cache page lists will be repurposed
/// before memory on higher priority standby cache page lists.
/// </summary>
StandbyCacheNormalReserveBytes,

/// <summary>
/// System Code Resident Bytes is the size, in bytes, of the pageable operating system code that is currently resident
/// and active in physical memory. This value is a component of Memory\\System Code Total Bytes. Memory\\System Code
/// Resident Bytes (and Memory\\System Code Total Bytes) does not include code that must remain in physical memory
/// and cannot be written to disk. This counter displays the last observed value only; it is not an average.
/// </summary>
SystemCodeResidentBytes,

/// <summary>
/// System Code Total Bytes is the size, in bytes, of the pageable operating system code currently mapped into the system
/// virtual address space. This value is calculated by summing the bytes in Ntoskrnl.exe, Hal.dll, the boot drivers,
/// and file systems loaded by Ntldr/osloader.  This counter does not include code that must remain in physical memory
/// and cannot be written to disk. This counter displays the last observed value only; it is not an average.
/// </summary>
SystemCodeTotalBytes,

/// <summary>
/// System Driver Resident Bytes is the size, in bytes, of the pageable physical memory being used by device drivers.
/// It is the working set (physical memory area) of the drivers. This value is a component of Memory\\System Driver Total Bytes,
/// which also includes driver memory that has been written to disk. Neither Memory\\System Driver Resident Bytes nor
/// Memory\\System Driver Total Bytes includes memory that cannot be written to disk.
/// </summary>
SystemDriverResidentBytes,

/// <summary>
/// System Driver Total Bytes is the size, in bytes, of the pageable virtual memory currently being used by device drivers.
/// Pageable memory can be written to disk when it is not being used. It includes both physical memory
/// (Memory\\System Driver Resident Bytes) and code and data paged to disk. It is a component of Memory\\System Code Total Bytes.
/// This counter displays the last observed value only; it is not an average.
/// </summary>
SystemDriverTotalBytes,

/// <summary>
/// Transition Faults/sec is the rate at which page faults are resolved by recovering pages that were being used by
/// another process sharing the page, or were on the modified page list or the standby list, or were being written to
/// disk at the time of the page fault. The pages were recovered without additional disk activity. Transition faults
/// are counted in numbers of faults; because only one page is faulted in each operation, it is also equal to the
/// number of pages faulted.
/// </summary>
TransitionFaultsPerSec,

/// <summary>
/// Transition Pages RePurposed is the rate at which the number of transition cache pages were reused for a different purpose.
/// These pages would have otherwise remained in the page cache to provide a (fast) soft fault (instead of retrieving
/// it from backing store) in the event the page was accessed in the future.  Note these pages can contain private or
/// sharable memory.
/// </summary>
TransitionPagesRepurposedPerSec,

/// <summary>
/// Write Copies/sec is the rate at which page faults are caused by attempts to write that have been satisfied by copying
/// of the page from elsewhere in physical memory. This is an economical way of sharing data since pages are only copied
/// when they are written to; otherwise, the page is shared. This counter shows the number of copies, without regard
/// for the number of pages copied in each operation.
/// </summary>
WriteCopiesPerSec
*/
