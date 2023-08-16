using System.Reactive.Linq;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.App.Watcher
{
  public sealed class WatcherStartup
  {
    /// <summary>
    /// Window handles currently managed by the window manager.
    /// </summary>
    private List<IntPtr> _managedHandles = new();

    public ExitCode Run(int ipcServerPort)
    {
      var client = new WebsocketClient(ipcServerPort);

      try
      {
        var isConnected = client.Connect();

        if (!isConnected)
          throw new Exception("Unable to connect to IPC server.");

        client.ReceiveAsync();
        client.SendAsync("windows ls");

        var serverMessages = GetMessagesObservable(client);

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
    /// Restore all managed window handles.
    /// </summary>
    private void RestoreManagedHandles()
    {
      foreach (var windowHandle in _managedHandles)
        ShowWindow(windowHandle, ShowWindowFlags.ShowNoActivate);
    }
  }
}
