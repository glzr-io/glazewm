using System.Collections.Generic;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class ExecProcessCommand : Command
  {
    public string ProcessName { get; }
    public List<string> Args { get; }

    public ExecProcessCommand(string processName, List<string> args)
    {
      ProcessName = processName;
      Args = args;
    }
  }
}
