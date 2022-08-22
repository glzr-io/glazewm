using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class StartProcessCommand : Command
  {
    public string ProcessName { get; init; }
    public string[] Args { get; init; }

    public StartProcessCommand(string processName, string[] args)
    {
      ProcessName = processName;
      Args = args;
    }
  }
}
