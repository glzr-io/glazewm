using GlazeWM.Bar;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.WindowsApi;
using GlazeWM.Infrastructure.WindowsApi.Events;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive.Linq;
using System.Text.Json.Serialization;
using System.Windows.Forms;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bootstrapper
{
  internal class Startup
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly JsonService _jsonService;
    private readonly KeybindingService _keybindingService;
    private readonly WindowEventService _windowEventService;
    private readonly BarService _barService;
    private readonly SystemTrayService _systemTrayService;
    private readonly SystemEventService _systemEventService;

    public Startup(
      Bus bus,
      ContainerService containerService,
      JsonService jsonService,
      KeybindingService keybindingService,
      WindowEventService windowEventService,
      BarService barService,
      SystemTrayService systemTrayService,
      SystemEventService systemEventService)
    {
      _bus = bus;
      _containerService = containerService;
      _jsonService = jsonService;
      _keybindingService = keybindingService;
      _windowEventService = windowEventService;
      _barService = barService;
      _systemTrayService = systemTrayService;
      _systemEventService = systemEventService;
    }

    public void Run()
    {
      // Set the process-default DPI awareness.
      _ = SetProcessDpiAwarenessContext(DpiAwarenessContext.Context_PerMonitorAwareV2);

      _bus.Events.OfType<ApplicationExitingEvent>()
        .Subscribe(_ => OnApplicationExit());

      // Launch bar WPF application. Spawns bar window when monitors are added, so the service needs
      // to be initialized before populating initial state.
      _barService.StartApp();

      // Populate initial monitors, windows, workspaces and user config.
      _bus.Invoke(new PopulateInitialStateCommand());

      // Listen on registered keybindings.
      _keybindingService.Start();

      // Listen for window events (eg. close, focus).
      _windowEventService.Start();

      // Listen for system-related events (eg. changes to display settings).
      _systemEventService.Start();

      // Add application to system tray.
      _systemTrayService.AddToSystemTray();

      Application.Run();
    }

    private void OnApplicationExit()
    {
      WriteStateDumpToErrorLog();
      _bus.Invoke(new ShowAllWindowsCommand());
      _barService.ExitApp();
      _systemTrayService.RemoveFromSystemTray();
      Application.Exit();
      // TODO: Use exit code 1 if exiting due to an unhandled error.
      Environment.Exit(0);
    }

    // TODO: Move to dedicated logging service.
    private void WriteStateDumpToErrorLog()
    {
      var errorLogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
        "./.glaze-wm/errors.log"
      );

      Directory.CreateDirectory(Path.GetDirectoryName(errorLogPath));

      try
      {
        var stateDump = _jsonService.Serialize(_containerService.ContainerTree, new List<JsonConverter> { new JsonContainerConverter() });
        File.AppendAllText(errorLogPath, $"\nState dump: {stateDump}");
      }
      catch (Exception)
      {
        File.AppendAllText(errorLogPath, "\nFailed to get state dump.");
      }
    }
  }
}
