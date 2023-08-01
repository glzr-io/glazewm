using System.Reactive.Linq;
using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    public static void Start(string[] args, int ipcServerPort)
    {
      var client = new WebsocketClient(ipcServerPort);
      client.ConnectAsync();
      Console.WriteLine("fjdoisajfdioas");

      client.Connected.Select((_) =>
      {
        Console.WriteLine("Done!");
        // Thread.Sleep(1000);
        client.SendBinaryAsync(string.Join(" ", args));
        // client.SendTextAsync(string.Join(" ", args));
        Console.WriteLine("Done!");

        // client.Messages.TakeWhile(value => true)
        return client.Messages;
        // .Subscribe(message => Console.WriteLine(message));
        // .Subscribe(message =>
        // {
        //   Console.WriteLine(message);
        // });
      }).Switch()
        .Subscribe(message => Console.WriteLine(message));
      // var x = client.ConnectAsync();
      // // client.ReceiveAsync();
      // // client.ReceiveAsync();
      // if (x)
      // {
      var _ = Console.ReadLine();
      // client.DisconnectAndStop();
      client.Disconnect();

      // var response = _websocketClient.Send(message);
      // var response = message.MapResult(
      //   (SubscribeMessage message) => HandleSubscribe(message),
      //   _ => _websocketClient.Send(parsedArgs.Text)
      // );

      // Console.WriteLine(response.Message);

      // }
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
