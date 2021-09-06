namespace LarsWM.Domain.UserConfigs
{
  public class FocusWorkspaceKeybindingCommand
  {
    public string WorkspaceName { get; }

    public FocusWorkspaceKeybindingCommand(string workspaceName)
    {
      WorkspaceName = workspaceName;
    }
  }
}
