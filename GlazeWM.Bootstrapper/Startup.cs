using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Forms;
using GlazeWM.Bar;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bootstrapper
{
  internal sealed class Startup
  {
    private readonly BarService _barService;
    private readonly Bus _bus;
    private readonly KeybindingService _keybindingService;
    private readonly SystemEventService _systemEventService;
    private readonly WindowEventService _windowEventService;

    private NetworkIcon _NetworkIcon { get; set; }

    public Startup(
      BarService barService,
      Bus bus,
      KeybindingService keybindingService,
      SystemEventService systemEventService,
      WindowEventService windowEventService)
    {
      _barService = barService;
      _bus = bus;
      _keybindingService = keybindingService;
      _systemEventService = systemEventService;
      _windowEventService = windowEventService;
    }

    public void Run()
    {
      try
      {
        // Set the process-default DPI awareness.
        _ = SetProcessDpiAwarenessContext(DpiAwarenessContext.PerMonitorAwareV2);

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

        var NetworkIconConfig = new NetworkIconConfig
        {
          HoverText = "GlazeWM",
          IconResourceName = "GlazeWM.Bootstrapper.icon.ico",
          Actions = new Dictionary<string, Action>
          {
            { "Reload config", () => _bus.Invoke(new ReloadUserConfigCommand()) },
            { "Exit", () => _bus.Invoke(new ExitApplicationCommand(false)) },
          }
        };

        // Add application to network.
        _NetworkIcon = new NetworkIcon(NetworkIconConfig);
        _NetworkIcon.Show();

        Application.Run();
      }
      catch (Exception exception)
      {
        _bus.Invoke(new HandleFatalExceptionCommand(exception));
      }
    }

    private void OnApplicationExit()
    {
      _bus.Invoke(new ShowAllWindowsCommand());
      _barService.ExitApp();
      _NetworkIcon?.Remove();
      Application.Exit();
    }
  }
}
