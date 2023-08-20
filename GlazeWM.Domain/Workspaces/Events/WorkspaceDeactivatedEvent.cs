using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public record WorkspaceDeactivatedEvent(Workspace DeactivatedWorkspace)
    : Event(DomainEvent.WorkspaceDeactivated);
}
