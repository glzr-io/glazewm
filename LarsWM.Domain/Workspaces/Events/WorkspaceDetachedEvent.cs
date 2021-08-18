using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Events
{
  public class WorkspaceDetachedEvent : Event
  {
    public Workspace DetachedWorkspace { get; }

    public WorkspaceDetachedEvent(Workspace detachedWorkspace)
    {
      DetachedWorkspace = detachedWorkspace;
    }
  }
}
