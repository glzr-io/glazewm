using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public class WorkspaceDeactivatedEvent : Event
  {
    public Workspace DeactivatedWorkspace { get; }

    public WorkspaceDeactivatedEvent(Workspace deactivatedWorkspace)
    {
      DeactivatedWorkspace = deactivatedWorkspace;
    }
  }
}
