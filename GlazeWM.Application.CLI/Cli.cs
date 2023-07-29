using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    private readonly WebsocketClient _websocketClient;

    public Cli(WebsocketClient websocketClient)
    {
      _websocketClient = websocketClient;
    }

    public void Start(string message)
    {
      _websocketClient.Send(message.Split(" "));
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
