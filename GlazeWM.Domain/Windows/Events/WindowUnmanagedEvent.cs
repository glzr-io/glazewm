using System;
using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Events
{
  public record WindowUnmanagedEvent(Guid RemovedId, IntPtr RemovedHandle)
    : Event(DomainEvent.WindowUnmanaged);
}
