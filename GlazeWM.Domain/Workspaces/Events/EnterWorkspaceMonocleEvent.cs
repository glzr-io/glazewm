using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public record EnterWorkspaceMonocleEvent(Workspace ActivatedWorkspace)
    : Event(DomainEvent.WorkspaceMonocleEntered);
}
