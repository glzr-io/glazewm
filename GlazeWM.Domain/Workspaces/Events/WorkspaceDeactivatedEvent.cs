using System;
using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public record WorkspaceDeactivatedEvent(Guid DeactivatedId, string DeactivatedName)
    : Event(DomainEvent.WorkspaceDeactivated);
}
