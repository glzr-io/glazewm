using System.Reactive.Linq;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    public static void Start(string[] args, int ipcServerPort)
    {
      var client = new WebsocketClient(ipcServerPort);
      client.Connect();
      client.ReceiveAsync();
      client.SendBinaryAsync(string.Join(" ", args));

      // Special handling is needed for subscribe messages (eg. `subscribe -e
      // window_focused`).
      var isContinuousOutput = true;

      client.Messages.TakeWhile(value => isContinuousOutput)
        .Subscribe(message => Console.WriteLine(message));

      var _ = Console.ReadLine();
      client.Disconnect();
    }
  }
}
