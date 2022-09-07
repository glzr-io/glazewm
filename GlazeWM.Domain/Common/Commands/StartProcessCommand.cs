using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class ExecProcessCommand : Command
  {
    public string ProcessName { get; init; }
    public string[] Args { get; init; }

    public ExecProcessCommand(string processName, string[] args)
    {
      ProcessName = processName;
      Args = args;
    }
  }
}
