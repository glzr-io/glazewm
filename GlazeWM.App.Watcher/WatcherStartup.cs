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
    public static async Task<ExitCode> Run(int ipcServerPort)
    {
      var client = new WebSocketClient(ipcServerPort);
      var managedHandles = new List<IntPtr>();

      try
      {
        await client.ConnectAsync(CancellationToken.None);

        foreach (var handle in await GetInitialHandles(client))
          managedHandles.Add(handle);

        await client.SendTextAsync(
          "subscribe -e window_managed,window_unmanaged",
          CancellationToken.None
        );

        while (true)
        {
          var (isManaged, handle) = await GetManagedEvent(client);

          if (isManaged)
            managedHandles.Add(handle);
          else
            managedHandles.Remove(handle);
        }
      }
      catch (Exception)
      {
        RestoreHandles(managedHandles);
        await client.DisconnectAsync(CancellationToken.None);
        return ExitCode.Success;
      }
    }

    /// <summary>
    /// Query for initial window handles via IPC server.
    /// </summary>
    private static async Task<IEnumerable<IntPtr>> GetInitialHandles(WebSocketClient client)
    {
      await client.SendTextAsync("windows", CancellationToken.None);
      var response = await client.ReceiveTextAsync(CancellationToken.None);

      return ParseServerMessage(response)
        .EnumerateArray()
        .Select(value => new IntPtr(value.GetInt32()));
    }

    /// <summary>
    /// Get window handles from managed and unmanaged window events.
    /// </summary>
    private static async Task<(bool, IntPtr)> GetManagedEvent(WebSocketClient client)
    {
      var response = await client.ReceiveTextAsync(CancellationToken.None);
      var parsedResponse = ParseServerMessage(response);

      return parsedResponse.GetProperty("type").GetString() switch
      {
        "window_managed" =>
          (true, parsedResponse.GetProperty("handle").GetInt32()),
        "window_unmanaged" =>
          (false, parsedResponse.GetProperty("removedHandle").GetInt32()),
        _ => throw new Exception("Received unrecognized event.")
      };
    }

    /// <summary>
    /// Restore given window handles.
    /// </summary>
    private static void RestoreHandles(List<IntPtr> managedHandles)
    {
      foreach (var handle in managedHandles)
        ShowWindow(handle, ShowWindowFlags.ShowNoActivate);
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
