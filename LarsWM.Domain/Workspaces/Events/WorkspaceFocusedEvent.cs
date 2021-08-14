using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Events
{
  public class WorkspaceFocusedEvent : Event
  {
    public Workspace FocusedWorkspace { get; }

    public WorkspaceFocusedEvent(Workspace focusedWorkspace)
    {
      FocusedWorkspace = focusedWorkspace;
    }
  }
}
