using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Events
{
  public class WindowUnmanagedEvent : Event
  {
    public Guid RemovedId { get; }
    public IntPtr RemovedHandle { get; }

    public WindowUnmanagedEvent(Guid removedId, IntPtr removedHandle)
    {
      RemovedId = removedId;
      RemovedHandle = removedHandle;
    }
  }
}
