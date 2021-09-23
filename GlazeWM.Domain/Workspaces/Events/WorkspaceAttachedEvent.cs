using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Events
{
  public class WorkspaceAttachedEvent : Event
  {
    public Workspace AttachedWorkspace { get; }

    public WorkspaceAttachedEvent(Workspace attachedWorkspace)
    {
      AttachedWorkspace = attachedWorkspace;
    }
  }
}
