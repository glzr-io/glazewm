using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Events
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
