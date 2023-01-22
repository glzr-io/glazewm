using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Workspaces.Commands
{
  public class FocusWorkspaceSequenceCommand : Command
  {
    public Sequence Direction { get; }

    public FocusWorkspaceSequenceCommand(Sequence direction)
    {
      Direction = direction;
    }
  }
}
