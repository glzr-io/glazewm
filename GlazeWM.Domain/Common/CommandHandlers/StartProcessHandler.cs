using System.Diagnostics;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class StartProcessHandler : ICommandHandler<StartProcessCommand>
  {
    public CommandResponse Handle(StartProcessCommand command)
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
