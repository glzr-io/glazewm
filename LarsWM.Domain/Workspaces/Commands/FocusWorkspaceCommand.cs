using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.Commands
{
    public class FocusWorkspaceCommand : Command
    {
        public Workspace Workspace { get; }

        public FocusWorkspaceCommand(Workspace workspace)
        {
            Workspace = workspace;
        }
    }
}
