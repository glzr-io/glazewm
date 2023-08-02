using System.Reactive.Linq;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    public static async Task<ExitCode> Start(
      string[] args,
      int ipcServerPort,
      bool isSubscribeMessage)
    {
      var client = new WebsocketClient(ipcServerPort);

      try
      {
        var isConnected = client.Connect();

        if (!isConnected)
          throw new Exception("Unable to connect to IPC server.");

        client.ReceiveAsync();

        var message = string.Join(" ", args);
        var sendSuccess = client.SendTextAsync(message);

        if (!sendSuccess)
          throw new Exception("Failed to send message to IPC server.");

        // Wait for server to respond with a message.
        var firstMessage = await client.Messages
          .FirstAsync()
          .Timeout(TimeSpan.FromSeconds(5));

        var parsedMessage = JsonDocument.Parse(firstMessage).RootElement;
        var error = parsedMessage.Get("error");

        if (error is not null)
          throw new Exception(error);

        // Exit on first message received for messages that aren't subscriptions.
        if (!isSubscribeMessage)
        {
          Console.WriteLine(parsedMessage.Get("data"));
          client.Disconnect();
          return ExitCode.Success;
        }

        // Special handling is needed for subscribe messages. For subscribe messages,
        // skip the acknowledgement message and ignore timeouts.
        client.Messages.Subscribe(message =>
        {
          var parsedMessage = JsonDocument.Parse(message).RootElement;
          var error = parsedMessage.Get("error");

          if (error is not null)
            Console.Error.WriteLine(error)

          Console.WriteLine(parsedMessage.Get("data"))
        });

        var _ = Console.ReadLine();

        client.Disconnect();
        return ExitCode.Success;
      }
      catch (Exception exception)
      {
        Console.Error.WriteLine(exception.Message);
        client.Disconnect();
        return ExitCode.Error;
      }
    }
  }
}
