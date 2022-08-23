using System.Diagnostics;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class StartProcessHandler : ICommandHandler<StartProcessCommand>
  {
    public CommandResponse Handle(StartProcessCommand command)
    {
      using (var cmdProc = new Process())
      {
        cmdProc.StartInfo = new ProcessStartInfo
        {
          FileName = "cmd.exe",
          Arguments = $"/C start {command.ProcessName} {string.Join(" ", command.Args)}",
          CreateNoWindow = true,
        };
        cmdProc.Start();
      };
      return CommandResponse.Ok;
    }
  }
}
