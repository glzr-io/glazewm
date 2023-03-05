using System.Collections.Generic;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class ExecProcessCommand : Command
  {
    public string ProcessName { get; }
    public string ProcessUserName { get; }
    public string ProcessPassword { get; }
    public List<string> Args { get; }

    public ExecProcessCommand(string processName, List<string> args, string processUserName = "", string processPassword = "")
    {
      ProcessName = processName;
      Args = args;
      ProcessUserName = processUserName;
      ProcessPassword = processPassword;
    }
  }
}
