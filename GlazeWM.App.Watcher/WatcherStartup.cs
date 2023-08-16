using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Utils;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.App.Watcher
{
  public sealed class WatcherStartup
  {
    /// <summary>
    /// Window handles currently managed by the window manager.
    /// </summary>
    private readonly List<IntPtr> _managedHandles = new();

    public async Task<ExitCode> Run(int ipcServerPort)
    {
      var client = new WebSocketClient(ipcServerPort);

      try
      {
        await client.ConnectAsync(CancellationToken.None);

        await foreach (var handle in GetWindowHandles(client))
          _managedHandles.Add(handle);

        return ExitCode.Success;
      }
      catch (Exception)
      {
        RestoreManagedHandles();
        await client.DisconnectAsync(CancellationToken.None);
        return ExitCode.Success;
      }
    }

    /// <summary>
    /// Query for initial windows via IPC server.
    /// </summary>
    private async IAsyncEnumerable<IntPtr> GetWindowHandles(WebSocketClient client)
    {
      await client.SendTextAsync("windows", CancellationToken.None);
      var windowsResponse = await client.ReceiveTextAsync(CancellationToken.None);

      var initialHandles = ParseServerMessage(windowsResponse)
        .EnumerateArray()
        .Select(value => new IntPtr(value.GetInt32()));

      foreach (var handle in initialHandles)
        yield return handle;

      await client.SendTextAsync(
        "subscribe -e window_managed,window_unmanaged",
        CancellationToken.None
      );

      while (true)
      {
        var response = await client.ReceiveTextAsync(CancellationToken.None);
        var parsedResponse = ParseServerMessage(windowsResponse);
        var eventType = parsedResponse.GetProperty("type").GetString();

        switch (eventType)
        {
          case "window_managed":
            var newHandle = parsedResponse.GetProperty("handle").GetInt32();
            _managedHandles.Add(newHandle);
            break;
          case "window_unmanaged":
            var removedHandle = parsedResponse.GetProperty("removedHandle").GetInt32();
            _managedHandles.Remove(removedHandle);
            break;
        }
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

    /// <summary>
    /// Parse JSON in server message.
    /// </summary>
    private static JsonElement ParseServerMessage(string message)
    {
      var parsedMessage = JsonDocument.Parse(message).RootElement;
      var error = parsedMessage.GetProperty("error").GetString();

      if (error is not null)
        throw new Exception(error);

      return parsedMessage.GetProperty("data");
    }
  }
}
