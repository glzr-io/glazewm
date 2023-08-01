using System.Reactive.Linq;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    public static ExitCode Start(
      string[] args,
      int ipcServerPort,
      bool isSubscribeMessage)
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

        // Special handling is needed for subscribe messages. For subscribe messages,
        // skip the acknowledgement message and ignore timeouts. For all other messages,
        // exit on first message received.
        if (isSubscribeMessage)
          client.Messages.Skip(1)
            .Subscribe(message => Console.WriteLine(message));
        else
          client.Messages.Take(1)
            .Timeout(TimeSpan.FromSeconds(5))
            .Subscribe(
              onNext: message => Console.WriteLine(message),
              onError: _ => Console.Error.WriteLine("IPC message timed out.")
            );

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
