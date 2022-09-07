using System.Diagnostics;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class ExecProcessHandler : ICommandHandler<ExecProcessCommand>
  {
    public CommandResponse Handle(ExecProcessCommand command)
    {
      using var process = new Process();
      process.StartInfo = new ProcessStartInfo
      {
        FileName = command.ProcessName,
        Arguments = string.Join(" ", command.Args),
        UseShellExecute = true
      };
      process.Start();

      return CommandResponse.Ok;
    }
  }
}
