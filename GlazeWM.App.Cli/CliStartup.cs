using System.Text.Json;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.App.Cli
{
  public sealed class CliStartup
  {
    public static async Task<ExitCode> Run(
      string[] args,
      int ipcServerPort,
      bool isSubscribeMessage)
    {
      var client = new WebSocketClient(ipcServerPort);

      try
      {
        await client.ConnectAsync(CancellationToken.None);

        var clientMessage = string.Join(" ", args);
        await client.SendTextAsync(clientMessage, CancellationToken.None);

        // Wait for server to respond with a message.
        var serverResponse = await client.ReceiveTextAsync(CancellationToken.None);

        // Exit on first message received when not subscribing to an event.
        if (!isSubscribeMessage)
        {
          var parsedMessage = ParseServerMessage(serverResponse);
          Console.WriteLine(parsedMessage);
          await client.DisconnectAsync(CancellationToken.None);
          return ExitCode.Success;
        }

        // When subscribing to events, continuously listen for server messages.
        while (true)
        {
          var message = await client.ReceiveTextAsync(CancellationToken.None);
          var parsedMessage = ParseServerMessage(message);
          Console.WriteLine(parsedMessage);
        }
      }
      catch (Exception exception)
      {
        Console.Error.WriteLine(exception.Message);
        await client.DisconnectAsync(CancellationToken.None);
        return ExitCode.Error;
      }
    }

    /// <summary>
    /// Parse JSON in server message.
    /// </summary>
    private static string ParseServerMessage(string message)
    {
      var parsedMessage = JsonDocument.Parse(message).RootElement;
      var error = parsedMessage.GetProperty("error").GetString();

      if (error is not null)
        throw new Exception(error);

      return parsedMessage.GetProperty("data").ToString();
    }
  }
}
