using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
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
      Console.WriteLine("started");
      File.WriteAllText("beep.txt", $"fjsdoai");
      var client = new WebSocketClient(ipcServerPort);

      try
      {
        await client.ConnectAsync(CancellationToken.None);

        // Query for initial windows via IPC server.
        await client.SendTextAsync("windows", CancellationToken.None);
        var windowsResponse = await client.ReceiveTextAsync(CancellationToken.None);

        var initialWindows = JsonParser.ToInstance<ServerMessage<List<Window>>>(
          windowsResponse,
          _serializeOptions
        );

        await client.SendTextAsync("subscribe -e focus_changed", CancellationToken.None);

        while (true)
        {
          var res = await client.ReceiveTextAsync(CancellationToken.None);
          Console.WriteLine($"res {res}");
        }
      }
      catch (Exception e)
      {
        Console.WriteLine($"caught {e}");
        File.WriteAllText("errors.txt", $"aaaaaaa: {e}");
        RestoreManagedHandles();
        await client.DisconnectAsync(CancellationToken.None);
        return ExitCode.Success;
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
