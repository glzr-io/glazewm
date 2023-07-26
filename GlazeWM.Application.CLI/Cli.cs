using GlazeWM.Infrastructure.Utils;

namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    private readonly NamedPipeClient _namedPipeClient;

    public Cli(NamedPipeClient namedPipeClient)
    {
      _namedPipeClient = namedPipeClient;
    }

    public void Start(string message)
    {
      var response = message.MapResult(
        (SubscribeMessage message) => HandleSubscribe(message),
        _ => _namedPipeClient.Send(parsedArgs.Text)
      );

      Console.WriteLine(response.Message);
    }

    /// <summary>
    /// Special handling is needed for subscribe messages (eg. `subscribe -e
    /// window_focused`).
    /// </summary>
    private void HandleSubscribe(SubscribeMessage message)
    {
      string line = Console.ReadLine();
    }
  }
}
