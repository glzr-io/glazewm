using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public record WorkspaceActivatedEvent(Workspace ActivatedWorkspace)
    : Event(DomainEvent.WorkspaceActivated);
}
