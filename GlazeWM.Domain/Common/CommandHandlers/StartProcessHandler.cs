using System;
using System.Diagnostics;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class StartProcessHandler : ICommandHandler<StartProcessCommand>
  {
    public CommandResponse Handle(StartProcessCommand command)
    {
      /*
       * If Visual Studio Code is in PATH, a batch file with the name `code.cmd` is invoked instead
       * of the regular VS Code binary.
       * 
       * When `code.cmd` is invoked with `start`, it leaves behind a ghost `cmd.exe` window running
       * which does not disappear unless manually shut down.
       * 
       * So if the user is trying to launch VS Code, it is best to directly invoke the batch file
       * instead of using `start`. Doing this still results in a conhost process being
       * spawned but the process dies when VS Code exits.
       * 
       * See the discussion at https://github.com/lars-berger/GlazeWM/pull/108/ for more details.
       */
      var start = command.ProcessName.ToLower() == "code" ? string.Empty : "start";

      using (var cmdProc = new Process())
      {
        cmdProc.StartInfo = new ProcessStartInfo
        {
          FileName = "cmd.exe",
          Arguments = $"/C {start} {command.ProcessName} {string.Join(" ", command.Args)}",
          CreateNoWindow = true
        };
        cmdProc.Start();
      };
      return CommandResponse.Ok;
    }
  }
}
