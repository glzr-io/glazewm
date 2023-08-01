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
      client.SendText(string.Join(" ", args));

      client.Messages.TakeWhile(value => true)
        .Subscribe(message => Console.WriteLine(message));

      var _ = Console.ReadLine();
      client.DisconnectAndStop();

      // var response = _websocketClient.Send(message);
      // var response = message.MapResult(
      //   (SubscribeMessage message) => HandleSubscribe(message),
      //   _ => _websocketClient.Send(parsedArgs.Text)
      // );

      // Console.WriteLine(response.Message);
    }

    /// <summary>
    /// Special handling is needed for subscribe messages (eg. `subscribe -e
    /// window_focused`).
    /// </summary>
    // private void HandleSubscribe(SubscribeMessage message)
    // {
    //   string line = Console.ReadLine();
    // }
  }
}
