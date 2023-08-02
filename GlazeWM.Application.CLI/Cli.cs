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

        var serverMessages = GetMessagesObservable(client);

        // Wait for server to respond with a message.
        var firstMessage = await serverMessages
          .FirstAsync()
          .Timeout(TimeSpan.FromSeconds(5));

        // Exit on first message received when not subscribing to an event.
        if (!isSubscribeMessage)
        {
          Console.WriteLine(firstMessage);
          client.Disconnect();
          return ExitCode.Success;
        }

        // Special handling is needed for event subscriptions.
        serverMessages.Subscribe(
          onNext: message => Console.WriteLine(message),
          onError: error => Console.Error.WriteLine(error)
        );

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

    /// <summary>
    /// Get `IObservable` of parsed server messages.
    /// </summary>
    private static IObservable<string> GetMessagesObservable(WebsocketClient client)
    {
      return client.Messages.Select(message =>
      {
        var parsedMessage = JsonDocument.Parse(firstMessage).RootElement;

        var error = parsedMessage.Get("error");
        var data = parsedMessage.Get("data");

        if (error is not null)
          throw new Exception(error);

        return data;
      });
    }
  }
}
