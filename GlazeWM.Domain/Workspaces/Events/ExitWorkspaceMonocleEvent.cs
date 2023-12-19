using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public record ExitWorkspaceMonocleEvent(Workspace ActivatedWorkspace)
    : Event(DomainEvent.WorkspaceMonocleExited);
}
