using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.Common;

namespace GlazeWM.App.Cli
{
  public sealed class CliStartup
  {
    public static async Task<ExitCode> Run(
      string[] args,
      int ipcServerPort,
      bool isSubscribeMessage)
    {
      var client = new IpcClient(ipcServerPort);

      try
      {
        await client.ConnectAsync();

        // Wait for server to respond with a message.
        var clientMessage = string.Join(" ", args);
        var firstResponse = await client.SendAndWaitReplyAsync(clientMessage);

        // Exit on first message received when not subscribing to an event.
        if (!isSubscribeMessage)
        {
          Console.WriteLine(firstResponse);
          await client.DisconnectAsync();
          return ExitCode.Success;
        }

        // When subscribing to events, continuously listen for server messages.
        while (true)
        {
          var eventResponse = await client.ReceiveAsync();
          Console.WriteLine(eventResponse);
        }
      }
      catch (Exception exception)
      {
        Console.Error.WriteLine(exception.Message);
        await client.DisconnectAsync();
        return ExitCode.Error;
      }
    }
  }
}
