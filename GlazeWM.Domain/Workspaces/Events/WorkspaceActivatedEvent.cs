using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public class WorkspaceActivatedEvent : Event
  {
    public Workspace ActivatedWorkspace { get; }

    public WorkspaceActivatedEvent(Workspace activatedWorkspace)
    {
      ActivatedWorkspace = activatedWorkspace;
    }
  }
}
