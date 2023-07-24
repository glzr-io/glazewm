namespace GlazeWM.Application.CLI
{
  public sealed class Cli
  {
    private readonly IpcClient _ipcClient;

    public Cli(IpcClient ipcClient)
    {
      _ipcClient = ipcClient;
    }

    public void Start(ParsedArgs<object> parsedArgs)
    {
      var response = parsedArgs.MapResult(
        (SubscribeMessage message) => HandleSubscribe(message),
        _ => _ipcClient.Send(parsedArgs.Text)
      );

      Console.WriteLine(response.Message);

      return 1;
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
