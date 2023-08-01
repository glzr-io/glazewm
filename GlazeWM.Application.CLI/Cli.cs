using System.Reactive.Linq;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    public static ExitCode Start(string[] args, int ipcServerPort)
    {
      try
      {
        var client = new WebsocketClient(ipcServerPort);
        var isConnected = client.Connect();

        if (!isConnected)
          throw new Exception("Unable to connect to IPC server.");

        client.ReceiveAsync();

        var message = string.Join(" ", args);
        var sendSuccess = client.SendTextAsync(message);

        if (!sendSuccess)
          throw new Exception("Failed to send message to IPC server.");

        // Special handling is needed for subscribe messages (eg. `subscribe -e
        // window_focused`).
        var isContinuousOutput = true;

        client.Messages.TakeWhile(value => isContinuousOutput)
          .Subscribe(message => Console.WriteLine(message));

        var _ = Console.ReadLine();
        client.Disconnect();

        return ExitCode.Success;
      }
      catch (Exception exception)
      {
        Console.Error.WriteLine(exception.Message);
        return ExitCode.Error;
      }
    }
  }
}
