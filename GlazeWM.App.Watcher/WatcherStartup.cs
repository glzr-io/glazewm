using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.Utils;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.App.Watcher
{
  public sealed class WatcherStartup
  {
    private readonly JsonSerializerOptions _serializeOptions =
      JsonParser.OptionsFactory((options) =>
      {
        options.PropertyNamingPolicy = JsonNamingPolicy.CamelCase;
        options.Converters.Add(new JsonContainerConverter());
      });

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

        foreach (var handle in await GetWindowHandles(client))
          _managedHandles.Add(handle);

        await client.SendTextAsync(
          "subscribe -e window_managed,window_unmanaged",
          CancellationToken.None
        );

        while (true)
        {
          var res = await client.ReceiveTextAsync(CancellationToken.None);
          Console.WriteLine($"res {res}");
        }
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
    private async Task<IEnumerable<IntPtr>> GetWindowHandles(WebSocketClient client)
    {
      await client.SendTextAsync("windows", CancellationToken.None);

      var windowsResponse = await client.ReceiveTextAsync(CancellationToken.None);

      return ParseServerMessage(windowsResponse)
        .EnumerateArray()
        .Select(value => new IntPtr(value.GetInt32()));
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
